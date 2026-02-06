#!/usr/bin/env python3
"""
Multi-GPU SFT training on thought traces using accelerate.

Kills vLLM, uses both GPUs, then you restart vLLM after.

Usage:
    # First kill vLLM manually, then:
    accelerate launch --num_processes=2 train_traces_multi.py --epochs 3

    # Or with config:
    accelerate launch --config_file accelerate_config.yaml train_traces_multi.py
"""
import argparse
import json
import torch
from pathlib import Path
from datetime import datetime
from PIL import Image
from transformers import Qwen3VLForConditionalGeneration, AutoProcessor
from peft import get_peft_model, PeftModel, LoraConfig
from accelerate import Accelerator
from torch.utils.data import Dataset, DataLoader

TRACES_FILE = Path(__file__).parent / "traces" / "traces_expanded.json"
OUTPUT_DIR = Path("/tmp/vl-checkpoints")
MODEL_ID = "Qwen/Qwen3-VL-8B-Instruct"


def log(accelerator, msg, level="INFO"):
    if accelerator.is_main_process:
        ts = datetime.now().strftime("%H:%M:%S")
        print(f"[{ts}] [{level}] {msg}", flush=True)


class TraceDataset(Dataset):
    def __init__(self, traces, processor):
        self.traces = traces
        self.processor = processor
        self.tokenized = []

        for t in traces:
            images = [Image.open(p) for p in t["image_paths"]]
            tok = self._tokenize(t["messages"], images)
            self.tokenized.append(tok)

    def _tokenize(self, messages, images):
        text = self.processor.apply_chat_template(messages, tokenize=False, add_generation_prompt=False)
        inputs = self.processor(text=text, images=images, return_tensors="pt", padding=False)

        input_ids = inputs["input_ids"][0]
        labels = input_ids.clone()

        tokenizer = self.processor.tokenizer
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

        result = {
            "input_ids": input_ids,
            "labels": labels,
            "attention_mask": inputs["attention_mask"][0],
        }
        if "pixel_values" in inputs:
            result["pixel_values"] = inputs["pixel_values"]
        if "image_grid_thw" in inputs:
            result["image_grid_thw"] = inputs["image_grid_thw"]
        return result

    def __len__(self):
        return len(self.tokenized)

    def __getitem__(self, idx):
        return self.tokenized[idx]


def collate_fn(batch):
    """Custom collate - pad sequences to same length for batching."""
    if len(batch) == 1:
        return batch[0]

    # Find max length
    max_len = max(b["input_ids"].shape[0] for b in batch)

    # Pad all sequences
    padded = {
        "input_ids": [],
        "attention_mask": [],
        "labels": [],
    }

    for b in batch:
        seq_len = b["input_ids"].shape[0]
        pad_len = max_len - seq_len

        # Pad input_ids with 0 (or pad token)
        padded["input_ids"].append(
            torch.cat([b["input_ids"], torch.zeros(pad_len, dtype=torch.long)])
        )
        # Pad attention_mask with 0
        padded["attention_mask"].append(
            torch.cat([b["attention_mask"], torch.zeros(pad_len, dtype=torch.long)])
        )
        # Pad labels with -100 (ignore)
        padded["labels"].append(
            torch.cat([b["labels"], torch.full((pad_len,), -100, dtype=torch.long)])
        )

    result = {
        "input_ids": torch.stack(padded["input_ids"]),
        "attention_mask": torch.stack(padded["attention_mask"]),
        "labels": torch.stack(padded["labels"]),
    }

    # Handle pixel_values - concat along batch dim
    if "pixel_values" in batch[0]:
        result["pixel_values"] = torch.cat([b["pixel_values"] for b in batch], dim=0)
    if "image_grid_thw" in batch[0]:
        result["image_grid_thw"] = torch.cat([b["image_grid_thw"] for b in batch], dim=0)

    return result


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--epochs", type=int, default=5)
    parser.add_argument("--lr", type=float, default=2e-5)
    parser.add_argument("--batch-size", type=int, default=1)
    parser.add_argument("--resume", type=str, default=None)
    parser.add_argument("--output", type=str, default="sft_traces_multi")
    args = parser.parse_args()

    accelerator = Accelerator(gradient_accumulation_steps=2)

    log(accelerator, f"Devices: {accelerator.num_processes}")
    log(accelerator, f"Loading traces from {TRACES_FILE}")

    with open(TRACES_FILE) as f:
        traces = json.load(f)
    log(accelerator, f"Loaded {len(traces)} traces")

    # Load model
    log(accelerator, "Loading model...")
    model = Qwen3VLForConditionalGeneration.from_pretrained(
        MODEL_ID,
        torch_dtype=torch.bfloat16,
        attn_implementation="flash_attention_2",
        low_cpu_mem_usage=True,
        device_map={"": accelerator.local_process_index},  # each process loads to its GPU
    )
    processor = AutoProcessor.from_pretrained(MODEL_ID)

    if args.resume:
        log(accelerator, f"Loading LoRA from {args.resume}")
        model = PeftModel.from_pretrained(model, args.resume)
    else:
        lora_config = LoraConfig(
            r=16, lora_alpha=16, lora_dropout=0, bias="none",
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                           "gate_proj", "up_proj", "down_proj"],
            task_type="CAUSAL_LM",
        )
        model = get_peft_model(model, lora_config)

    model.gradient_checkpointing_enable()
    model.enable_input_require_grads()

    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    log(accelerator, f"Model: {trainable:,} trainable / {total:,} total")

    # Dataset
    log(accelerator, "Tokenizing traces...")
    dataset = TraceDataset(traces, processor)
    for i, t in enumerate(traces):
        n_train = (dataset.tokenized[i]["labels"] != -100).sum().item()
        n_total = len(dataset.tokenized[i]["input_ids"])
        log(accelerator, f"  Trace {i}: {n_total} tokens, {n_train} trainable")

    dataloader = DataLoader(dataset, batch_size=args.batch_size, shuffle=True, collate_fn=collate_fn)

    optimizer = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=args.lr, weight_decay=0.01,
    )

    # Prepare with accelerator
    model, optimizer, dataloader = accelerator.prepare(model, optimizer, dataloader)

    log(accelerator, f"\nTraining: {args.epochs} epochs, lr={args.lr}")

    for epoch in range(args.epochs):
        model.train()
        epoch_loss = 0.0

        for step, batch in enumerate(dataloader):
            with accelerator.accumulate(model):
                # Handle both batched and single-item cases
                if batch["input_ids"].dim() == 1:
                    inputs = {
                        "input_ids": batch["input_ids"].unsqueeze(0),
                        "attention_mask": batch["attention_mask"].unsqueeze(0),
                        "labels": batch["labels"].unsqueeze(0),
                    }
                else:
                    inputs = {
                        "input_ids": batch["input_ids"],
                        "attention_mask": batch["attention_mask"],
                        "labels": batch["labels"],
                    }
                if "pixel_values" in batch:
                    inputs["pixel_values"] = batch["pixel_values"]
                if "image_grid_thw" in batch:
                    inputs["image_grid_thw"] = batch["image_grid_thw"]

                # Log actual batch info
                bs = inputs["input_ids"].shape[0] if inputs["input_ids"].dim() > 1 else 1
                seq_len = inputs["input_ids"].shape[-1]
                log(accelerator, f"    batch_size={bs}, seq_len={seq_len}, total_tokens={bs*seq_len}")

                outputs = model(**inputs)
                loss = outputs.loss
                accelerator.backward(loss)

                if accelerator.sync_gradients:
                    accelerator.clip_grad_norm_(model.parameters(), 1.0)

                optimizer.step()
                optimizer.zero_grad()

                epoch_loss += loss.item()
                log(accelerator, f"  epoch={epoch+1}/{args.epochs} step={step+1} loss={loss.item():.4f}")

        avg_loss = epoch_loss / len(dataloader)
        log(accelerator, f"  Epoch {epoch+1}: avg_loss={avg_loss:.4f}")

    # Save
    accelerator.wait_for_everyone()
    if accelerator.is_main_process:
        output_path = OUTPUT_DIR / args.output
        output_path.mkdir(parents=True, exist_ok=True)
        unwrapped = accelerator.unwrap_model(model)
        unwrapped.save_pretrained(str(output_path))
        log(accelerator, f"\nSaved to {output_path}")


if __name__ == "__main__":
    main()
