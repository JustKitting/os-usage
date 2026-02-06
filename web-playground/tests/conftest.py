"""Shared fixtures for component-level Playwright tests."""

import subprocess
import time
import socket

import pytest
from playwright.sync_api import Page


TEST_PORT = 8081
BASE_URL = f"http://localhost:{TEST_PORT}"


def _port_open(port: int) -> bool:
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(("localhost", port)) == 0


@pytest.fixture(scope="session")
def server():
    """Build and serve the app on a dedicated test port."""
    proc = subprocess.Popen(
        [
            "dx", "serve",
            "--release",
            "--port", str(TEST_PORT),
            "--open", "false",
            "--hot-reload", "false",
            "--interactive", "false",
        ],
        cwd="/home/kit/Documents/Python/vl-computer-use/web-playground",
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
    )

    # Wait for the server to be ready (up to 120s for compilation)
    deadline = time.time() + 120
    while time.time() < deadline:
        if proc.poll() is not None:
            out = proc.stdout.read().decode() if proc.stdout else ""
            raise RuntimeError(f"dx serve exited early (code {proc.returncode}):\n{out}")
        if _port_open(TEST_PORT):
            # Port open — give it a moment to finish serving the WASM
            time.sleep(2)
            break
        time.sleep(1)
    else:
        proc.kill()
        raise RuntimeError("dx serve did not start within 120s")

    yield BASE_URL

    proc.terminate()
    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()


@pytest.fixture
def setup(page: Page, server):
    """Navigate to a test route, freeze the viewport at 800x600, and wait."""

    def _setup(route: str):
        page.goto(f"{server}{route}")
        page.wait_for_selector("#viewport", state="visible", timeout=10_000)
        # The freshly-built code has data-fixed support in autoFit, but set
        # the attribute + dimensions explicitly in case of timing races.
        page.evaluate("""() => {
            const vp = document.getElementById('viewport');
            vp.dataset.fixed = 'true';
            vp.style.width = '800px';
            vp.style.height = '600px';
        }""")
        time.sleep(0.1)
        return page

    return _setup


@pytest.fixture
def result(page: Page):
    """Read the hidden #result div's text content."""

    def _result() -> str:
        el = page.locator("#result")
        return el.text_content() or ""

    return _result


@pytest.fixture
def vp_offset(page: Page):
    """Get the #viewport element's page position for coordinate translation."""

    def _offset() -> dict:
        return page.evaluate("""() => {
            const vp = document.getElementById('viewport');
            const r = vp.getBoundingClientRect();
            return { x: r.left, y: r.top };
        }""")

    return _offset


@pytest.fixture
def console_events(page: Page):
    """Capture JSON-logged events from the global event listeners."""
    events = []

    def handler(msg):
        text = msg.text
        if text.startswith("{") and '"event"' in text:
            import json
            try:
                events.append(json.loads(text))
            except json.JSONDecodeError:
                pass

    page.on("console", handler)
    return events
