#!/usr/bin/env python3
"""
Collect thought traces for OBSERVE → THINK → ACT training.

Navigates stages, captures real screenshots, builds traces with
hand-crafted reasoning around DOM ground truth actions.

Saves to /tmp/vl-traces/ as JSON + images.

Usage:
    uv run python collect_traces.py --stages 1 4 5 6
    uv run python collect_traces.py --stages 1 2 3 4 5 6 7 --traces-per-stage 3
"""
import argparse
import json
import time
import sys
from pathlib import Path
from datetime import datetime
from PIL import Image
import requests
import numpy as np

SERVER = "http://localhost:8080"
SITE = "http://127.0.0.1:37163"
TRACE_DIR = Path(__file__).parent / "traces"

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

SYSTEM_PROMPT = """You control a browser. Each turn you receive two images: the current screenshot and a diff image highlighting pixel changes since the last frame. Use the diff to detect movement, animations, and state changes.

You have three modes:
- OBSERVE: Look at the current state. No action taken. You receive a fresh frame.
- THINK: Reason about what you see, check the diff for movement, and plan your next moves. No action taken.
- ACT: Execute an action.

Always check the diff for movement before acting. If elements are moving, account for their trajectory. If nothing is moving, note that it's safe to act at current positions.

Actions (normalized 0-1000 coordinates):
- CLICK x y
- TYPE text
- KEY keyname
- SCROLL dy (positive=down)
- WAIT
- DONE"""


def log(msg, level="INFO"):
    ts = datetime.now().strftime("%H:%M:%S")
    print(f"[{ts}] [{level}] {msg}", flush=True)


def api(method, endpoint, data=None, timeout=120):
    url = f"{SERVER}{endpoint}"
    if method == "GET":
        resp = requests.get(url, timeout=timeout)
    else:
        resp = requests.post(url, json=data, timeout=timeout)
    return resp.json()


def js_eval(js):
    return api("POST", "/eval", {"js": js}).get("result")


def navigate(url):
    return api("POST", "/navigate", {"url": url})


def get_task_state():
    return js_eval("window.getTaskState()")


def execute(action):
    return api("POST", "/execute", {"action": action.replace(",", "")})


def take_screenshot():
    result = api("GET", "/screenshot")
    path = result.get("path")
    if not path:
        raise RuntimeError(f"Screenshot failed: {result}")
    return Image.open(path).resize((1024, 1024), Image.Resampling.LANCZOS)


def compute_diff(current: Image.Image, previous: Image.Image) -> Image.Image:
    """Pixel-level diff between two frames, highlights changes."""
    curr_arr = np.array(current).astype(np.int16)
    prev_arr = np.array(previous).astype(np.int16)
    diff = np.abs(curr_arr - prev_arr).clip(0, 255).astype(np.uint8)
    return Image.fromarray(diff)


def get_dom_elements():
    return api("GET", "/dom").get("elements", [])


def resolve_expected(expected, elements, task_state):
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
                return el_coords(el), el
        for el in elements:
            el_text = (el.get("text") or "").strip().lower()
            if el_text and el_text in target_parts:
                return el_coords(el), el
        for el in elements:
            el_id = (el.get("id") or "").lower()
            el_text = (el.get("text") or "").lower()
            for tp in target_parts:
                if tp in el_id or tp in el_text:
                    return el_coords(el), el
    elif verb == "DISMISS":
        for el in elements:
            el_id = (el.get("id") or "").lower()
            el_text = (el.get("text") or "").lower()
            el_classes = " ".join(el.get("classes") or []).lower()
            if (any(tp in el_id for tp in target_parts) or
                any(tp in el_classes for tp in target_parts) or
                any(w in el_text for w in ["close", "dismiss", "accept", "got it", "\u00d7", "x", "ok"]) or
                any(w in el_classes for w in ["close", "dismiss"])):
                return el_coords(el), el
    elif verb == "SCROLL":
        return "SCROLL 3", None
    elif verb == "TYPE":
        custom = task_state.get("custom", {})
        code = custom.get("expected_code", "")
        if code:
            return f"TYPE {code}", None
    return "WAIT", None


def describe_elements(elements, max_els=10):
    """Build a human-readable description of visible elements."""
    parts = []
    for el in elements[:max_els]:
        tag = el.get("tag", "?")
        text = (el.get("text") or "").strip()
        el_id = el.get("id", "")
        coords = el.get("coords", {}).get("normalized", {})
        x, y = coords.get("x", "?"), coords.get("y", "?")

        if text:
            parts.append(f'{tag} "{text}" at ({x}, {y})')
        elif el_id:
            parts.append(f'{tag}#{el_id} at ({x}, {y})')
    return parts


def build_think_text(stage, step, elements, expected, target_el, task_state):
    """Generate a realistic THINK trace for the current step."""
    el_desc = describe_elements(elements)
    page_desc = "; ".join(el_desc) if el_desc else "page elements visible"

    # Movement assessment — always check diff for motion
    movement_note = ("Checking the diff view — no movement detected, elements are stationary. "
                     "Safe to click at current positions.")

    # Stage-specific reasoning
    if stage == 1:
        if step == 0:
            return (f"I see the page with: {page_desc}. "
                    f"{movement_note} "
                    f"The task is to click the Submit button. "
                    f"I can see a Submit button. I'll click it.")
        return (f"I see: {page_desc}. {movement_note} "
                f"Looking for the Submit button to click it.")

    elif stage == 2:
        if step == 0:
            return (f"I see several buttons: {page_desc}. "
                    f"{movement_note} "
                    f"I need to find the correct one. Wrong ones turn red. "
                    f"I'll start by clicking one and use the feedback.")
        return (f"I see: {page_desc}. {movement_note} "
                f"Some buttons may have turned red from wrong clicks. "
                f"I'll try a different one that hasn't been tried yet.")

    elif stage == 3:
        if step == 0:
            return (f"I see: {page_desc}. "
                    f"{movement_note} "
                    f"Buttons shuffle after wrong clicks, so I need to re-scan each time. "
                    f"I'll click one and adapt based on shuffled positions.")
        return (f"Buttons shuffled. Current layout: {page_desc}. "
                f"Checking diff — the buttons have repositioned since last frame. "
                f"Re-scanning to find the correct button in its new position.")

    elif stage == 4:
        # Sequential button clicking — step number = which button we're on
        current_button = step + 1
        if step == 0:
            return (f"I see 4 numbered buttons: {page_desc}. "
                    f"{movement_note} "
                    f"I need to click them in order 1→2→3→4. "
                    f"Starting with button 1.")
        if target_el:
            text = (target_el.get("text") or "").strip()
            c = target_el.get("coords", {}).get("normalized", {})
            return (f"Button {current_button - 1} clicked successfully. "
                    f"Checking diff — the clicked button changed state but nothing is moving. "
                    f"Now I need button {current_button}. "
                    f"I can see it labeled '{text}' at ({c.get('x', '?')}, {c.get('y', '?')}). Clicking it.")
        return (f"Progressing through the sequence. {movement_note} "
                f"Next is button {current_button}. Current layout: {page_desc}.")

    elif stage == 5:
        if step == 0:
            return (f"I see: {page_desc}. {movement_note} "
                    f"There are decoy buttons that look similar to the real Submit. "
                    f"I need to identify the real one — it may differ in styling, size, or exact label.")
        return (f"I see: {page_desc}. {movement_note} "
                f"Checking which button is the real Submit vs decoys.")

    elif stage == 6:
        if "DISMISS" in expected.upper() if expected else False:
            return (f"I see a popup blocking the page: {page_desc}. "
                    f"{movement_note} "
                    f"I need to dismiss it first before I can reach the goal button. "
                    f"Looking for a close/dismiss/accept button on the popup.")
        return (f"Popup dismissed. Now I can see the page: {page_desc}. "
                f"{movement_note} "
                f"Looking for the goal button to click.")

    elif stage == 7:
        if "SCROLL" in expected.upper() if expected else False:
            return (f"I see: {page_desc}. {movement_note} "
                    f"The target button is not visible yet. I need to scroll down to find it.")
        return (f"After scrolling, I can now see: {page_desc}. "
                f"{movement_note} "
                f"I can see the target button. Clicking it.")

    elif stage == 8:
        if "TYPE" in expected.upper() if expected else False:
            code = task_state.get("custom", {}).get("expected_code", "???")
            return (f"I see a code block on the page showing: '{code}'. "
                    f"{movement_note} "
                    f"I need to type this into the input field.")
        if "CLICK" in expected.upper() if expected else False:
            return (f"I've typed the code. Now I need to click Submit to confirm. "
                    f"{movement_note} I see: {page_desc}.")
        return f"I see: {page_desc}. {movement_note} Reading the code to type it."

    # Default
    return (f"I see: {page_desc}. {movement_note} "
            f"Planning my next action based on the task requirements.")


def collect_trace(stage, trace_id):
    """Collect one thought trace for a stage.

    Every user turn with a screenshot includes [current_frame, diff_frame].
    Diff shows pixel-level changes since previous frame.

    Returns {
        "stage": int,
        "task": str,
        "trace_id": str,
        "messages": [...],  # with image placeholders (2 per visual turn)
        "image_paths": [str, ...],
    }
    """
    task = STAGE_TASKS.get(stage, "Complete the task.")
    navigate(f"{SITE}/level{stage}")
    time.sleep(1)

    system_content = f"{SYSTEM_PROMPT}\n\nTask: {task}"
    messages = [{"role": "system", "content": system_content}]
    image_paths = []
    prev_img = None  # track previous frame for diff

    trace_dir = TRACE_DIR / f"stage{stage}" / trace_id
    trace_dir.mkdir(parents=True, exist_ok=True)

    def add_visual_turn(img, label):
        """Add a user turn with [current, diff] images."""
        nonlocal prev_img

        # Save current frame
        img_path = str(trace_dir / f"{label}.png")
        img.save(img_path)
        image_paths.append(img_path)

        # Compute and save diff
        if prev_img is not None:
            diff = compute_diff(img, prev_img)
        else:
            # First frame: diff is blank (no previous)
            diff = Image.new("RGB", img.size, (0, 0, 0))
        diff_path = str(trace_dir / f"{label}_diff.png")
        diff.save(diff_path)
        image_paths.append(diff_path)

        messages.append({
            "role": "user",
            "content": [
                {"type": "image"},
                {"type": "image"},
                {"type": "text", "text": "Current view and diff from previous frame."},
            ],
        })
        prev_img = img

    for step in range(STAGE_MAX_STEPS.get(stage, 15)):
        task_state = get_task_state()
        if not task_state:
            break
        if task_state.get("completed"):
            break

        expected = task_state.get("expected_next", "")
        if not expected:
            break

        # Screenshot + DOM
        img = take_screenshot()
        elements = get_dom_elements()
        action, target_el = resolve_expected(expected, elements, task_state)

        if action == "WAIT":
            log(f"  Step {step}: can't resolve expected={expected}", "WARN")
            break

        # User turn — current frame + diff
        add_visual_turn(img, f"step{step}")

        if step == 0:
            # First step: OBSERVE → fresh frame+diff → THINK → fresh frame+diff → ACT
            messages.append({"role": "assistant", "content": "OBSERVE"})

            img2 = take_screenshot()
            add_visual_turn(img2, f"step{step}_observe")

            think_text = build_think_text(stage, step, elements, expected, target_el, task_state)
            messages.append({"role": "assistant", "content": f"THINK {think_text}"})

            img3 = take_screenshot()
            add_visual_turn(img3, f"step{step}_post_think")
            messages.append({"role": "assistant", "content": f"ACT {action}"})

        elif stage in (4, 6, 8, 10):
            # Complex stages: THINK → fresh frame+diff → ACT
            think_text = build_think_text(stage, step, elements, expected, target_el, task_state)
            messages.append({"role": "assistant", "content": f"THINK {think_text}"})

            img_post = take_screenshot()
            add_visual_turn(img_post, f"step{step}_post_think")
            messages.append({"role": "assistant", "content": f"ACT {action}"})

        else:
            # Simple steps: just act
            messages.append({"role": "assistant", "content": f"ACT {action}"})

        # Execute
        execute(action)
        time.sleep(0.3)

    # Verify completion
    task_state = get_task_state()
    completed = task_state and task_state.get("completed", False)

    if completed:
        log(f"  Trace OK: {len(image_paths)} steps")
    else:
        log(f"  Trace FAIL at step {step}", "WARN")

    return {
        "stage": stage,
        "task": task,
        "trace_id": trace_id,
        "completed": completed,
        "messages": messages,
        "image_paths": image_paths,
    }


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--stages", type=int, nargs="+", default=[1, 4, 5, 6])
    parser.add_argument("--traces-per-stage", type=int, default=3)
    args = parser.parse_args()

    TRACE_DIR.mkdir(parents=True, exist_ok=True)

    # Check server
    try:
        api("GET", "/screenshot")
        log("Server connected.")
    except Exception as e:
        log(f"Server not available: {e}", "ERROR")
        sys.exit(1)

    all_traces = []

    for stage in args.stages:
        task = STAGE_TASKS.get(stage, "?")
        log(f"\n{'='*50}")
        log(f"Stage {stage}: {task}")
        log(f"{'='*50}")

        for i in range(args.traces_per_stage):
            trace_id = f"trace_{i}"
            log(f"  Collecting trace {i+1}/{args.traces_per_stage}...")
            trace = collect_trace(stage, trace_id)
            if trace["completed"]:
                all_traces.append(trace)
            else:
                log(f"  Skipping failed trace", "WARN")

    # Save
    output_path = TRACE_DIR / "traces.json"
    with open(output_path, "w") as f:
        json.dump(all_traces, f, indent=2)

    log(f"\nSaved {len(all_traces)} traces to {output_path}")
    log(f"Images in {TRACE_DIR}/")

    # Summary
    log("\nSummary:")
    by_stage = {}
    for t in all_traces:
        by_stage.setdefault(t["stage"], []).append(t)
    for stage, traces in sorted(by_stage.items()):
        total_steps = sum(len(t["image_paths"]) for t in traces)
        log(f"  Stage {stage}: {len(traces)} traces, {total_steps} total steps")


if __name__ == "__main__":
    main()
