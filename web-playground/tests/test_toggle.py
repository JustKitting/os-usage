"""Test: toggle switch component — verifies DOM/CSS state changes."""
import re


def _parse_rgb(rgb_str):
    """Parse 'rgb(r, g, b)' into (r, g, b) tuple."""
    m = re.match(r"rgb\((\d+),\s*(\d+),\s*(\d+)\)", rgb_str)
    return tuple(int(x) for x in m.groups()) if m else None


def _get_track_style(page):
    """Get the toggle track's computed background color and knob left position."""
    return page.evaluate("""() => {
        const toggle = document.querySelector('[data-label="Dark mode"]');
        // The toggle container has: <span>label</span> <div>track<div>knob</div></div>
        // Find the track by its 44px width
        const divs = toggle.querySelectorAll('div');
        let track = null;
        for (const d of divs) {
            if (d.style.width === '44px') { track = d; break; }
        }
        if (!track) {
            const topDivs = toggle.querySelectorAll(':scope > div');
            track = topDivs[topDivs.length - 1];
        }
        const knob = track ? track.querySelector('div') : null;
        return {
            trackBg: track ? getComputedStyle(track).backgroundColor : null,
            knobLeft: knob ? knob.style.left : null,
        };
    }""")


def _is_gray(rgb):
    """Check if an RGB tuple is gray-ish (all channels close together, highish values)."""
    r, g, b = rgb
    return max(r, g, b) - min(r, g, b) < 30 and r > 150


def _is_blue(rgb):
    """Check if an RGB tuple is blue-ish (blue > red, blue > 200)."""
    r, g, b = rgb
    return b > 200 and b > r + 50


def test_toggle_starts_off(setup, result):
    page = setup("/test/toggle")
    assert result() == "off"

    # Verify initial CSS state
    style = _get_track_style(page)
    rgb = _parse_rgb(style["trackBg"])
    assert _is_gray(rgb), f"expected gray track, got: {style['trackBg']}"
    assert style["knobLeft"] == "2px", f"expected knob at 2px, got: {style['knobLeft']}"


def test_toggle_click_turns_on(setup, result, vp_offset):
    page = setup("/test/toggle")
    vp = vp_offset()
    page.mouse.click(vp["x"] + 400, vp["y"] + 300)

    assert result() == "on"
    # Wait for CSS transition to complete (150ms transition + margin)
    page.wait_for_timeout(250)

    # Verify CSS state changed to ON
    style = _get_track_style(page)
    rgb = _parse_rgb(style["trackBg"])
    assert _is_blue(rgb), f"expected blue track, got: {style['trackBg']}"
    assert style["knobLeft"] == "22px", f"expected knob at 22px, got: {style['knobLeft']}"


def test_toggle_double_click_returns_off(setup, result, vp_offset):
    page = setup("/test/toggle")
    vp = vp_offset()

    # First click — ON
    page.mouse.click(vp["x"] + 400, vp["y"] + 300)
    assert result() == "on"
    page.wait_for_timeout(250)
    style = _get_track_style(page)
    rgb = _parse_rgb(style["trackBg"])
    assert _is_blue(rgb), f"expected blue track on first click, got: {style['trackBg']}"

    # Second click — OFF
    page.mouse.click(vp["x"] + 400, vp["y"] + 300)
    assert result() == "off"
    page.wait_for_timeout(250)
    style = _get_track_style(page)
    rgb = _parse_rgb(style["trackBg"])
    assert _is_gray(rgb), f"expected gray track after toggle off, got: {style['trackBg']}"
    assert style["knobLeft"] == "2px"
