#!/usr/bin/env python3
"""
Control server for VL-Computer-Use training.

Human-in-the-loop: every action is explicit, nothing runs automatically.
Actor and Oracle are separate - this just handles environment control.

Endpoints:
    GET  /                  → status and available endpoints
    GET  /screenshot        → take screenshot, return path
    GET  /dom               → get DOM state and clickable elements
    POST /execute           → execute action {"action": "CLICK 640 500"}
    POST /check             → check if coords hit target {"x": 640, "y": 500, "target": "button"}
    GET  /coords            → get coordinate conversion info
    POST /convert           → convert coords {"from": "model", "to": "screen", "x": 640, "y": 500}
    POST /inject            → inject training correction
    POST /reload            → reload to start page
    GET  /state             → current training state/stats

Run: uv run python server.py
"""
import os
import sys
import json
import glob
import time
import base64
import io
import asyncio
import subprocess
from pathlib import Path
from datetime import datetime
from dataclasses import dataclass, asdict
from typing import Optional

from aiohttp import web
from PIL import Image
from trainer.utils import MetricsLogger

# Will be set up on startup
PLAYWRIGHT = None
BROWSER = None
PAGE = None
MODEL = None
TOKENIZER = None
PROCESSOR = None
INJECTOR = None
TRAINER = None  # TrajectoryTrainer instance
GRPO_TRAINER = None  # GRPOTrainer instance

# Stored rollouts for GRPO training
ROLLOUTS = []  # List of {"trajectory": [...], "reward": float}

# Trajectory state - rolling observation-action sequence
TRAJECTORY = []  # List of {"type": "image"|"action", "image": PIL, "text": str, "timestamp": float}

# Oracle config (GPT-OSS via vLLM)
ORACLE_URL = "http://localhost:8001/v1/chat/completions"
ORACLE_MODEL = "openai/gpt-oss-120b"
HTTP_CLIENT = None

# vLLM inference server (serves frozen base model for fast inference)
# Start with: vllm serve Qwen/Qwen3-VL-8B-Instruct --port 8002 --dtype bfloat16 --limit-mm-per-prompt image=8
VLLM_INFER_URL = "http://localhost:8002/v1"
VLLM_INFER_MODEL = os.environ.get("VLLM_MODEL", "Qwen/Qwen3-VL-8B-Instruct")
VLLM_CLIENT = None

# DOM Timeline - tracks element changes with timestamps
DOM_TIMELINE = []  # List of {timestamp, elements, url}
DOM_POLL_TASK = None
MAX_TIMELINE_ENTRIES = 100  # Keep last N snapshots

# State
STATE = {
    "screenshots_taken": 0,
    "actions_executed": 0,
    "corrections_injected": 0,
    "last_action": None,
    "last_screenshot": None,
    "model_loaded": False,
    "started_at": None,
}


def _env_number(name: str, default, cast):
    value = os.environ.get(name)
    if value is None:
        return default
    try:
        return cast(value)
    except (TypeError, ValueError):
        print(f"[config] Invalid value for {name}={value!r}, using default {default}", flush=True)
        return default


METRICS_LOGGER = MetricsLogger(os.environ.get("TRAINING_METRICS_FILE"))
TRAJECTORY_CONFIG = {
    "window_size": int(_env_number("TRAJECTORY_WINDOW_SIZE", 8, int)),
    "grad_clip": float(_env_number("TRAJECTORY_GRAD_CLIP", 1.0, float)),
}
GRPO_CONFIG = {
    "advantage_threshold": float(_env_number("GRPO_ADV_THRESHOLD", 0.01, float)),
    "val_fraction": float(_env_number("GRPO_VAL_FRACTION", 0.2, float)),
    "grad_clip": float(_env_number("GRPO_GRAD_CLIP", 1.0, float)),
}


async def poll_dom():
    """Background task that polls DOM and tracks changes + animations."""
    global DOM_TIMELINE
    last_positions = {}  # Track element positions for movement detection

    while True:
        try:
            if PAGE:
                elements = await PAGE.evaluate('''
                    () => {
                        const selectors = 'a, button, input, [role="button"], [class*="popup"], [class*="modal"], [class*="dialog"], [class*="animate"], [class*="loading"]';
                        return Array.from(document.querySelectorAll(selectors)).map(el => {
                            const rect = el.getBoundingClientRect();
                            if (rect.width === 0 || rect.height === 0) return null;

                            const style = getComputedStyle(el);

                            // Check for CSS animations
                            const hasAnimation = style.animationName !== 'none' && style.animationName !== '';
                            const hasTransition = style.transitionDuration !== '0s' && style.transitionProperty !== 'none';

                            // Check for animation-related classes
                            const classList = [...el.classList];
                            const hasAnimateClass = classList.some(c =>
                                c.includes('animate') || c.includes('loading') ||
                                c.includes('spin') || c.includes('fade') ||
                                c.includes('slide') || c.includes('pulse')
                            );

                            // Check opacity (fading elements)
                            const opacity = parseFloat(style.opacity);
                            const isFading = opacity > 0 && opacity < 1;

                            // Check transform (might be animating)
                            const hasTransform = style.transform !== 'none' && style.transform !== '';

                            const isAnimating = hasAnimation || hasTransition || hasAnimateClass || isFading;

                            return {
                                tag: el.tagName.toLowerCase(),
                                text: (el.innerText || el.value || '').slice(0, 50).trim(),
                                id: el.id || null,
                                classes: classList.slice(0, 5),
                                x: Math.round(rect.x),
                                y: Math.round(rect.y),
                                width: Math.round(rect.width),
                                height: Math.round(rect.height),
                                animating: isAnimating,
                                animationDetails: {
                                    cssAnimation: hasAnimation,
                                    cssTransition: hasTransition,
                                    animateClass: hasAnimateClass,
                                    fading: isFading,
                                    hasTransform: hasTransform,
                                },
                                opacity: opacity,
                            };
                        }).filter(Boolean);
                    }
                ''')

                # Detect position changes (movement) by comparing to last poll
                current_time = time.time()
                for el in elements:
                    el_id = f"{el['tag']}:{el.get('id') or el.get('text','')[:20]}"
                    current_pos = (el['x'], el['y'])

                    if el_id in last_positions:
                        last_pos, last_time = last_positions[el_id]
                        dx = abs(current_pos[0] - last_pos[0])
                        dy = abs(current_pos[1] - last_pos[1])
                        dt = current_time - last_time

                        # If moved more than 5px in last poll interval, it's moving
                        if dx > 5 or dy > 5:
                            el['animating'] = True
                            el['moving'] = True
                            el['velocity'] = {'dx': dx/dt if dt > 0 else 0, 'dy': dy/dt if dt > 0 else 0}

                    last_positions[el_id] = (current_pos, current_time)

                # Count animating elements
                animating_elements = [e for e in elements if e.get('animating')]
                moving_elements = [e for e in elements if e.get('moving')]

                snapshot = {
                    "timestamp": time.time(),
                    "url": PAGE.url,
                    "elements": elements,
                    "element_ids": set(f"{e['tag']}:{e.get('text','')[:20]}" for e in elements),
                    "animations": {
                        "count": len(animating_elements),
                        "moving_count": len(moving_elements),
                        "elements": [
                            {"tag": e['tag'], "text": e.get('text','')[:30], "details": e.get('animationDetails')}
                            for e in animating_elements[:5]  # Limit to 5
                        ],
                    },
                    "has_animations": len(animating_elements) > 0,
                }

                DOM_TIMELINE.append(snapshot)

                # Trim old entries
                if len(DOM_TIMELINE) > MAX_TIMELINE_ENTRIES:
                    DOM_TIMELINE = DOM_TIMELINE[-MAX_TIMELINE_ENTRIES:]

        except Exception as e:
            pass  # Silently ignore polling errors

        await asyncio.sleep(0.1)  # Poll every 100ms


def get_dom_at_time(timestamp: float) -> dict:
    """Get the DOM snapshot closest to (but not after) a given timestamp."""
    if not DOM_TIMELINE:
        return None

    # Find the snapshot closest to but not after the timestamp
    best = None
    for snapshot in DOM_TIMELINE:
        if snapshot["timestamp"] <= timestamp:
            best = snapshot
        else:
            break
    return best


def get_new_elements_since(timestamp: float) -> list:
    """Get elements that appeared AFTER the given timestamp."""
    dom_at_time = get_dom_at_time(timestamp)
    if not dom_at_time or not DOM_TIMELINE:
        return []

    old_ids = dom_at_time.get("element_ids", set())
    current = DOM_TIMELINE[-1]
    current_ids = current.get("element_ids", set())

    new_ids = current_ids - old_ids

    # Return the actual elements
    new_elements = []
    for el in current.get("elements", []):
        el_id = f"{el['tag']}:{el.get('text','')[:20]}"
        if el_id in new_ids:
            new_elements.append(el)

    return new_elements


def setup_wayland():
    """Setup Wayland environment variables."""
    xdg_runtime = os.environ.get("XDG_RUNTIME_DIR", f"/run/user/{os.getuid()}")
    sockets = sorted(glob.glob(f"{xdg_runtime}/sway-ipc.*.sock"))
    if sockets:
        os.environ["SWAYSOCK"] = sockets[0]
    for display in ["wayland-1", "wayland-0"]:
        if os.path.exists(f"{xdg_runtime}/{display}"):
            os.environ["WAYLAND_DISPLAY"] = display
            break
    print(f"[Env] WAYLAND_DISPLAY={os.environ.get('WAYLAND_DISPLAY')}")
    print(f"[Env] SWAYSOCK={os.environ.get('SWAYSOCK')}")


async def setup_browser():
    """Connect to Chrome via CDP."""
    global PLAYWRIGHT, BROWSER, PAGE
    from playwright.async_api import async_playwright

    PLAYWRIGHT = await async_playwright().start()
    BROWSER = await PLAYWRIGHT.chromium.connect_over_cdp("http://127.0.0.1:9222")
    PAGE = BROWSER.contexts[0].pages[0]
    print(f"[Browser] Connected to {PAGE.url}")


# --- Screenshot ---

async def handle_screenshot(request):
    """Take screenshot, return path and metadata."""
    screenshots_dir = Path("/tmp/vl-screenshots")
    screenshots_dir.mkdir(exist_ok=True)

    STATE["screenshots_taken"] += 1
    timestamp = int(time.time() * 1000)
    path = screenshots_dir / f"screenshot_{timestamp}.png"

    # Take screenshot via grim
    result = subprocess.run(
        ["grim", "-c", str(path)],
        capture_output=True,
    )

    if result.returncode != 0:
        return web.json_response({
            "error": f"grim failed: {result.stderr.decode()}"
        }, status=500)

    # Get image size
    img = Image.open(path)
    width, height = img.size

    STATE["last_screenshot"] = str(path)

    return web.json_response({
        "path": str(path),
        "size": {"width": width, "height": height},
        "model_size": {"width": 1280, "height": 704},
        "timestamp": timestamp,
    })


# --- DOM ---

async def handle_dom(request):
    """Get DOM state and clickable elements."""
    # Get coordinate info for conversion
    info = await PAGE.evaluate('''
        () => ({
            chrome_height: window.outerHeight - window.innerHeight,
            screen_height: screen.height,
            screen_width: screen.width
        })
    ''')

    # Get page info
    url = PAGE.url
    text = await PAGE.evaluate("document.body.innerText")

    # Get all clickable elements with model coords
    elements = await PAGE.evaluate('''
        (info) => {
            const selectors = 'a, button, input, select, textarea, [onclick], [role="button"], [role="link"]';
            return Array.from(document.querySelectorAll(selectors)).map(el => {
                const rect = el.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return null;

                const dom_cx = rect.x + rect.width / 2;
                const dom_cy = rect.y + rect.height / 2;

                // Convert to screen then pixel coords (1280x704)
                const screen_x = dom_cx;
                const screen_y = dom_cy + info.chrome_height;
                const pixel_x = Math.round(screen_x);
                const pixel_y = Math.round(screen_y * (704 / info.screen_height));

                // Convert to normalized 0-1000 (what model uses)
                const norm_x = Math.round(pixel_x / 1280 * 1000);
                const norm_y = Math.round(pixel_y / 704 * 1000);

                return {
                    tag: el.tagName.toLowerCase(),
                    text: (el.innerText || el.value || el.placeholder || '').slice(0, 50).trim(),
                    id: el.id || null,
                    type: el.type || null,
                    classes: [...el.classList].slice(0, 5),
                    coords: {
                        dom: { x: Math.round(dom_cx), y: Math.round(dom_cy) },
                        screen: { x: Math.round(screen_x), y: Math.round(screen_y) },
                        pixel: { x: pixel_x, y: pixel_y },
                        normalized: { x: norm_x, y: norm_y }
                    },
                    size: { width: Math.round(rect.width), height: Math.round(rect.height) }
                };
            }).filter(Boolean);
        }
    ''', info)

    return web.json_response({
        "url": url,
        "text": text[:1000],
        "elements": elements,
        "coordinate_info": info,
    })


# --- Execute Action ---

async def handle_execute(request):
    """Execute an action (CLICK, TYPE, KEY, SCROLL, WAIT)."""
    data = await request.json()
    action = data.get("action", "").strip()

    if not action:
        return web.json_response({"error": "No action provided"}, status=400)

    socket = os.environ.get("SWAYSOCK")
    action_upper = action.upper()
    result = {"action": action, "executed": False, "details": {}}

    # Image dimensions for coordinate conversion
    MODEL_WIDTH = 1280
    MODEL_HEIGHT = 704
    SCREEN_HEIGHT = 720

    try:
        if action_upper.startswith("CLICK"):
            parts = action_upper.split()
            if len(parts) >= 3:
                norm_x, norm_y = int(parts[1]), int(parts[2])

                # Convert from normalized 0-1000 to pixel coordinates
                pixel_x = int(norm_x / 1000 * MODEL_WIDTH)
                pixel_y = int(norm_y / 1000 * MODEL_HEIGHT)

                # Then convert from model coords (704 height) to screen coords (720 height)
                screen_x = pixel_x
                screen_y = int(pixel_y * (SCREEN_HEIGHT / MODEL_HEIGHT))

                result["details"] = {
                    "normalized_0_1000": {"x": norm_x, "y": norm_y},
                    "pixel_coords": {"x": pixel_x, "y": pixel_y},
                    "screen_coords": {"x": screen_x, "y": screen_y},
                }

                # Move cursor
                subprocess.run([
                    "swaymsg", "-s", socket,
                    "seat", "-", "cursor", "set", str(screen_x), str(screen_y)
                ], capture_output=True)
                time.sleep(0.05)

                # Click
                subprocess.run(["ydotool", "click", "0xC0"])
                result["executed"] = True

        elif action_upper.startswith("TYPE"):
            text = action[5:].strip()
            result["details"] = {"text": text}
            subprocess.run(["ydotool", "type", "--key-delay", "20", text])
            result["executed"] = True

        elif action_upper.startswith("KEY"):
            key = action[4:].strip().lower()
            key_map = {
                "enter": "28", "return": "28", "tab": "15",
                "escape": "1", "esc": "1", "backspace": "14",
                "space": "57", "up": "103", "down": "108",
                "left": "105", "right": "106",
            }
            keycode = key_map.get(key, key)
            result["details"] = {"key": key, "keycode": keycode}
            subprocess.run(["ydotool", "key", f"{keycode}:1", f"{keycode}:0"])
            result["executed"] = True

        elif action_upper.startswith("SCROLL"):
            parts = action_upper.split()
            if len(parts) >= 2:
                dy = int(parts[1])
                direction = "up" if dy > 0 else "down"
                result["details"] = {"direction": direction, "amount": abs(dy)}
                for _ in range(abs(dy)):
                    subprocess.run(["ydotool", "mousemove", "--wheel", direction])
                    time.sleep(0.02)
                result["executed"] = True

        elif action_upper == "WAIT":
            result["details"] = {"duration": 1.0}
            time.sleep(1.0)
            result["executed"] = True

        else:
            result["error"] = f"Unknown action: {action}"

    except Exception as e:
        result["error"] = str(e)

    if result["executed"]:
        STATE["actions_executed"] += 1
        STATE["last_action"] = action
        time.sleep(0.2)  # Wait for UI to update

    return web.json_response(result)


# --- Check Click ---

async def handle_check(request):
    """Check if coordinates hit a target element."""
    data = await request.json()
    norm_x = data.get("x")
    norm_y = data.get("y")
    target_text = data.get("target")  # Optional: text to match

    if norm_x is None or norm_y is None:
        return web.json_response({"error": "x and y required"}, status=400)

    # Get coordinate info
    info = await PAGE.evaluate('''
        () => ({
            chrome_height: window.outerHeight - window.innerHeight,
            screen_height: screen.height
        })
    ''')

    # Convert from normalized 0-1000 to pixel coords
    MODEL_WIDTH = 1280
    MODEL_HEIGHT = 704
    pixel_x = int(norm_x / 1000 * MODEL_WIDTH)
    pixel_y = int(norm_y / 1000 * MODEL_HEIGHT)

    # Convert to screen then DOM coords
    screen_y = int(pixel_y * (info['screen_height'] / MODEL_HEIGHT))
    dom_x = pixel_x
    dom_y = screen_y - info['chrome_height']

    # Check what element is at those coords
    hit = await PAGE.evaluate('''
        (coords) => {
            const el = document.elementFromPoint(coords.x, coords.y);
            if (!el) return null;
            const rect = el.getBoundingClientRect();
            return {
                tag: el.tagName.toLowerCase(),
                text: (el.innerText || el.value || '').slice(0, 100),
                id: el.id || null,
                classes: [...el.classList],
                bbox: {
                    x: rect.x, y: rect.y,
                    width: rect.width, height: rect.height
                }
            };
        }
    ''', {"x": dom_x, "y": dom_y})

    result = {
        "normalized_0_1000": {"x": norm_x, "y": norm_y},
        "pixel_coords": {"x": pixel_x, "y": pixel_y},
        "dom_coords": {"x": dom_x, "y": dom_y},
        "hit_element": hit,
        "hit_target": False,
    }

    # Check if we hit the target
    if target_text and hit:
        hit_text = (hit.get("text") or "").lower()
        result["hit_target"] = target_text.lower() in hit_text

    return web.json_response(result)


# --- Coordinate Conversion ---

async def handle_coords(request):
    """Get coordinate conversion info."""
    info = await PAGE.evaluate('''
        () => ({
            viewport: { width: window.innerWidth, height: window.innerHeight },
            window: { width: window.outerWidth, height: window.outerHeight },
            position: { x: window.screenX, y: window.screenY },
            screen: { width: screen.width, height: screen.height },
            chrome: {
                width: window.outerWidth - window.innerWidth,
                height: window.outerHeight - window.innerHeight
            }
        })
    ''')

    return web.json_response({
        "info": info,
        "model_height": 704,
        "conversions": {
            "dom_to_screen": f"screen_y = dom_y + {info['chrome']['height']}",
            "screen_to_model": f"model_y = screen_y * (704 / {info['screen']['height']})",
            "model_to_screen": f"screen_y = model_y * ({info['screen']['height']} / 704)",
        }
    })


async def handle_convert(request):
    """Convert coordinates between systems."""
    data = await request.json()
    from_sys = data.get("from", "model")
    to_sys = data.get("to", "screen")
    x = data.get("x", 0)
    y = data.get("y", 0)

    info = await PAGE.evaluate('''
        () => ({
            chrome_height: window.outerHeight - window.innerHeight,
            screen_height: screen.height
        })
    ''')

    chrome_h = info['chrome_height']
    screen_h = info['screen_height']

    # First convert to screen coords
    if from_sys == "model":
        screen_x, screen_y = x, int(y * (screen_h / 704))
    elif from_sys == "dom":
        screen_x, screen_y = x, y + chrome_h
    else:
        screen_x, screen_y = x, y

    # Then convert to target system
    if to_sys == "model":
        out_x, out_y = screen_x, int(screen_y * (704 / screen_h))
    elif to_sys == "dom":
        out_x, out_y = screen_x, screen_y - chrome_h
    else:
        out_x, out_y = screen_x, screen_y

    return web.json_response({
        "input": {"system": from_sys, "x": x, "y": y},
        "output": {"system": to_sys, "x": out_x, "y": out_y},
    })


# --- Training Injection ---

async def handle_inject(request):
    """Inject a training correction."""
    global MODEL, TOKENIZER, PROCESSOR, INJECTOR

    data = await request.json()
    screenshot_path = data.get("screenshot")
    task = data.get("task")
    model_output = data.get("model_output")
    corrected_output = data.get("corrected_output")
    reward = data.get("reward", 1.0)
    fast_mode = data.get("fast", True)
    # Trajectory-based injection: pass full trajectory for sequence training
    trajectory = data.get("trajectory")

    if not all([task, corrected_output]):
        return web.json_response({
            "error": "Required: task, corrected_output"
        }, status=400)

    if MODEL is None:
        return web.json_response({
            "error": "Model not loaded. POST /load_model first",
            "model_loaded": False
        }, status=400)

    # Load screenshot if provided (legacy path)
    screenshot = None
    if screenshot_path:
        try:
            screenshot = Image.open(screenshot_path)
            screenshot = screenshot.resize((1280, 704), Image.Resampling.LANCZOS)
        except Exception as e:
            return web.json_response({"error": f"Failed to load screenshot: {e}"}, status=400)

    from trainer.injection import Correction, TrainingInjector

    if INJECTOR is None:
        INJECTOR = TrainingInjector(MODEL, PROCESSOR or TOKENIZER)

    correction = Correction(
        screenshot=screenshot,
        task=task,
        model_output=model_output or "",
        corrected_output=corrected_output,
        reward=reward,
    )

    if fast_mode:
        result = INJECTOR.inject_fast(correction)
    else:
        result = INJECTOR.inject(correction)
    STATE["corrections_injected"] += 1

    return web.json_response({
        **result,
        "fast_mode": fast_mode,
    })


async def handle_save_checkpoint(request):
    """Save model checkpoint for later resumption."""
    global MODEL

    if MODEL is None:
        return web.json_response({"error": "Model not loaded"}, status=400)

    data = await request.json() if request.body_exists else {}
    checkpoint_dir = data.get("path", "/tmp/vl-checkpoints")
    name = data.get("name", f"checkpoint_{int(time.time())}")

    save_path = Path(checkpoint_dir) / name
    save_path.mkdir(parents=True, exist_ok=True)

    # Save LoRA weights only (not full model)
    MODEL.save_pretrained(str(save_path))

    return web.json_response({
        "saved": True,
        "path": str(save_path),
        "injections": STATE.get("corrections_injected", 0),
    })


async def handle_load_checkpoint(request):
    """Load model checkpoint to resume training."""
    global MODEL, TOKENIZER

    data = await request.json()
    checkpoint_path = data.get("path")

    if not checkpoint_path:
        return web.json_response({"error": "path required"}, status=400)

    if MODEL is None:
        return web.json_response({"error": "Load base model first with /load_model"}, status=400)

    from peft import PeftModel

    # Load LoRA weights
    MODEL = PeftModel.from_pretrained(MODEL.base_model, checkpoint_path)

    return web.json_response({
        "loaded": True,
        "path": checkpoint_path,
    })


def _load_training_model():
    """Load Qwen3-VL-8B with LoRA for training. Not needed for inference (vLLM handles that)."""
    global MODEL, TOKENIZER, PROCESSOR

    if MODEL is not None:
        return

    import torch
    from transformers import Qwen3VLForConditionalGeneration, AutoProcessor
    from peft import get_peft_model, LoraConfig

    print("[model] Loading training model (Qwen3-VL-8B + LoRA)...", flush=True)

    MODEL = Qwen3VLForConditionalGeneration.from_pretrained(
        "Qwen/Qwen3-VL-8B-Instruct",
        torch_dtype=torch.bfloat16,
        device_map="cuda:1",
        attn_implementation="flash_attention_2",
    )

    PROCESSOR = AutoProcessor.from_pretrained(
        "Qwen/Qwen3-VL-8B-Instruct",
        min_pixels=256 * 28 * 28,
        max_pixels=512 * 28 * 28,
    )
    TOKENIZER = PROCESSOR.tokenizer

    lora_config = LoraConfig(
        r=16,
        lora_alpha=16,
        lora_dropout=0,
        bias="none",
        target_modules=["q_proj", "k_proj", "v_proj", "o_proj",
                         "gate_proj", "up_proj", "down_proj"],
        task_type="CAUSAL_LM",
    )
    MODEL = get_peft_model(MODEL, lora_config)
    MODEL.enable_input_require_grads()
    MODEL.gradient_checkpointing_enable()

    trainable = sum(p.numel() for p in MODEL.parameters() if p.requires_grad)
    total = sum(p.numel() for p in MODEL.parameters())
    print(f"[model] Loaded: {trainable:,} trainable / {total:,} total ({100*trainable/total:.2f}%)", flush=True)

    STATE["model_loaded"] = True


async def handle_load_model(request):
    """Load Qwen3-VL-8B with LoRA for trajectory training."""
    if MODEL is not None:
        return web.json_response({"status": "already_loaded"})

    _load_training_model()

    trainable = sum(p.numel() for p in MODEL.parameters() if p.requires_grad)
    total = sum(p.numel() for p in MODEL.parameters())

    return web.json_response({
        "status": "loaded",
        "trainable_params": trainable,
        "total_params": total,
        "trainable_pct": f"{100*trainable/total:.2f}%",
    })


async def handle_reset_trajectory(request):
    """Reset trajectory state for a new episode/stage."""
    global TRAJECTORY
    TRAJECTORY = []
    return web.json_response({"reset": True, "trajectory_len": 0})


async def handle_get_trajectory(request):
    """Get current trajectory state (for debugging)."""
    summary = []
    for entry in TRAJECTORY:
        if entry["type"] == "image":
            summary.append({"type": "image", "timestamp": entry["timestamp"]})
        else:
            summary.append({"type": "action", "text": entry["text"], "timestamp": entry["timestamp"]})
    return web.json_response({"length": len(TRAJECTORY), "entries": summary})


# Max trajectory entries before sliding window
# Fewer images, more room for SEE descriptions between frames
MAX_TRAJECTORY_PAIRS = 8  # 8 image-action pairs


def pil_to_data_uri(img: Image.Image) -> str:
    """Convert PIL Image to base64 data URI for OpenAI-compatible API."""
    buf = io.BytesIO()
    img.save(buf, format="PNG")
    b64 = base64.b64encode(buf.getvalue()).decode()
    return f"data:image/png;base64,{b64}"


def get_vllm_client():
    """Get or create AsyncOpenAI client for vLLM inference."""
    global VLLM_CLIENT
    if VLLM_CLIENT is None:
        from openai import AsyncOpenAI
        VLLM_CLIENT = AsyncOpenAI(
            base_url=VLLM_INFER_URL,
            api_key="unused",
        )
    return VLLM_CLIENT


async def handle_infer(request):
    """
    Trajectory-based inference via vLLM.

    Each call:
    1. Takes a screenshot, adds to trajectory
    2. Builds OpenAI-format messages with base64 images
    3. Calls vLLM for next action prediction
    4. Action added to trajectory
    5. Returns action

    The model sees the rolling temporal context of past observations and actions.
    """
    global TRAJECTORY

    data = await request.json()
    task = data.get("task", "Complete the current task.")
    temperature = data.get("temperature", 0.7)

    screenshots_dir = Path("/tmp/vl-screenshots")
    screenshots_dir.mkdir(exist_ok=True)

    # Take screenshot
    now = time.time()
    screenshot_path = screenshots_dir / f"traj_{int(now * 1000)}.png"
    result = subprocess.run(["grim", "-c", str(screenshot_path)], capture_output=True)
    if result.returncode != 0:
        return web.json_response({
            "error": f"Screenshot failed: {result.stderr.decode()}"
        }, status=500)

    frame = Image.open(screenshot_path).resize((1280, 704), Image.Resampling.LANCZOS)

    # Add image to trajectory
    TRAJECTORY.append({
        "type": "image",
        "image": frame,
        "timestamp": now,
        "path": str(screenshot_path),
    })

    # Slide window if too long
    if len(TRAJECTORY) > MAX_TRAJECTORY_PAIRS * 2:
        # Keep system context fresh, drop oldest pairs
        TRAJECTORY = TRAJECTORY[-(MAX_TRAJECTORY_PAIRS * 2):]

    # Build interleaved messages from trajectory
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

    messages = [{"role": "system", "content": system_content}]

    for entry in TRAJECTORY:
        if entry["type"] == "image":
            t = entry["timestamp"]
            t_rel = t - TRAJECTORY[0]["timestamp"] if TRAJECTORY else 0
            messages.append({
                "role": "user",
                "content": [
                    {"type": "image_url", "image_url": {"url": pil_to_data_uri(entry["image"])}},
                    {"type": "text", "text": f"[t={t_rel:.2f}s]"},
                ],
            })
        elif entry["type"] == "action":
            messages.append({
                "role": "assistant",
                "content": entry["text"],
            })

    # Call vLLM for inference
    client = get_vllm_client()
    n_images = sum(1 for e in TRAJECTORY if e["type"] == "image")
    print(f"[infer] images={n_images}, traj_len={len(TRAJECTORY)}", flush=True)

    try:
        completion = await client.chat.completions.create(
            model=VLLM_INFER_MODEL,
            messages=messages,
            max_tokens=200,
            temperature=temperature,
            extra_body={"min_p": 0.1},
        )
        response = completion.choices[0].message.content.strip()
        input_tokens = completion.usage.prompt_tokens if completion.usage else 0
        output_tokens = completion.usage.completion_tokens if completion.usage else 0
    except Exception as e:
        import traceback
        return web.json_response({
            "error": f"vLLM inference error: {e}",
            "traceback": traceback.format_exc(),
        }, status=500)

    # Parse SEE and ACTION from response
    see = ""
    action = None
    valid_prefixes = ('CLICK', 'TYPE', 'KEY', 'SCROLL', 'WAIT', 'DONE')

    for line in response.split('\n'):
        line = line.strip()
        if not line:
            continue
        if line.upper().startswith("SEE:"):
            see = line[4:].strip()
        elif line.upper().startswith("ACTION:"):
            rest = line[7:].strip()
            parts = rest.split()
            if parts and parts[0].upper() in valid_prefixes:
                action = rest
        else:
            # Fallback: bare action line
            parts = line.split()
            if parts and parts[0].upper() in valid_prefixes:
                action = line

    if not action:
        action = "WAIT"

    # Store full response (SEE + ACTION) in trajectory
    full_turn = response if see else action
    TRAJECTORY.append({
        "type": "action",
        "text": full_turn,
        "timestamp": time.time(),
    })

    STATE["last_screenshot"] = str(screenshot_path)

    elapsed = time.time() - now

    return web.json_response({
        "action": action,
        "see": see,
        "full_response": response,
        "trajectory_len": len(TRAJECTORY),
        "task": task,
        "screenshot": str(screenshot_path),
        "inference_time": now,
        "temperature": temperature,
        "input_tokens": input_tokens,
        "output_tokens": output_tokens,
        "elapsed_s": round(elapsed, 2),
    })


async def handle_train_trajectory(request):
    """
    Train on the current trajectory (successful completion).

    Unrolls the trajectory into windowed sub-sequences and trains
    with loss only on action tokens. Only call this after the model
    successfully completes a task.

    Input:
        - task: task description

    Returns:
        Training stats (loss, grad_norm, num_examples, etc.)
    """
    global MODEL, PROCESSOR, TRAJECTORY, TRAINER

    data = await request.json()
    task = data.get("task", "Complete the current task.")

    # Lazy-load training model (not needed for inference, only for training)
    if MODEL is None:
        print("[train] Training model not loaded, loading now...", flush=True)
        _load_training_model()


    if TRAINER is None:
        from trainer.injection import TrajectoryTrainer
        TRAINER = TrajectoryTrainer(
            MODEL,
            PROCESSOR,
            window_size=TRAJECTORY_CONFIG["window_size"],
            grad_clip=TRAJECTORY_CONFIG["grad_clip"],
            metrics_logger=METRICS_LOGGER,
        )

    if not TRAJECTORY:
        return web.json_response({"error": "No trajectory data"}, status=400)

    result = TRAINER.train_on_trajectory(TRAJECTORY, task)

    if result.get("trained"):
        STATE["corrections_injected"] += 1

    return web.json_response(result)


# --- GRPO Rollout Management ---

async def handle_append_trajectory(request):
    """Manually append an entry to the trajectory (for synthetic demonstrations)."""
    global TRAJECTORY

    data = await request.json()
    entry_type = data.get("type")

    if entry_type == "image":
        screenshots_dir = Path("/tmp/vl-screenshots")
        screenshots_dir.mkdir(exist_ok=True)
        now = time.time()
        screenshot_path = screenshots_dir / f"demo_{int(now * 1000)}.png"
        result = subprocess.run(["grim", "-c", str(screenshot_path)], capture_output=True)
        if result.returncode != 0:
            return web.json_response({"error": "Screenshot failed"}, status=500)

        frame = Image.open(screenshot_path).resize((1280, 704), Image.Resampling.LANCZOS)
        TRAJECTORY.append({
            "type": "image",
            "image": frame,
            "timestamp": now,
            "path": str(screenshot_path),
        })
    elif entry_type == "action":
        text = data.get("text", "")
        TRAJECTORY.append({
            "type": "action",
            "text": text,
            "timestamp": time.time(),
        })
    else:
        return web.json_response({"error": "type must be 'image' or 'action'"}, status=400)

    return web.json_response({
        "appended": entry_type,
        "trajectory_len": len(TRAJECTORY),
    })


async def handle_save_rollout(request):
    """Save current TRAJECTORY as a rollout with a reward for GRPO."""
    global ROLLOUTS, TRAJECTORY

    data = await request.json()
    reward = data.get("reward", 0.0)

    if not TRAJECTORY:
        return web.json_response({"error": "No trajectory to save"}, status=400)

    ROLLOUTS.append({
        "trajectory": list(TRAJECTORY),
        "reward": reward,
    })

    return web.json_response({
        "saved": True,
        "reward": reward,
        "trajectory_len": len(TRAJECTORY),
        "total_rollouts": len(ROLLOUTS),
    })


async def handle_clear_rollouts(request):
    """Clear all stored rollouts."""
    global ROLLOUTS
    ROLLOUTS = []
    return web.json_response({"cleared": True})


async def handle_grpo_train(request):
    """Run GRPO training on stored rollouts."""
    global MODEL, PROCESSOR, ROLLOUTS, GRPO_TRAINER

    data = await request.json()
    task = data.get("task", "Complete the current task.")

    if not ROLLOUTS:
        return web.json_response({"error": "No rollouts stored"}, status=400)

    if MODEL is None:
        print("[grpo] Training model not loaded, loading now...", flush=True)
        try:
            _load_training_model()
        except Exception as e:
            return web.json_response({"error": f"Model load failed: {e}"}, status=500)

    if GRPO_TRAINER is None:
        from trainer.grpo import GRPOTrainer
        GRPO_TRAINER = GRPOTrainer(
            MODEL,
            PROCESSOR,
            advantage_threshold=GRPO_CONFIG["advantage_threshold"],
            val_fraction=GRPO_CONFIG["val_fraction"],
            grad_clip=GRPO_CONFIG["grad_clip"],
            metrics_logger=METRICS_LOGGER,
        )

    try:
        result = GRPO_TRAINER.train_step(ROLLOUTS, task)
    except Exception as e:
        import traceback
        traceback.print_exc()
        return web.json_response({"error": f"GRPO train failed: {e}"}, status=500)

    # Clear rollouts after training
    ROLLOUTS = []

    return web.json_response(result)


# --- Oracle (GPT-OSS grader) ---

async def handle_oracle(request):
    """
    Oracle GRADER: Given before/after state + action taken, grade if correct.
    If incorrect, provide the corrected action for training injection.

    Input:
        - task: what the model was trying to do
        - action: what the model outputted (e.g., "CLICK 500 400")
        - before_dom: DOM state before action (optional, will use stored)
        - after_dom: DOM state after action (optional, will capture current)
        - element_hit: what element was at click coords (optional)

    Output:
        - correct: bool
        - reasoning: why correct/incorrect
        - correction: what action should have been taken (if incorrect)
    """
    global HTTP_CLIENT
    import httpx

    data = await request.json()
    task = data.get("task", "Complete the current task on screen.")
    action_taken = data.get("action", "")  # Full multi-line action string
    actions_taken = data.get("actions", [])  # Individual actions list
    before_dom = data.get("before_dom")  # Stored before execution
    element_hit = data.get("element_hit")  # Legacy single hit
    action_hits = data.get("action_hits", [])  # Per-action hit results
    inference_time = data.get("inference_time")  # When model made decision

    if HTTP_CLIENT is None:
        HTTP_CLIENT = httpx.AsyncClient(timeout=60.0)

    # Check for elements that appeared AFTER the model made its decision
    new_elements_warning = ""
    if inference_time:
        new_elements = get_new_elements_since(inference_time)
        if new_elements:
            new_desc = ", ".join([f"{e['tag']}:\"{e.get('text','')}\"" for e in new_elements[:3]])
            new_elements_warning = f"\n\nIMPORTANT: {len(new_elements)} elements appeared AFTER the model took its screenshots ({new_desc}). Do NOT penalize the model for not interacting with these elements - they were not visible when the decision was made."

    # Get current (after) DOM state
    after_dom = await PAGE.evaluate('''
        () => {
            const selectors = 'a, button, input, select, textarea, [onclick], [role="button"], [role="link"]';
            const elements = Array.from(document.querySelectorAll(selectors)).map(el => {
                const rect = el.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return null;

                const dom_cx = rect.x + rect.width / 2;
                const dom_cy = rect.y + rect.height / 2;
                const chrome_h = window.outerHeight - window.innerHeight;
                const screen_y = dom_cy + chrome_h;
                const pixel_x = Math.round(dom_cx);
                const pixel_y = Math.round(screen_y * (704 / screen.height));
                const norm_x = Math.round(pixel_x / 1280 * 1000);
                const norm_y = Math.round(pixel_y / 704 * 1000);

                return {
                    tag: el.tagName.toLowerCase(),
                    text: (el.innerText || el.value || el.placeholder || '').slice(0, 50).trim(),
                    norm_coords: { x: norm_x, y: norm_y }
                };
            }).filter(Boolean);

            return {
                url: window.location.href,
                title: document.title,
                visible_text: document.body.innerText.slice(0, 1500),
                elements: elements.slice(0, 30)
            };
        }
    ''')

    # Format elements for prompt
    def format_elements(dom):
        if not dom or not dom.get('elements'):
            return "(no elements)"
        return "\n".join([
            f"  - {el['tag']}: \"{el['text']}\" at CLICK {el['norm_coords']['x']} {el['norm_coords']['y']}"
            for el in dom['elements'] if el.get('text')
        ])

    before_text = before_dom.get('visible_text', '')[:600] if before_dom else "(not provided)"
    before_elements = format_elements(before_dom) if before_dom else "(not provided)"
    after_text = after_dom.get('visible_text', '')[:600]
    after_elements = format_elements(after_dom)

    # Build grading prompt
    oracle_prompt = f"""You are an oracle grader for a VLM computer-use agent learning via RL.

TASK THE MODEL WAS GIVEN: {task}

ACTION(S) THE MODEL TOOK:
{action_taken}

PER-ACTION RESULTS:
{chr(10).join([f"  {i+1}. {h['action']} → hit: {json.dumps(h.get('element_hit')) if h.get('element_hit') else '(no click)'}" for i, h in enumerate(action_hits)]) if action_hits else f"  1. {action_taken} → hit: {json.dumps(element_hit) if element_hit else '(unknown)'}"}

=== BEFORE STATE ===
URL: {before_dom.get('url', '?') if before_dom else '?'}
Text: {before_text}
Elements:
{before_elements}

=== AFTER STATE ===
URL: {after_dom.get('url', '?')}
Text: {after_text}
Elements:
{after_elements}

COORDINATE SYSTEM: Actions use normalized 0-1000 coordinates (0,0)=top-left, (1000,1000)=bottom-right
{new_elements_warning}

YOUR JOB:
1. Did the action make progress toward the task?
2. Was the click on a relevant element?
3. Did the page state change appropriately?
4. IMPORTANT: Only judge based on what the model could see at decision time. Do not penalize for elements that appeared after.

OUTPUT JSON ONLY:
{{
    "correct": true/false,
    "reasoning": "brief explanation of why correct or incorrect",
    "correction": "CLICK x y" or "TYPE text" or multi-line "TYPE text\nCLICK x y" (only if incorrect, null if correct. One action per line.)
}}"""

    try:
        response = await HTTP_CLIENT.post(
            ORACLE_URL,
            json={
                "model": ORACLE_MODEL,
                "messages": [{"role": "user", "content": oracle_prompt}],
                "max_tokens": 16384,
                "temperature": 0.2,
            },
        )

        result = response.json()
        choice = result.get("choices", [{}])[0]
        message = choice.get("message", {})

        content = message.get("content") or ""
        thinking = message.get("reasoning_content") or message.get("reasoning") or ""

        # Parse JSON from response - check both content and thinking
        import re
        grade = {"correct": None, "reasoning": "parse_error", "correction": None}

        # Try to find JSON in content first, then in thinking
        text_to_parse = content if content else thinking
        if text_to_parse:
            json_match = re.search(r'\{[^{}]*\}', text_to_parse, re.DOTALL)
            if json_match:
                try:
                    grade = json.loads(json_match.group())
                except:
                    pass

        return web.json_response({
            "task": task,
            "action_taken": action_taken,
            "element_hit": element_hit,
            "grade": grade,
            "thinking": thinking,
            "raw_response": content,
        })

    except Exception as e:
        return web.json_response({
            "error": str(e),
            "task": task,
            "action_taken": action_taken,
        }, status=500)


async def handle_capture_state(request):
    """
    Capture current DOM state for before/after comparison.
    Call this BEFORE executing an action to store the 'before' state.
    """
    dom_state = await PAGE.evaluate('''
        () => {
            const selectors = 'a, button, input, select, textarea, [onclick], [role="button"], [role="link"]';
            const elements = Array.from(document.querySelectorAll(selectors)).map(el => {
                const rect = el.getBoundingClientRect();
                if (rect.width === 0 || rect.height === 0) return null;

                const dom_cx = rect.x + rect.width / 2;
                const dom_cy = rect.y + rect.height / 2;
                const chrome_h = window.outerHeight - window.innerHeight;
                const screen_y = dom_cy + chrome_h;
                const pixel_x = Math.round(dom_cx);
                const pixel_y = Math.round(screen_y * (704 / screen.height));
                const norm_x = Math.round(pixel_x / 1280 * 1000);
                const norm_y = Math.round(pixel_y / 704 * 1000);

                return {
                    tag: el.tagName.toLowerCase(),
                    text: (el.innerText || el.value || el.placeholder || '').slice(0, 50).trim(),
                    id: el.id || null,
                    norm_coords: { x: norm_x, y: norm_y }
                };
            }).filter(Boolean);

            return {
                url: window.location.href,
                title: document.title,
                visible_text: document.body.innerText.slice(0, 2000),
                elements: elements.slice(0, 50),
                timestamp: Date.now()
            };
        }
    ''')

    # Also take screenshot
    screenshots_dir = Path("/tmp/vl-screenshots")
    screenshots_dir.mkdir(exist_ok=True)
    timestamp = int(time.time() * 1000)
    screenshot_path = screenshots_dir / f"state_{timestamp}.png"

    subprocess.run(["grim", "-c", str(screenshot_path)], capture_output=True)

    return web.json_response({
        "dom": dom_state,
        "screenshot": str(screenshot_path),
    })


# --- Reload ---

async def handle_reload(request):
    """Reload to start page."""
    url = "http://127.0.0.1:37163/"
    await PAGE.goto(url)
    await PAGE.wait_for_load_state("networkidle")

    return web.json_response({
        "url": PAGE.url,
        "reloaded": True,
    })


# --- State ---

async def handle_state(request):
    """Get current state."""
    return web.json_response({
        **STATE,
        "current_url": PAGE.url if PAGE else None,
        "timestamp": datetime.now().isoformat(),
    })


async def handle_animations(request):
    """Get current animation status from DOM timeline."""
    if not DOM_TIMELINE:
        return web.json_response({"has_animations": False, "message": "No DOM snapshots yet"})

    latest = DOM_TIMELINE[-1]
    return web.json_response({
        "timestamp": latest["timestamp"],
        "has_animations": latest.get("has_animations", False),
        "animations": latest.get("animations", {}),
        "url": latest.get("url"),
    })


async def handle_dom_at_time(request):
    """Get DOM state at a specific timestamp (for fair oracle grading)."""
    data = await request.json()
    timestamp = data.get("timestamp")

    if not timestamp:
        return web.json_response({"error": "timestamp required"}, status=400)

    dom_snapshot = get_dom_at_time(timestamp)
    new_elements = get_new_elements_since(timestamp)

    return web.json_response({
        "requested_timestamp": timestamp,
        "snapshot": dom_snapshot,
        "new_elements_since": new_elements,
        "new_element_count": len(new_elements),
    })


# --- Index ---

async def handle_index(request):
    """Show available endpoints."""
    return web.json_response({
        "name": "VL-Computer-Use Control Server",
        "endpoints": {
            "GET /": "This help",
            "GET /screenshot": "Take screenshot, return path",
            "GET /dom": "Get DOM state and clickable elements (with model coords)",
            "POST /execute": "Execute action {action: 'CLICK 640 500'}",
            "POST /check": "Check coords {x, y, target?}",
            "GET /coords": "Get coordinate conversion info",
            "POST /convert": "Convert coords {from, to, x, y}",
            "POST /load_model": "Load VLM model with LoRA",
            "POST /save_checkpoint": "Save LoRA checkpoint {path?, name?}",
            "POST /load_checkpoint": "Load LoRA checkpoint {path}",
            "POST /infer": "Run inference with 2 frames {task, temperature?, frame_delay?}",
            "POST /inject": "Inject training {screenshot, task, corrected_output}",
            "POST /oracle": "Grade action {task, action, before_dom, element_hit}",
            "GET /capture": "Capture current DOM state (call before action)",
            "POST /reload": "Reload to start page",
            "GET /state": "Get current state",
        },
        "state": STATE,
    })


async def handle_eval(request):
    """Evaluate arbitrary JS on the page and return result."""
    data = await request.json()
    js = data.get("js", "")
    try:
        result = await PAGE.evaluate(js)
        return web.json_response({"result": result})
    except Exception as e:
        return web.json_response({"error": str(e)}, status=500)


async def handle_navigate(request):
    """Navigate to a URL."""
    data = await request.json()
    url = data.get("url", "")
    try:
        await PAGE.goto(url, wait_until="networkidle", timeout=15000)
        return web.json_response({"url": PAGE.url, "title": await PAGE.title()})
    except Exception as e:
        return web.json_response({"error": str(e)}, status=500)


def create_app():
    """Create the aiohttp application."""
    app = web.Application()
    app.router.add_get("/", handle_index)
    app.router.add_get("/screenshot", handle_screenshot)
    app.router.add_get("/dom", handle_dom)
    app.router.add_post("/execute", handle_execute)
    app.router.add_post("/check", handle_check)
    app.router.add_get("/coords", handle_coords)
    app.router.add_post("/convert", handle_convert)
    app.router.add_post("/load_model", handle_load_model)
    app.router.add_post("/save_checkpoint", handle_save_checkpoint)
    app.router.add_post("/load_checkpoint", handle_load_checkpoint)
    app.router.add_post("/infer", handle_infer)
    app.router.add_post("/reset_trajectory", handle_reset_trajectory)
    app.router.add_get("/trajectory", handle_get_trajectory)
    app.router.add_post("/inject", handle_inject)
    app.router.add_post("/train_trajectory", handle_train_trajectory)
    app.router.add_post("/append_to_trajectory", handle_append_trajectory)
    app.router.add_post("/save_rollout", handle_save_rollout)
    app.router.add_post("/clear_rollouts", handle_clear_rollouts)
    app.router.add_post("/grpo_train", handle_grpo_train)
    app.router.add_post("/oracle", handle_oracle)
    app.router.add_get("/capture", handle_capture_state)
    app.router.add_post("/reload", handle_reload)
    app.router.add_get("/state", handle_state)
    app.router.add_get("/animations", handle_animations)
    app.router.add_post("/dom_at_time", handle_dom_at_time)
    app.router.add_post("/eval", handle_eval)
    app.router.add_post("/navigate", handle_navigate)
    return app


async def main():
    print("=" * 50)
    print("VL-Computer-Use Control Server")
    print("=" * 50)

    setup_wayland()
    await setup_browser()

    STATE["started_at"] = datetime.now().isoformat()

    app = create_app()
    runner = web.AppRunner(app)
    await runner.setup()

    site = web.TCPSite(runner, "127.0.0.1", 8080)
    await site.start()

    print("\nServer running at http://127.0.0.1:8080")
    print("\nEndpoints:")
    print("  GET  /screenshot  - take screenshot")
    print("  GET  /dom         - get DOM + elements with model coords")
    print("  POST /execute     - execute action")
    print("  POST /check       - check if coords hit target")
    print("  POST /inject      - inject training correction")
    print("  POST /reload      - reload to start")
    print("  GET  /animations  - get current animation status")
    print("\nPress Ctrl+C to stop")

    # Start DOM polling for animation tracking
    global DOM_POLL_TASK
    DOM_POLL_TASK = asyncio.create_task(poll_dom())
    print("[DOM] Started animation/element polling (100ms interval)")

    # Keep running
    while True:
        await asyncio.sleep(3600)


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\nShutting down...")
