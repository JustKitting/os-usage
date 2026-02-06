#!/usr/bin/env python3
"""
SFT on collected thought traces (OBSERVE → THINK → ACT pattern).

Loads traces from traces/traces.json, trains with assistant-only loss masking.

Usage:
    uv run python train_traces.py
    uv run python train_traces.py --epochs 5 --lr 1e-5
    uv run python train_traces.py --resume /tmp/vl-checkpoints/sft_traces
"""
import os
os.environ["CUDA_VISIBLE_DEVICES"] = "1"

import argparse
import json
import math
from pathlib import Path
from datetime import datetime

import torch
import torch.nn.functional as F
from PIL import Image
from transformers import Qwen3VLForConditionalGeneration, AutoProcessor
from peft import get_peft_model, PeftModel, LoraConfig

from trainer.utils import MetricsLogger

TRACES_FILE = Path(__file__).parent / "traces" / "traces.json"
OUTPUT_DIR = Path("/tmp/vl-checkpoints")
MODEL_ID = "Qwen/Qwen3-VL-8B-Instruct"


def log(msg, level="INFO"):
    ts = datetime.now().strftime("%H:%M:%S")
    print(f"[{ts}] [{level}] {msg}", flush=True)


def load_model(resume_from=None):
    log("Loading model...")
    model = Qwen3VLForConditionalGeneration.from_pretrained(
        MODEL_ID,
        torch_dtype=torch.bfloat16,
        device_map="auto",
        attn_implementation="flash_attention_2",
    )
    processor = AutoProcessor.from_pretrained(MODEL_ID)

    if resume_from:
        log(f"Loading LoRA from {resume_from}")
        model = PeftModel.from_pretrained(model, resume_from)
    else:
        lora_config = LoraConfig(
            r=16, lora_alpha=16, lora_dropout=0, bias="none",
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                             "gate_proj", "up_proj", "down_proj"],
            task_type="CAUSAL_LM",
        )
        model = get_peft_model(model, lora_config)

    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    log(f"Model: {trainable:,} trainable / {total:,} total ({100*trainable/total:.2f}%)")
    return model, processor


def tokenize_trace(messages, images, processor):
    """Tokenize a trace with assistant-only label masking."""
    text = processor.apply_chat_template(messages, tokenize=False, add_generation_prompt=False)
    inputs = processor(text=text, images=images, return_tensors="pt", padding=False)

    input_ids = inputs["input_ids"][0]
    labels = input_ids.clone()

    tokenizer = processor.tokenizer
    im_start_id = tokenizer.convert_tokens_to_ids("<|im_start|>")
    im_end_id = tokenizer.convert_tokens_to_ids("<|im_end|>")
    assistant_ids = tokenizer.encode("assistant\n", add_special_tokens=False)
    prefix_len = len(assistant_ids)

    ids = input_ids.tolist()
    labels[:] = -100

    i = 0
    while i < len(ids):
        if ids[i] == im_start_id:
            next_tokens = ids[i + 1:i + 1 + prefix_len]
            if next_tokens == assistant_ids:
                content_start = i + 1 + prefix_len
                j = content_start
                while j < len(ids) and ids[j] != im_end_id:
                    j += 1
                labels[content_start:j] = input_ids[content_start:j]
                i = j + 1
                continue
        i += 1

    n_train = (labels != -100).sum().item()
    n_total = len(ids)
    log(f"  Tokenized: {n_total} tokens, {n_train} trainable ({100*n_train/n_total:.1f}%)")

    result = {"input_ids": input_ids, "labels": labels, "attention_mask": inputs["attention_mask"][0]}
    if "pixel_values" in inputs:
        result["pixel_values"] = inputs["pixel_values"]
    if "image_grid_thw" in inputs:
        result["image_grid_thw"] = inputs["image_grid_thw"]
    return result


def train_step(model, batch, device):
    """Single forward/backward pass, return loss."""
    inputs = {
        "input_ids": batch["input_ids"].unsqueeze(0).to(device),
        "attention_mask": batch["attention_mask"].unsqueeze(0).to(device),
        "labels": batch["labels"].unsqueeze(0).to(device),
    }
    if "pixel_values" in batch:
        inputs["pixel_values"] = batch["pixel_values"].to(device)
    if "image_grid_thw" in batch:
        inputs["image_grid_thw"] = batch["image_grid_thw"].to(device)

    with torch.amp.autocast("cuda", dtype=torch.bfloat16):
        outputs = model(**inputs)

    loss = outputs.loss
    loss.backward()

    del inputs, outputs
    return loss.item()


def evaluate(model, batches, device):
    """Evaluate on held-out batches without gradient updates."""
    if not batches:
        return {}

    model.eval()
    losses = []

    with torch.no_grad():
        for batch in batches:
            inputs = {
                "input_ids": batch["input_ids"].unsqueeze(0).to(device),
                "attention_mask": batch["attention_mask"].unsqueeze(0).to(device),
                "labels": batch["labels"].unsqueeze(0).to(device),
            }
            if "pixel_values" in batch:
                inputs["pixel_values"] = batch["pixel_values"].to(device)
            if "image_grid_thw" in batch:
                inputs["image_grid_thw"] = batch["image_grid_thw"].to(device)

            with torch.amp.autocast("cuda", dtype=torch.bfloat16):
                outputs = model(**inputs)

            losses.append(outputs.loss.item())

            del inputs, outputs

    model.train()

    avg_loss = sum(losses) / len(losses)
    return {
        "val_loss": avg_loss,
        "val_perplexity": math.exp(avg_loss),
        "val_batches": len(losses),
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", type=int, default=5)
    parser.add_argument("--lr", type=float, default=2e-5)
    parser.add_argument("--grad-accum", type=int, default=2)
    parser.add_argument("--grad-clip", type=float, default=1.0)
    parser.add_argument("--val-split", type=float, default=0.1,
                        help="Fraction of traces reserved for validation (0-0.5 recommended)")
    parser.add_argument("--resume", type=str, default=None)
    parser.add_argument("--output", type=str, default="sft_traces")
    parser.add_argument("--metrics-file", type=str, default=None,
                        help="Optional JSONL file for structured training metrics")
    args = parser.parse_args()

    # Load traces
    log(f"Loading traces from {TRACES_FILE}")
    with open(TRACES_FILE) as f:
        traces = json.load(f)
    log(f"Loaded {len(traces)} traces")

    # Load images for each trace
    trace_data = []
    for t in traces:
        images = [Image.open(p) for p in t["image_paths"]]
        trace_data.append({"messages": t["messages"], "images": images, "stage": t["stage"]})
        log(f"  Stage {t['stage']}: {len(images)} images, {len(t['messages'])} messages")

    # Load model
    model, processor = load_model(args.resume)
    device = next(model.parameters()).device

    # Tokenize all traces
    log("Tokenizing traces...")
    tokenized = []
    for td in trace_data:
        tok = tokenize_trace(td["messages"], td["images"], processor)
        tokenized.append(tok)

    if not tokenized:
        log("No traces available for training", level="ERROR")
        return

    val_split = max(0.0, min(args.val_split, 0.5))
    val_count = int(len(tokenized) * val_split)
    if val_count > 0:
        val_batches = tokenized[-val_count:]
        train_batches = tokenized[:-val_count]
    else:
        val_batches = []
        train_batches = tokenized

    if not train_batches:
        log("Validation split too large, no training data left", level="ERROR")
        return

    # Training
    model.train()
    model.enable_input_require_grads()
    model.gradient_checkpointing_enable()

    optimizer = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=args.lr, weight_decay=0.01,
    )

    log(f"\nTraining: {args.epochs} epochs, lr={args.lr}, grad_accum={args.grad_accum}, "
        f"grad_clip={args.grad_clip}")
    log(f"Traces: train={len(train_batches)} val={len(val_batches)} output: {args.output}")

    metrics_logger = MetricsLogger(args.metrics_file)

    total_steps = 0
    for epoch in range(args.epochs):
        epoch_loss = 0.0
        optimizer.zero_grad()

        for i, batch in enumerate(train_batches):
            loss = train_step(model, batch, device)
            epoch_loss += loss

            if (i + 1) % args.grad_accum == 0 or (i + 1) == len(train_batches):
                grad_norm = torch.nn.utils.clip_grad_norm_(model.parameters(), args.grad_clip)
                optimizer.step()
                optimizer.zero_grad()
                total_steps += 1

                log(f"  epoch={epoch+1}/{args.epochs} step={total_steps} "
                    f"loss={loss:.4f} grad={grad_norm.item():.3f}")

            torch.cuda.empty_cache()

        avg_loss = epoch_loss / len(train_batches)
        val_metrics = evaluate(model, val_batches, device) if val_batches else {}

        log_msg = f"  Epoch {epoch+1}: train_loss={avg_loss:.4f}"
        if val_metrics:
            log_msg += f" val_loss={val_metrics['val_loss']:.4f} val_ppl={val_metrics['val_perplexity']:.2f}"
        log(log_msg)

        metrics_payload = {
            "epoch": epoch + 1,
            "train_loss": avg_loss,
            "steps": total_steps,
            "grad_clip": args.grad_clip,
        }
        if val_metrics:
            metrics_payload.update(val_metrics)
        metrics_logger.log("sft_epoch", metrics_payload)

    # Save
    output_path = OUTPUT_DIR / args.output
    output_path.mkdir(parents=True, exist_ok=True)
    model.save_pretrained(str(output_path))
    log(f"\nSaved to {output_path}")


if __name__ == "__main__":
    main()
