#!/usr/bin/env python3
"""
SFT + DPO staged training for VL-Computer-Use using TRL.

Phase 1: Collect demo trajectories via DOM ground truth
Phase 2: Train with TRL SFTTrainer (then DPO later)
Phase 3: Save checkpoint for loading into RL / vLLM

Server (localhost:8080) handles browser control + screenshots.
Training runs locally using TRL on cuda:1.

Usage:
    uv run python train_stages.py --stage 4 --demos 10
    uv run python train_stages.py --stage 4 --demos 10 --resume /path/to/checkpoint
    uv run python train_stages.py --assess-only
"""
import argparse
import time
import sys
from pathlib import Path
from datetime import datetime
from PIL import Image
import requests

SERVER = "http://localhost:8080"
SITE = "http://127.0.0.1:37163"

STAGE_TASKS = {
    1: "Click the Submit button.",
    2: "Click buttons to find the correct one. Wrong buttons turn red. Use feedback to find the right one.",
    3: "Click buttons to find the correct one. Buttons shuffle after each wrong click. Re-scan and adapt.",
    4: "Click the 4 numbered buttons in order: 1, 2, 3, 4. A visual indicator shows which is next.",
    5: "Click the real Submit button. Ignore decoy buttons that look similar.",
    6: "Dismiss all popups blocking the page, then click the goal button.",
    7: "Scroll down to find and click the hidden target button.",
    8: "Read the code shown on the page, type it into the input field, then click Submit.",
    9: "Click the moving target element. Predict its position from its animation.",
    10: "Complete the full challenge: dismiss popups, find the code, type it, and submit. All distractors active.",
}

STAGE_MAX_STEPS = {
    1: 3, 2: 8, 3: 10, 4: 8, 5: 3,
    6: 10, 7: 10, 8: 8, 9: 10, 10: 15,
}

SYSTEM_PROMPT = """You control a browser. Each turn you see a screenshot.

First describe what you see, then choose an action.

Output format (exactly two lines):
SEE: <brief description of visible elements, their labels, positions, and any changes from last frame>
ACTION: <one action>

Actions (normalized 0-1000 coordinates):
- CLICK x y - Click at coordinates
- TYPE text - Type text into focused input
- KEY keyname - Press key (enter, tab, escape, etc)
- SCROLL dy - Scroll (positive=down, negative=up)
- WAIT - Wait and observe (nothing to do yet)
- DONE - Task complete"""

OUTPUT_DIR = Path("/tmp/vl-checkpoints")


def max_steps(stage: int) -> int:
    return STAGE_MAX_STEPS.get(stage, 15)


def log(msg: str, level: str = "INFO"):
    ts = datetime.now().strftime("%H:%M:%S")
    print(f"[{ts}] [{level}] {msg}", flush=True)


def init_wandb(run_name: str, config: dict) -> object | None:
    """Initialize Weights & Biases if WANDB_API_KEY is set."""
    import os

    if not os.getenv("WANDB_API_KEY"):
        return None

    try:
        import wandb
    except Exception as e:
        log(f"wandb not available ({e}), skipping W&B logging", "WARN")
        return None

    project = os.getenv("WANDB_PROJECT", "vl-computer-use")
    entity = os.getenv("WANDB_ENTITY")
    mode = os.getenv("WANDB_MODE", "online")
    run = wandb.init(project=project, entity=entity, name=run_name, config=config, mode=mode)
    return run


# --- Server API helpers (browser control only) ---

def api(method: str, endpoint: str, data: dict = None, timeout: int = 120) -> dict:
    url = f"{SERVER}{endpoint}"
    try:
        if method == "GET":
            resp = requests.get(url, timeout=timeout)
        else:
            resp = requests.post(url, json=data, timeout=timeout)
        return resp.json()
    except requests.exceptions.Timeout:
        raise RuntimeError(f"Timeout on {endpoint}")
    except Exception as e:
        raise RuntimeError(f"API error on {endpoint}: {e}")


def js_eval(js: str):
    return api("POST", "/eval", {"js": js}).get("result")


def navigate(url: str):
    return api("POST", "/navigate", {"url": url})


def get_task_state() -> dict:
    """Get task state — tries Dioxus score first, falls back to legacy getTaskState."""
    # Dioxus levels: read score from DOM
    score_text = js_eval('document.querySelector("[style*=\\"22c55e\\"]")?.textContent')
    if score_text and "score:" in str(score_text):
        score = int(str(score_text).split(":")[-1].strip())
        return {"score": score, "completed": score > 0}
    # Legacy stages
    return js_eval("window.getTaskState()") or {}


def execute(action: str):
    return api("POST", "/execute", {"action": action.replace(",", "")})


def take_screenshot() -> Image.Image:
    """Take screenshot via server, return as PIL Image."""
    result = api("GET", "/screenshot")
    path = result.get("path")
    if not path:
        raise RuntimeError(f"Screenshot failed: {result}")
    return Image.open(path).resize((1280, 704), Image.Resampling.LANCZOS)


def get_dom_elements() -> list:
    return api("GET", "/dom").get("elements", [])


def infer(task: str) -> dict:
    return api("POST", "/infer", {"task": task, "temperature": 0.7})


def reset_trajectory():
    return api("POST", "/reset_trajectory")


# --- Data collection ---

def resolve_expected(expected: str, elements: list, task_state: dict) -> str:
    """Convert expected_next hint from page to an actual action with coordinates."""
    parts = expected.split()
    verb = parts[0].upper() if parts else ""
    target = "_".join(parts[1:]) if len(parts) > 1 else ""
    target_parts = [p.lower() for p in target.replace("_", " ").split() if p]

    def el_coords(el):
        c = el["coords"]["normalized"]
        return f"CLICK {c['x']} {c['y']}"

    if verb == "CLICK":
        for el in elements:
            if el.get("id") == target:
                return el_coords(el)
        for el in elements:
            el_text = (el.get("text") or "").strip().lower()
            if el_text and el_text in target_parts:
                return el_coords(el)
        for el in elements:
            el_id = (el.get("id") or "").lower()
            el_text = (el.get("text") or "").lower()
            for tp in target_parts:
                if tp in el_id or tp in el_text:
                    return el_coords(el)

    elif verb == "DISMISS":
        for el in elements:
            el_id = (el.get("id") or "").lower()
            el_text = (el.get("text") or "").lower()
            el_classes = " ".join(el.get("classes") or []).lower()
            if (any(tp in el_id for tp in target_parts) or
                any(tp in el_classes for tp in target_parts) or
                any(w in el_text for w in ["close", "dismiss", "accept", "got it", "\u00d7", "x", "ok"]) or
                any(w in el_classes for w in ["close", "dismiss"])):
                return el_coords(el)

    elif verb == "SCROLL":
        return "SCROLL 3"

    elif verb == "TYPE":
        custom = task_state.get("custom", {})
        code = custom.get("expected_code", "")
        if code:
            return f"TYPE {code}"

    return "WAIT"


def collect_demo(stage: int) -> dict | None:
    """
    Collect one correct demo trajectory.

    Uses DOM ground truth to navigate the stage correctly.
    Returns {"messages": [...], "images": [PIL, ...]} or None.
    Images are extracted into a separate list; messages use {"type": "image"} placeholders.
    """
    task = STAGE_TASKS.get(stage, "Complete the task.")
    navigate(f"{SITE}/level{stage}")
    time.sleep(1)

    system_content = f"{SYSTEM_PROMPT}\n\nTask: {task}"
    messages = [{"role": "system", "content": [{"type": "text", "text": system_content}]}]
    images = []

    t0 = time.time()

    for step in range(max_steps(stage)):
        task_state = get_task_state()
        if not task_state:
            break
        if task_state.get("completed"):
            break

        expected = task_state.get("expected_next", "")
        if not expected:
            break

        # Screenshot
        img = take_screenshot()
        images.append(img)
        t_rel = time.time() - t0

        # DOM for building SEE description
        elements = get_dom_elements()
        visible = []
        for el in elements[:8]:
            if el.get("text"):
                coords = el.get("coords", {}).get("normalized", {})
                visible.append(f'{el["tag"]} "{el["text"]}" at ({coords.get("x", "?")}, {coords.get("y", "?")})')
        see_text = ", ".join(visible) if visible else "Page elements visible."

        # Resolve correct action from DOM ground truth
        action = resolve_expected(expected, elements, task_state)
        if action == "WAIT":
            log(f"  Step {step}: can't resolve expected={expected}", "WARN")
            break

        # Add user turn (image placeholder + timestamp)
        messages.append({
            "role": "user",
            "content": [
                {"type": "image"},
                {"type": "text", "text": f"[t={t_rel:.2f}s]"},
            ],
        })

        # Add assistant turn (reasoning + action)
        response = f"SEE: {see_text}\nACTION: {action}"
        messages.append({
            "role": "assistant",
            "content": [{"type": "text", "text": response}],
        })

        # Execute to advance the page
        execute(action)
        time.sleep(0.3)

    # Verify completion
    task_state = get_task_state()
    if task_state and task_state.get("completed"):
        log(f"  Demo OK: {len(images)} actions")
        return {"messages": messages, "images": images}
    else:
        log(f"  Demo FAIL at step {step}", "WARN")
        return None


def collect_rollout(stage: int) -> dict | None:
    """
    Collect one model rollout via vLLM inference.

    Returns {"messages": [...], "images": [PIL, ...]} or None.
    Same format as collect_demo for DPO pairing.
    """
    task = STAGE_TASKS.get(stage, "Complete the task.")
    navigate(f"{SITE}/level{stage}")
    time.sleep(1)
    reset_trajectory()

    system_content = f"{SYSTEM_PROMPT}\n\nTask: {task}"
    messages = [{"role": "system", "content": [{"type": "text", "text": system_content}]}]
    images = []
    t0 = time.time()

    for step in range(max_steps(stage)):
        task_state = get_task_state()
        if task_state and task_state.get("completed"):
            break

        # Take screenshot before inference
        img = take_screenshot()
        images.append(img)
        t_rel = time.time() - t0

        messages.append({
            "role": "user",
            "content": [
                {"type": "image"},
                {"type": "text", "text": f"[t={t_rel:.2f}s]"},
            ],
        })

        result = infer(task)
        if "error" in result:
            break

        response = result.get("full_response", result.get("action", "WAIT"))
        messages.append({
            "role": "assistant",
            "content": [{"type": "text", "text": response}],
        })

        action = result.get("action", "WAIT")
        if action.upper().startswith("DONE"):
            break
        if action.upper().startswith("WAIT"):
            time.sleep(0.25)
            continue

        execute(action)
        time.sleep(0.3)

    if not images:
        return None

    return {"messages": messages, "images": images}


# --- TRL Training ---

def load_model_and_processor(resume_from: str = None):
    """Load Qwen3-VL-8B with LoRA on cuda:1 for training."""
    import torch
    from transformers import Qwen3VLForConditionalGeneration, AutoProcessor
    from peft import get_peft_model, LoraConfig, PeftModel

    log("Loading Qwen3-VL-8B + LoRA on cuda:1...")

    model = Qwen3VLForConditionalGeneration.from_pretrained(
        "Qwen/Qwen3-VL-8B-Instruct",
        torch_dtype=torch.bfloat16,
        device_map="auto",
        attn_implementation="flash_attention_2",
    )

    processor = AutoProcessor.from_pretrained(
        "Qwen/Qwen3-VL-8B-Instruct",
        min_pixels=256 * 28 * 28,
        max_pixels=512 * 28 * 28,
    )

    if resume_from:
        log(f"Loading LoRA from {resume_from}")
        model = PeftModel.from_pretrained(model, resume_from)
    else:
        lora_config = LoraConfig(
            r=16,
            lora_alpha=16,
            lora_dropout=0,
            bias="none",
            target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                             "gate_proj", "up_proj", "down_proj"],
            task_type="CAUSAL_LM",
        )
        model = get_peft_model(model, lora_config)

    trainable = sum(p.numel() for p in model.parameters() if p.requires_grad)
    total = sum(p.numel() for p in model.parameters())
    log(f"Model loaded: {trainable:,} trainable / {total:,} total ({100*trainable/total:.2f}%)")

    return model, processor


def tokenize_demo(messages: list, images: list, processor) -> dict:
    """
    Tokenize a demo and mask labels so loss is only on assistant tokens.

    Returns dict with input_ids, attention_mask, labels, pixel_values, image_grid_thw.
    """
    import torch

    text = processor.apply_chat_template(messages, tokenize=False, add_generation_prompt=False)
    inputs = processor(text=text, images=images, return_tensors="pt", padding=False)

    input_ids = inputs["input_ids"][0]
    labels = input_ids.clone()

    # Mask everything except assistant content
    tokenizer = processor.tokenizer
    im_start_id = tokenizer.convert_tokens_to_ids("<|im_start|>")
    im_end_id = tokenizer.convert_tokens_to_ids("<|im_end|>")
    assistant_ids = tokenizer.encode("assistant\n", add_special_tokens=False)
    prefix_len = len(assistant_ids)

    ids = input_ids.tolist()
    # Start with everything masked
    labels[:] = -100

    i = 0
    while i < len(ids):
        if ids[i] == im_start_id:
            next_tokens = ids[i + 1:i + 1 + prefix_len]
            if next_tokens == assistant_ids:
                # Unmask assistant content (after "assistant\n", before <|im_end|>)
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
    log(f"    Tokenized: {n_total} tokens, {n_train} trainable ({100*n_train/n_total:.1f}%)")

    result = {"input_ids": input_ids, "labels": labels, "attention_mask": inputs["attention_mask"][0]}
    if "pixel_values" in inputs:
        result["pixel_values"] = inputs["pixel_values"]
    if "image_grid_thw" in inputs:
        result["image_grid_thw"] = inputs["image_grid_thw"]
    result["n_tokens"] = n_total
    result["n_train_tokens"] = n_train
    result["n_images"] = len(images)
    return result


def get_sequence_logprobs(model, batch, device):
    """Compute sum of log probs on assistant tokens (where labels != -100)."""
    import torch
    import torch.nn.functional as F

    inputs = {
        "input_ids": batch["input_ids"].unsqueeze(0).to(device),
        "attention_mask": batch["attention_mask"].unsqueeze(0).to(device),
    }
    if "pixel_values" in batch:
        inputs["pixel_values"] = batch["pixel_values"].to(device)
    if "image_grid_thw" in batch:
        inputs["image_grid_thw"] = batch["image_grid_thw"].to(device)

    with torch.amp.autocast("cuda", dtype=torch.bfloat16):
        outputs = model(**inputs)

    # logprobs for each token predicting the NEXT token
    logits = outputs.logits[0, :-1]  # (seq_len-1, vocab)
    targets = batch["labels"][1:].to(device)  # (seq_len-1,)

    log_probs = F.log_softmax(logits.float(), dim=-1)
    # Gather log prob of actual target tokens
    token_logprobs = log_probs.gather(1, targets.unsqueeze(1).clamp(min=0)).squeeze(1)
    # Zero out masked positions (labels == -100)
    mask = (targets != -100).float()
    total_logprob = (token_logprobs * mask).sum()

    del inputs, outputs, logits, log_probs
    return total_logprob, mask.sum().item()


def train_dpo(model, processor, pairs: list, output_dir: str, beta: float = 0.1):
    """
    DPO training with manual assistant-token masking.

    pairs: list of {"chosen": {messages, images}, "rejected": {messages, images}}
    """
    import torch
    import torch.nn.functional as F

    log(f"DPO training on {len(pairs)} pairs (beta={beta}) → {output_dir}")
    log("Tokenizing pairs...")

    tokenized_pairs = []
    for p in pairs:
        chosen = tokenize_demo(p["chosen"]["messages"], p["chosen"]["images"], processor)
        rejected = tokenize_demo(p["rejected"]["messages"], p["rejected"]["images"], processor)
        tokenized_pairs.append({"chosen": chosen, "rejected": rejected})

    device = next(model.parameters()).device
    run = init_wandb(
        run_name=f"dpo_{len(pairs)}pairs_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
        config={
            "beta": beta,
            "pairs": len(pairs),
            "model": "Qwen/Qwen3-VL-8B-Instruct",
            "epochs": 3,
            "lr": 5e-6,
            "weight_decay": 0.01,
            "grad_clip": 1.0,
        },
    )
    if run:
        avg_chosen_tokens = sum(p["chosen"]["n_tokens"] for p in tokenized_pairs) / len(tokenized_pairs)
        avg_rejected_tokens = sum(p["rejected"]["n_tokens"] for p in tokenized_pairs) / len(tokenized_pairs)
        avg_chosen_train = sum(p["chosen"]["n_train_tokens"] for p in tokenized_pairs) / len(tokenized_pairs)
        avg_rejected_train = sum(p["rejected"]["n_train_tokens"] for p in tokenized_pairs) / len(tokenized_pairs)
        avg_chosen_imgs = sum(p["chosen"]["n_images"] for p in tokenized_pairs) / len(tokenized_pairs)
        avg_rejected_imgs = sum(p["rejected"]["n_images"] for p in tokenized_pairs) / len(tokenized_pairs)
        run.log({
            "data/pairs": len(tokenized_pairs),
            "data/avg_chosen_tokens": avg_chosen_tokens,
            "data/avg_rejected_tokens": avg_rejected_tokens,
            "data/avg_chosen_train_tokens": avg_chosen_train,
            "data/avg_rejected_train_tokens": avg_rejected_train,
            "data/avg_chosen_images": avg_chosen_imgs,
            "data/avg_rejected_images": avg_rejected_imgs,
        })

    # Phase 1: Compute reference logprobs (before training)
    log("Computing reference logprobs...")
    model.eval()
    ref_logprobs = []
    for pair in tokenized_pairs:
        with torch.no_grad():
            chosen_ref, _ = get_sequence_logprobs(model, pair["chosen"], device)
            rejected_ref, _ = get_sequence_logprobs(model, pair["rejected"], device)
        ref_logprobs.append({"chosen": chosen_ref.item(), "rejected": rejected_ref.item()})
        torch.cuda.empty_cache()
    log("  Reference logprobs computed.")
    if run:
        avg_ref_margin = sum(r["chosen"] - r["rejected"] for r in ref_logprobs) / len(ref_logprobs)
        run.log({"ref/avg_margin": avg_ref_margin})

    # Phase 2: DPO training
    optimizer = torch.optim.AdamW(
        [p for p in model.parameters() if p.requires_grad],
        lr=5e-6, weight_decay=0.01,
    )

    model.train()
    model.enable_input_require_grads()
    model.gradient_checkpointing_enable()

    num_epochs = 3
    total_steps = 0

    for epoch in range(num_epochs):
        epoch_loss = 0.0
        epoch_acc = 0.0

        for i, (pair, ref) in enumerate(zip(tokenized_pairs, ref_logprobs)):
            # Current policy logprobs
            chosen_logp, n_chosen = get_sequence_logprobs(model, pair["chosen"], device)
            rejected_logp, n_rejected = get_sequence_logprobs(model, pair["rejected"], device)

            # DPO loss: -log(sigmoid(beta * (chosen_margin - rejected_margin)))
            chosen_margin = chosen_logp - ref["chosen"]
            rejected_margin = rejected_logp - ref["rejected"]

            logit = beta * (chosen_margin - rejected_margin)
            loss = -F.logsigmoid(logit)

            loss.backward()

            # Track accuracy (is chosen preferred?)
            acc = (logit > 0).float().item()
            epoch_acc += acc

            if (i + 1) % 2 == 0 or (i + 1) == len(tokenized_pairs):
                grad_norm = torch.nn.utils.clip_grad_norm_(model.parameters(), 1.0)
                optimizer.step()
                optimizer.zero_grad()
                total_steps += 1

                log(f"  epoch={epoch+1}/{num_epochs} step={total_steps} "
                    f"loss={loss.item():.4f} grad={grad_norm.item():.3f} "
                    f"margin={logit.item():.3f} acc={acc:.0f}")
                if run:
                    run.log({
                        "train/loss": loss.item(),
                        "train/grad_norm": grad_norm.item(),
                        "train/margin": logit.item(),
                        "train/acc": acc,
                        "train/chosen_logp": chosen_logp.item(),
                        "train/rejected_logp": rejected_logp.item(),
                        "train/chosen_ref": ref["chosen"],
                        "train/rejected_ref": ref["rejected"],
                        "train/step": total_steps,
                        "train/epoch": epoch + 1,
                    })

            epoch_loss += loss.item()

            del chosen_logp, rejected_logp, loss
            torch.cuda.empty_cache()

        avg_loss = epoch_loss / len(tokenized_pairs)
        avg_acc = epoch_acc / len(tokenized_pairs)
        log(f"  Epoch {epoch+1}: loss={avg_loss:.4f} acc={avg_acc:.1%}")
        if run:
            run.log({
                "epoch/loss": avg_loss,
                "epoch/acc": avg_acc,
                "epoch": epoch + 1,
            })

    model.eval()

    Path(output_dir).mkdir(parents=True, exist_ok=True)
    model.save_pretrained(output_dir)
    processor.save_pretrained(output_dir)
    log(f"Saved to {output_dir}")
    if run:
        run.finish()

    return {"loss": avg_loss, "steps": total_steps, "acc": avg_acc}


# --- Assessment (uses vLLM via server) ---

def assess_stage(stage: int) -> tuple:
    """Run one trial. Returns (completed, steps)."""
    navigate(f"{SITE}/level{stage}")
    time.sleep(1)
    reset_trajectory()

    task = STAGE_TASKS.get(stage, "Complete the task.")
    steps_limit = max_steps(stage)

    for step in range(steps_limit):
        task_state = get_task_state()
        if task_state and task_state.get("completed"):
            return True, step

        result = infer(task)
        if "error" in result:
            return False, step

        action = result.get("action", "WAIT")
        if action.upper().startswith("WAIT"):
            time.sleep(0.25)
            continue
        if action.upper().startswith("DONE"):
            break

        execute(action)
        time.sleep(0.3)

    task_state = get_task_state()
    return (task_state and task_state.get("completed", False)), steps_limit


def assess(max_stage: int = 10, trials: int = 5, run_name: str = None):
    log("=" * 50)
    log(f"ASSESSMENT - {trials} trials per stage")
    log("=" * 50)

    run = init_wandb(
        run_name=run_name or f"assess_{datetime.now().strftime('%Y%m%d_%H%M%S')}",
        config={"trials": trials, "max_stage": max_stage, "model": "Qwen/Qwen3-VL-8B-Instruct"},
    )

    results = {}
    total_pass, total_trials = 0, 0

    for stage in range(1, max_stage + 1):
        successes = 0
        stage_steps = []
        for trial in range(trials):
            completed, steps = assess_stage(stage)
            if completed:
                successes += 1
            stage_steps.append(steps)
            log(f"  Stage {stage} trial {trial+1}: {'PASS' if completed else 'FAIL'} ({steps} steps)")

            if run:
                run.log({
                    f"trial/stage{stage}_pass": int(completed),
                    f"trial/stage{stage}_steps": steps,
                })

        rate = successes / trials * 100
        avg_steps = sum(stage_steps) / len(stage_steps)
        results[stage] = {"pass_rate": rate, "successes": successes, "avg_steps": avg_steps}
        total_pass += successes
        total_trials += trials
        log(f"Stage {stage}: {successes}/{trials} ({rate:.0f}%)", "ASSESS")

        if run:
            run.log({
                f"stage/stage{stage}_pass_rate": rate,
                f"stage/stage{stage}_avg_steps": avg_steps,
            })

    overall = total_pass / total_trials * 100 if total_trials else 0
    log(f"\nOverall: {total_pass}/{total_trials} ({overall:.0f}%)", "ASSESS")

    if run:
        run.log({"assess/overall_pass_rate": overall, "assess/total_pass": total_pass})
        # Summary table
        try:
            import wandb
            table = wandb.Table(columns=["stage", "pass_rate", "successes", "trials", "avg_steps"])
            for stage, r in sorted(results.items()):
                table.add_data(stage, r["pass_rate"], r["successes"], trials, r["avg_steps"])
            run.log({"assess/summary": table})
        except Exception:
            pass
        run.finish()

    return results


# --- Main ---

def main():
    import os
    os.environ["CUDA_VISIBLE_DEVICES"] = "1"

    parser = argparse.ArgumentParser(description="VL-Computer-Use DPO Training")
    parser.add_argument("--stage", type=int, nargs="+", default=[4],
                        help="Stage(s) to train on (e.g., --stage 4 6 8)")
    parser.add_argument("--demos", type=int, default=10, help="DPO pairs to collect per stage")
    parser.add_argument("--resume", type=str, help="LoRA checkpoint to resume from")
    parser.add_argument("--output", type=str, default=str(OUTPUT_DIR), help="Output directory")
    parser.add_argument("--assess-only", action="store_true", help="Only run assessment")
    parser.add_argument("--assess-trials", type=int, default=5, help="Trials per stage")
    parser.add_argument("--collect-only", action="store_true", help="Only collect demos, don't train")
    args = parser.parse_args()

    log("=" * 50)
    log("VL-Computer-Use DPO Training")
    log(f"Stages: {args.stage} | Pairs: {args.demos}")
    log("=" * 50)

    # Check server is up
    try:
        api("GET", "/state")
        log("Server connected.")
    except Exception as e:
        log(f"Server not available: {e}", "ERROR")
        sys.exit(1)

    if args.assess_only:
        assess(trials=args.assess_trials)
        return

    # Phase 1: Collect DPO pairs (chosen=demo, rejected=rollout)
    pairs = []
    for stage in args.stage:
        task = STAGE_TASKS.get(stage, "?")
        log(f"\n{'='*50}", "STAGE")
        log(f"Collecting DPO pairs for stage {stage}: {task}", "STAGE")
        log(f"{'='*50}", "STAGE")

        for i in range(args.demos):
            log(f"  Pair {i+1}/{args.demos}...")

            # Chosen: correct demo from DOM
            demo = collect_demo(stage)
            if not demo:
                log("  Demo failed, skipping", "WARN")
                continue

            # Rejected: model rollout (likely fails)
            rollout = collect_rollout(stage)
            if not rollout:
                log("  Rollout failed, skipping", "WARN")
                continue

            pairs.append({"chosen": demo, "rejected": rollout})
            log(f"  Pair OK (chosen: {len(demo['images'])} imgs, rejected: {len(rollout['images'])} imgs)")

    log(f"\nCollected {len(pairs)} DPO pairs")

    if args.collect_only:
        return

    if not pairs:
        log("No pairs collected, exiting", "ERROR")
        sys.exit(1)

    # Phase 2: DPO training
    output_dir = Path(args.output) / f"dpo_{'_'.join(str(s) for s in args.stage)}"
    output_dir.mkdir(parents=True, exist_ok=True)

    model, processor = load_model_and_processor(args.resume)
    train_dpo(model, processor, pairs, str(output_dir))

    log(f"\nCheckpoint saved to: {output_dir}")
    log("Done. Load this into vLLM or merge for inference.")


if __name__ == "__main__":
    main()
