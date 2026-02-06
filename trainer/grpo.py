"""
GRPO (Group Relative Policy Optimization) trainer with configurable
hyperparameters and structured metrics logging.
"""
import gc
import math
from typing import List, Tuple, Optional

import torch

from trainer.utils import MetricsLogger


class GRPOTrainer:

    def __init__(
        self,
        model,
        processor,
        lr: float = 2e-5,
        advantage_threshold: float = 0.01,
        min_advantage_std: float = 1e-8,
        grad_clip: float = 1.0,
        val_fraction: float = 0.2,
        metrics_logger: Optional[MetricsLogger] = None,
    ):
        self.model = model
        self.processor = processor
        self.optimizer = None
        self.lr = lr
        self.train_steps = 0
        self.advantage_threshold = advantage_threshold
        self.min_advantage_std = min_advantage_std
        self.grad_clip = grad_clip
        self.val_fraction = min(max(val_fraction, 0.0), 0.9)
        self.metrics_logger = metrics_logger or MetricsLogger()

    def _ensure_optimizer(self):
        if self.optimizer is None:
            self.optimizer = torch.optim.AdamW(
                [p for p in self.model.parameters() if p.requires_grad],
                lr=self.lr,
                weight_decay=0.01,
            )

    def _build_messages(self, trajectory: list, task: str) -> list:
        """Build chat messages from trajectory entries (processor format)."""
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

    def _find_assistant_token_mask(self, input_ids: torch.Tensor) -> torch.Tensor:
        """
        Mask for ALL assistant response tokens in the sequence.

        Finds <|im_start|>assistant\n...content...<|im_end|> spans
        and marks the content tokens as targets.
        """
        tokenizer = self.processor.tokenizer
        mask = torch.zeros_like(input_ids, dtype=torch.float32)

        ids = input_ids[0].tolist()

        im_start_id = tokenizer.convert_tokens_to_ids("<|im_start|>")
        im_end_id = tokenizer.convert_tokens_to_ids("<|im_end|>")
        assistant_prefix = tokenizer.encode("assistant\n", add_special_tokens=False)
        prefix_len = len(assistant_prefix)

        i = 0
        while i < len(ids):
            if ids[i] == im_start_id:
                next_tokens = ids[i + 1:i + 1 + prefix_len]
                if next_tokens == assistant_prefix:
                    content_start = i + 1 + prefix_len
                    j = content_start
                    while j < len(ids) and ids[j] != im_end_id:
                        j += 1
                    mask[0, content_start:j] = 1.0
                    i = j + 1
                    continue
            i += 1

        return mask

    def compute_advantages(self, rewards: list) -> list:
        """Group-relative advantages."""
        if not rewards:
            return []

        rewards_t = torch.tensor(rewards, dtype=torch.float32)
        mean = rewards_t.mean()
        std = rewards_t.std()
        if std < self.min_advantage_std:
            return [0.0] * len(rewards)
        return ((rewards_t - mean) / (std + 1e-8)).tolist()

    def _split_rollouts(self, rollouts: list) -> Tuple[List[dict], List[dict]]:
        if self.val_fraction <= 0 or len(rollouts) < 2:
            return rollouts, []

        val_count = max(1, int(len(rollouts) * self.val_fraction))
        if val_count >= len(rollouts):
            return rollouts, []

        return rollouts[:-val_count], rollouts[-val_count:]

    def _evaluate_rollouts(self, rollouts: list, task: str) -> dict:
        if not rollouts:
            return {}

        self.model.eval()
        losses = []
        total_tokens = 0

        for rollout in rollouts:
            trajectory = rollout["trajectory"]
            if not trajectory:
                continue

            messages = self._build_messages(trajectory, task)
            try:
                inputs = self.processor.apply_chat_template(
                    messages,
                    tokenize=True,
                    add_generation_prompt=False,
                    return_dict=True,
                    return_tensors="pt",
                )
            except Exception as e:
                print(f"[grpo] Validation tokenization error: {e}", flush=True)
                continue

            inputs.pop("token_type_ids", None)
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) if hasattr(v, "to") else v for k, v in inputs.items()}

            labels = inputs["input_ids"].clone()
            mask = self._find_assistant_token_mask(inputs["input_ids"])
            labels[mask == 0] = -100

            n_tokens = int(mask.sum().item())
            if n_tokens == 0:
                del inputs, labels, mask
                continue

            with torch.no_grad(), torch.amp.autocast("cuda", dtype=torch.bfloat16):
                outputs = self.model(**inputs, labels=labels)
                losses.append(outputs.loss.item())
                total_tokens += n_tokens

            del inputs, labels, mask, outputs

        if not losses or total_tokens == 0:
            return {}

        avg_loss = sum(losses) / len(losses)
        perplexity = math.exp(avg_loss)
        return {
            "val_loss": avg_loss,
            "val_perplexity": perplexity,
            "val_batches": len(losses),
            "val_tokens": total_tokens,
        }

    def train_step(self, rollouts: list, task: str) -> dict:
        """
        GRPO training step on a group of rollouts.

        Args:
            rollouts: list of {"trajectory": [...], "reward": float}
            task: task description

        Returns:
            Training stats dict
        """
        torch.cuda.empty_cache()
        gc.collect()
        self._ensure_optimizer()

        if not rollouts:
            return {"trained": False, "reason": "no rollouts provided"}

        rewards = [r["reward"] for r in rollouts]
        reward_tensor = torch.tensor(rewards, dtype=torch.float32)
        reward_mean = float(reward_tensor.mean().item())
        reward_std = float(reward_tensor.std(unbiased=False).item()) if len(rewards) > 1 else 0.0
        reward_variance = reward_std ** 2

        train_rollouts, val_rollouts = self._split_rollouts(rollouts)
        train_rewards = [r["reward"] for r in train_rollouts]
        advantages = self.compute_advantages(train_rewards)

        if not advantages or all(abs(a) < self.advantage_threshold for a in advantages):
            return {
                "trained": False,
                "reason": "no variance in rewards",
                "rewards": rewards,
                "mean_reward": reward_mean,
                "reward_variance": reward_variance,
            }

        self.model.train()
        self.model.gradient_checkpointing_enable()
        self.optimizer.zero_grad()

        total_loss = 0.0
        total_action_tokens = 0
        n = max(len(train_rollouts), 1)
        trained_count = 0
        skipped = 0

        for rollout, advantage in zip(train_rollouts, advantages):
            if abs(advantage) < self.advantage_threshold:
                skipped += 1
                continue

            trajectory = rollout["trajectory"]
            if not trajectory:
                skipped += 1
                continue

            messages = self._build_messages(trajectory, task)

            try:
                inputs = self.processor.apply_chat_template(
                    messages,
                    tokenize=True,
                    add_generation_prompt=False,
                    return_dict=True,
                    return_tensors="pt",
                )
            except Exception as e:
                print(f"[grpo] Tokenization error: {e}", flush=True)
                skipped += 1
                continue

            inputs.pop("token_type_ids", None)
            device = next(self.model.parameters()).device
            inputs = {k: v.to(device) if hasattr(v, "to") else v for k, v in inputs.items()}

            labels = inputs["input_ids"].clone()
            action_mask = self._find_assistant_token_mask(inputs["input_ids"])
            labels[action_mask == 0] = -100

            n_tokens = int(action_mask.sum().item())
            if n_tokens == 0:
                skipped += 1
                del inputs, labels, action_mask
                continue

            total_action_tokens += n_tokens

            with torch.amp.autocast("cuda", dtype=torch.bfloat16):
                outputs = self.model(**inputs, labels=labels)
                loss = advantage * outputs.loss / n

            loss.backward()
            total_loss += advantage * outputs.loss.item()
            trained_count += 1

            del inputs, outputs, labels, action_mask
            torch.cuda.empty_cache()

        if trained_count == 0:
            self.model.eval()
            return {
                "trained": False,
                "reason": "no valid rollouts after filtering",
                "rewards": rewards,
                "mean_reward": reward_mean,
                "reward_variance": reward_variance,
            }

        grad_norm = torch.nn.utils.clip_grad_norm_(self.model.parameters(), self.grad_clip)
        self.optimizer.step()
        self.model.eval()

        avg_loss = total_loss / max(trained_count, 1)
        self.train_steps += 1

        val_metrics = self._evaluate_rollouts(val_rollouts, task)

        result = {
            "trained": True,
            "loss": avg_loss,
            "grad_norm": grad_norm.item() if hasattr(grad_norm, "item") else float(grad_norm),
            "action_tokens": total_action_tokens,
            "num_rollouts": len(rollouts),
            "train_rollouts": len(train_rollouts),
            "skipped_rollouts": skipped,
            "advantages_used": trained_count,
            "rewards": rewards,
            "reward_mean": reward_mean,
            "reward_std": reward_std,
            "reward_variance": reward_variance,
            "advantage_threshold": self.advantage_threshold,
            "val_fraction": self.val_fraction,
            "train_steps": self.train_steps,
        }

        if val_metrics:
            result["validation"] = val_metrics

        self._log_metrics("grpo_train", result)
        return result

    def _log_metrics(self, event: str, payload: dict):
        if self.metrics_logger:
            self.metrics_logger.log(event, payload)
