"""
Training signal injection for trajectory-based and oracle corrections.

TrajectoryTrainer: Trains on interleaved observation-action sequences.
TrainingInjector: Legacy oracle-based correction injection.
"""
import torch
import gc
from dataclasses import dataclass
from typing import Optional
from PIL import Image
from trainer.utils import MetricsLogger


class TrajectoryTrainer:
    """
    Trains on interleaved image-action trajectories using sliding window unrolling.

    A trajectory of N actions produces N training examples:
      Example 1: [img1]                              → predict act1
      Example 2: [img1, act1, img2]                   → predict act2
      ...
      Example W: [img1, act1, ..., imgW]              → predict actW   (window full)
      Example W+1: [img2, act2, ..., img(W+1)]        → predict act(W+1) (slides)
      ...

    Each example: loss only on the LAST action (the one being predicted).
    Earlier actions in the window are context. Gradients accumulate across
    all examples, one optimizer step at the end.
    """

    WINDOW_SIZE = 8  # default max image-action pairs in context

    def __init__(
        self,
        model,
        processor,
        lr: float = 2e-4,
        window_size: Optional[int] = None,
        grad_clip: float = 1.0,
        metrics_logger: Optional[MetricsLogger] = None,
    ):
        self.model = model
        self.processor = processor
        self.optimizer = None
        self.lr = lr
        self.train_steps = 0
        self.total_loss = 0.0
        self.window_size = window_size or self.WINDOW_SIZE
        self.grad_clip = grad_clip
        self.metrics_logger = metrics_logger or MetricsLogger()

    def _ensure_optimizer(self):
        if self.optimizer is None:
            self.optimizer = torch.optim.AdamW(
                [p for p in self.model.parameters() if p.requires_grad],
                lr=self.lr,
                weight_decay=0.01,
            )

    def _build_messages(self, trajectory: list, task: str) -> list:
        """Build interleaved messages from trajectory, matching /infer format."""
        system_content = f"""You control a browser. Each turn you see a screenshot.

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
- DONE - Task complete

Task: {task}"""

        messages = [{"role": "system", "content": [{"type": "text", "text": system_content}]}]

        t0 = trajectory[0]["timestamp"] if trajectory else 0

        for entry in trajectory:
            if entry["type"] == "image":
                t_rel = entry["timestamp"] - t0
                messages.append({
                    "role": "user",
                    "content": [
                        {"type": "image", "image": entry["image"]},
                        {"type": "text", "text": f"[t={t_rel:.2f}s]"},
                    ],
                })
            elif entry["type"] == "action":
                messages.append({
                    "role": "assistant",
                    "content": [{"type": "text", "text": entry["text"]}],
                })

        return messages

    def _find_last_action_mask(self, input_ids, target_action: str) -> torch.Tensor:
        """
        Create a mask that is 1 only for the LAST occurrence of target_action
        tokens in input_ids. Earlier actions in the window are context, not targets.
        """
        mask = torch.zeros_like(input_ids, dtype=torch.float32)
        tokenizer = self.processor.tokenizer

        action_ids = tokenizer.encode(target_action, add_special_tokens=False)
        action_len = len(action_ids)
        if action_len == 0:
            return mask

        # Find the LAST occurrence of this subsequence
        ids = input_ids[0].tolist()
        last_match = -1
        for i in range(len(ids) - action_len + 1):
            if ids[i:i + action_len] == action_ids:
                last_match = i

        if last_match >= 0:
            mask[0, last_match:last_match + action_len] = 1.0

        return mask

    def _unroll_trajectory(self, trajectory: list) -> list:
        """
        Split trajectory into (window, target_action) pairs.

        Returns list of (window_entries, target_action_text) tuples.
        Each window is what the model would see at inference time for
        that step, and target_action is what it should predict.
        """
        examples = []

        # Find all action indices and pair them with their preceding image
        # Trajectory alternates: img, action, img, action, ...
        # We want each action as a prediction target
        action_positions = []
        img_action_pairs = []  # list of (img_entry, action_entry) tuples

        i = 0
        while i < len(trajectory):
            if trajectory[i]["type"] == "image":
                img = trajectory[i]
                # Look for the action following this image
                if i + 1 < len(trajectory) and trajectory[i + 1]["type"] == "action":
                    img_action_pairs.append((img, trajectory[i + 1]))
                    i += 2
                else:
                    # Image with no action (last frame) - skip
                    i += 1
            else:
                i += 1

        # Build windowed examples
        for step_idx in range(len(img_action_pairs)):
            # Window: last WINDOW_SIZE pairs, but only up to current step
            window_start = max(0, step_idx + 1 - self.window_size)
            window_pairs = img_action_pairs[window_start:step_idx + 1]

            # Build trajectory entries for this window
            # All pairs except the last get both img + action (context)
            # Last pair gets img + action (action is the target)
            window_entries = []
            for img, action in window_pairs:
                window_entries.append(img)
                window_entries.append(action)

            target_action = img_action_pairs[step_idx][1]["text"]
            examples.append((window_entries, target_action))

        return examples

    def train_on_trajectory(self, trajectory: list, task: str) -> dict:
        """
        Train on a trajectory by unrolling into windowed sub-sequences.

        Each action in the trajectory becomes a training example where:
        - Context: sliding window of past images + actions
        - Target: the action at this step (loss only on this)

        Gradients accumulate across all examples, one optimizer step.

        Args:
            trajectory: list of dicts with type "image" or "action"
            task: task description

        Returns:
            Dict with loss and training stats
        """
        torch.cuda.empty_cache()
        gc.collect()
        self._ensure_optimizer()

        examples = self._unroll_trajectory(trajectory)
        if not examples:
            return {"trained": False, "reason": "no actions in trajectory"}

        self.model.train()
        self.model.gradient_checkpointing_enable()
        self.optimizer.zero_grad()

        total_loss = 0.0
        total_action_tokens = 0
        num_examples = len(examples)

        for ex_idx, (window_entries, target_action) in enumerate(examples):
            messages = self._build_messages(window_entries, task)

            inputs = self.processor.apply_chat_template(
                messages,
                tokenize=True,
                add_generation_prompt=False,
                return_dict=True,
                return_tensors="pt",
            )
            inputs.pop("token_type_ids", None)
            inputs = {k: v.to("cuda:1") if hasattr(v, "to") else v for k, v in inputs.items()}

            # Labels: only the LAST action (the prediction target)
            labels = inputs["input_ids"].clone()
            action_mask = self._find_last_action_mask(inputs["input_ids"], target_action)
            labels[action_mask == 0] = -100

            n_tokens = int(action_mask.sum().item())
            if n_tokens == 0:
                del inputs, labels, action_mask
                continue

            total_action_tokens += n_tokens

            with torch.amp.autocast("cuda", dtype=torch.bfloat16):
                outputs = self.model(**inputs, labels=labels)
                # Accumulate: divide by num_examples for averaging
                loss = outputs.loss / num_examples

            loss.backward()
            total_loss += outputs.loss.item()

            del inputs, outputs, labels, action_mask
            torch.cuda.empty_cache()

        if total_action_tokens == 0:
            return {"trained": False, "reason": "no action tokens found in any example"}

        grad_norm = torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.grad_clip)
        self.optimizer.step()

        # Back to eval mode for inference
        self.model.eval()

        avg_loss = total_loss / num_examples
        self.train_steps += 1
        self.total_loss += avg_loss

        result = {
            "trained": True,
            "loss": avg_loss,
            "grad_norm": grad_norm.item() if hasattr(grad_norm, "item") else float(grad_norm),
            "action_tokens": total_action_tokens,
            "num_examples": num_examples,
            "trajectory_len": len(trajectory),
            "num_actions": num_examples,
            "train_steps": self.train_steps,
            "avg_loss": self.total_loss / self.train_steps,
            "window_size": self.window_size,
            "grad_clip": self.grad_clip,
        }
        self._log_metrics("trajectory_train", result)
        return result

    def save_checkpoint(self, path: str):
        """Save LoRA weights + optimizer state."""
        from pathlib import Path
        save_dir = Path(path)
        save_dir.mkdir(parents=True, exist_ok=True)

        self.model.save_pretrained(str(save_dir / "adapter"))

        torch.save({
            "optimizer_state_dict": self.optimizer.state_dict() if self.optimizer else None,
            "train_steps": self.train_steps,
            "total_loss": self.total_loss,
        }, str(save_dir / "trainer_state.pt"))

    def load_checkpoint(self, path: str):
        """Load optimizer state (model weights loaded separately)."""
        from pathlib import Path
        state_path = Path(path) / "trainer_state.pt"
        if state_path.exists():
            state = torch.load(str(state_path), weights_only=True)
            self._ensure_optimizer()
            if state.get("optimizer_state_dict"):
            self.optimizer.load_state_dict(state["optimizer_state_dict"])
            self.train_steps = state.get("train_steps", 0)
            self.total_loss = state.get("total_loss", 0.0)

    def _log_metrics(self, event: str, payload: dict):
        if self.metrics_logger:
            self.metrics_logger.log(event, payload)


@dataclass
class Correction:
    """A correction from the oracle."""
    screenshot: Image.Image
    task: str
    model_output: str  # What model said (wrong)
    corrected_output: str  # What it should have said
    reward: float = 1.0  # Positive reward for correction


class TrainingInjector:
    """
    Injects oracle corrections as training signals.

    Uses the model's own forward pass to compute logprobs of the
    corrected output, then backprops with positive reward.
    """

    def __init__(self, model, tokenizer, lr: float = 2e-4):
        self.model = model
        self.tokenizer = tokenizer
        self.optimizer = None
        self.lr = lr

        # Stats
        self.injections = 0
        self.total_loss = 0.0

    def _ensure_optimizer(self):
        """Lazy init optimizer."""
        if self.optimizer is None:
            self.optimizer = torch.optim.AdamW(
                self.model.parameters(),
                lr=self.lr,
                weight_decay=0.01,
            )

    def _build_prompt(self, task: str) -> str:
        """Build the system prompt for the model."""
        system_prompt = """You are a computer use agent. You see a screenshot.

Actions:
- CLICK x y - Click at pixel coordinates (1280x704 resolution)
- TYPE text - Type text
- KEY keyname - Press key (enter, tab, escape, etc)
- SCROLL dy - Scroll (positive=up, negative=down)
- WAIT - Wait for animation/loading
- DONE - Task complete

Output exactly one action, nothing else."""

        return f"{system_prompt}\n\nTask: {task}"

    def _compute_logprobs(self, inputs, labels) -> dict:
        """Compute logprobs for a sequence without updating weights."""
        import torch.nn.functional as F

        with torch.no_grad(), torch.amp.autocast('cuda', dtype=torch.bfloat16):
            outputs = self.model(**inputs)
            logits = outputs.logits

            # Shift for next-token prediction
            shift_logits = logits[..., :-1, :].contiguous()
            shift_labels = labels[..., 1:].contiguous()

            # Compute log probs
            log_probs = F.log_softmax(shift_logits, dim=-1)

            # Get log prob of actual tokens
            token_log_probs = log_probs.gather(
                dim=-1,
                index=shift_labels.unsqueeze(-1)
            ).squeeze(-1)

            # Mask out padding (assuming pad_token_id exists)
            mask = shift_labels != self.tokenizer.pad_token_id if self.tokenizer.pad_token_id else torch.ones_like(shift_labels)

            # Stats
            total_log_prob = (token_log_probs * mask).sum().item()
            num_tokens = mask.sum().item()
            avg_log_prob = total_log_prob / max(num_tokens, 1)
            perplexity = torch.exp(-torch.tensor(avg_log_prob)).item()

            return {
                "total_log_prob": total_log_prob,
                "avg_log_prob": avg_log_prob,
                "num_tokens": int(num_tokens),
                "perplexity": perplexity,
            }

    def inject_fast(self, correction: Correction) -> dict:
        """
        Fast injection - just train on correct, skip logprob tracking.
        Uses less memory than full inject().
        """
        import gc
        torch.cuda.empty_cache()
        gc.collect()

        self._ensure_optimizer()

        prompt_text = self._build_prompt(correction.task)
        messages = [
            {"role": "user", "content": [
                {"type": "image"},
                {"type": "text", "text": prompt_text},
            ]},
            {"role": "assistant", "content": correction.corrected_output}
        ]

        full_text = self.tokenizer.apply_chat_template(
            messages, tokenize=False, add_generation_prompt=False
        )

        inputs = self.tokenizer(
            correction.screenshot, full_text,
            add_special_tokens=False, return_tensors="pt"
        ).to("cuda:1")

        if "pixel_values" in inputs:
            inputs["pixel_values"] = inputs["pixel_values"].to(torch.bfloat16)

        labels = inputs["input_ids"].clone()

        # Single training pass
        self.model.train()
        self.optimizer.zero_grad()

        with torch.amp.autocast('cuda', dtype=torch.bfloat16):
            outputs = self.model(**inputs, labels=labels)
            loss = outputs.loss * 10.0  # strong positive weight

        loss.backward()
        torch.nn.utils.clip_grad_norm_(self.model.parameters(), 1.0)
        self.optimizer.step()

        self.injections += 1
        self.total_loss += loss.item()

        # Clear memory
        del inputs, outputs, labels
        torch.cuda.empty_cache()

        return {
            "injected": True,
            "loss": loss.item(),
            "injections": self.injections,
        }

    def inject(self, correction: Correction) -> dict:
        """
        Inject a single correction as a training signal.

        Args:
            correction: The correction to inject

        Returns:
            Dict with loss, logprobs before/after, and stats
        """
        self._ensure_optimizer()

        # Build the full prompt + corrected response
        prompt_text = self._build_prompt(correction.task)

        # Format as chat messages with the corrected output as assistant response
        messages = [
            {"role": "user", "content": [
                {"type": "image"},
                {"type": "text", "text": prompt_text},
            ]},
            {"role": "assistant", "content": correction.corrected_output}
        ]

        # Apply chat template (includes the assistant's response)
        full_text = self.tokenizer.apply_chat_template(
            messages,
            tokenize=False,
            add_generation_prompt=False,
        )

        # Tokenize with image
        inputs = self.tokenizer(
            correction.screenshot,
            full_text,
            add_special_tokens=False,
            return_tensors="pt",
        ).to("cuda:1")

        # Ensure correct dtype for model (bfloat16)
        if "pixel_values" in inputs:
            inputs["pixel_values"] = inputs["pixel_values"].to(torch.bfloat16)

        labels = inputs["input_ids"].clone()

        # === BEFORE TRAINING: compute logprobs ===
        self.model.eval()
        logprobs_before = self._compute_logprobs(inputs, labels)

        # === TRAINING: forward + backward ===
        self.model.train()
        self.optimizer.zero_grad()

        # Use autocast to handle mixed precision
        with torch.amp.autocast('cuda', dtype=torch.bfloat16):
            # 1. Loss on CORRECTED output (pull toward correct) - aggressive
            outputs = self.model(**inputs, labels=labels)
            loss_correct = outputs.loss * 10.0  # strong positive weight

            # 2. Loss on WRONG output (push away from wrong) - if provided
            loss_wrong = torch.tensor(0.0, device="cuda")
            if correction.model_output and correction.model_output != correction.corrected_output:
                # Tokenize the wrong output
                wrong_messages = [
                    {"role": "user", "content": [
                        {"type": "image"},
                        {"type": "text", "text": self._build_prompt(correction.task)},
                    ]},
                    {"role": "assistant", "content": correction.model_output}
                ]
                wrong_text = self.tokenizer.apply_chat_template(
                    wrong_messages, tokenize=False, add_generation_prompt=False
                )
                wrong_inputs = self.tokenizer(
                    correction.screenshot, wrong_text,
                    add_special_tokens=False, return_tensors="pt"
                ).to("cuda:1")
                wrong_labels = wrong_inputs["input_ids"].clone()

                wrong_outputs = self.model(**wrong_inputs, labels=wrong_labels)
                # Negative weight = push away from this output (aggressive)
                loss_wrong = wrong_outputs.loss * -10.0

            loss = loss_correct + loss_wrong

        # Backward pass
        loss.backward()

        # Gradient clipping
        total_norm = torch.nn.utils.clip_grad_norm_(self.model.parameters(), 1.0)

        # Check gradient stats
        grad_stats = {
            "grad_norm": total_norm.item() if hasattr(total_norm, 'item') else float(total_norm),
            "num_params_with_grad": sum(1 for p in self.model.parameters() if p.grad is not None),
            "num_trainable_params": sum(1 for p in self.model.parameters() if p.requires_grad),
        }

        # Update weights
        self.optimizer.step()

        # === AFTER TRAINING: compute logprobs ===
        self.model.eval()
        logprobs_after = self._compute_logprobs(inputs, labels)

        # Stats
        self.injections += 1
        self.total_loss += loss.item()

        # Log the change
        log_prob_delta = logprobs_after["avg_log_prob"] - logprobs_before["avg_log_prob"]
        perplexity_delta = logprobs_after["perplexity"] - logprobs_before["perplexity"]

        return {
            "loss": loss.item(),
            "loss_correct": loss_correct.item() if hasattr(loss_correct, 'item') else float(loss_correct),
            "loss_wrong": loss_wrong.item() if hasattr(loss_wrong, 'item') else float(loss_wrong),
            "reward": correction.reward,
            "grad_stats": grad_stats,
            "logprobs_before": logprobs_before,
            "logprobs_after": logprobs_after,
            "log_prob_delta": log_prob_delta,
            "perplexity_delta": perplexity_delta,
            "injections": self.injections,
            "avg_loss": self.total_loss / self.injections,
        }

    def inject_batch(self, corrections: list[Correction]) -> dict:
        """
        Inject a batch of corrections with accumulated gradients.

        Args:
            corrections: List of corrections

        Returns:
            Dict with batch stats
        """
        self._ensure_optimizer()
        self.model.train()
        self.optimizer.zero_grad()

        batch_loss = 0.0

        for correction in corrections:
            prompt_text = self._build_prompt(correction.task)

            messages = [
                {"role": "user", "content": [
                    {"type": "image"},
                    {"type": "text", "text": prompt_text},
                ]},
                {"role": "assistant", "content": correction.corrected_output}
            ]

            full_text = self.tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=False,
            )

            inputs = self.tokenizer(
                correction.screenshot,
                full_text,
                add_special_tokens=False,
                return_tensors="pt",
            ).to("cuda:1")

            outputs = self.model(
                **inputs,
                labels=inputs["input_ids"],
            )

            # Accumulate gradients (divide by batch size for averaging)
            loss = (outputs.loss * correction.reward) / len(corrections)
            loss.backward()

            batch_loss += loss.item() * len(corrections)  # Undo the division for logging

        # Clip and update
        torch.nn.utils.clip_grad_norm_(self.model.parameters(), 1.0)
        self.optimizer.step()

        self.injections += len(corrections)
        self.total_loss += batch_loss

        return {
            "batch_loss": batch_loss,
            "batch_size": len(corrections),
            "injections": self.injections,
            "avg_loss": self.total_loss / self.injections,
        }


def create_correction(
    screenshot: Image.Image,
    task: str,
    model_output: str,
    target_element: dict,
    model_coords: tuple[int, int],
    correct_coords: tuple[int, int],
) -> Correction:
    """
    Helper to create a Correction from oracle data.

    Args:
        screenshot: The screenshot the model saw
        task: The task description
        model_output: What model said (e.g., "CLICK 499 718")
        target_element: Element info from oracle (tag, text, bbox)
        model_coords: Where model clicked
        correct_coords: Where it should have clicked

    Returns:
        Correction object ready for injection
    """
    tag = target_element.get("tag", "element")
    text = target_element.get("text", "")[:30]
    correct_x, correct_y = correct_coords

    # Generate corrected reasoning
    corrected_output = f"""I need to {task.lower().rstrip('.')}.
Looking at the screenshot, I can see a {tag} with text "{text}".
The element is located at approximately x={correct_x}, y={correct_y}.
CLICK {correct_x} {correct_y}"""

    return Correction(
        screenshot=screenshot,
        task=task,
        model_output=model_output,
        corrected_output=corrected_output,
        reward=1.0,
    )
