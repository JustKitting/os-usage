"""Test: drag and drop component — verifies element actually moves in DOM during drag."""


def _file_pos(page):
    """Read the file icon's position from its inline style (left/top)."""
    return page.evaluate("""() => {
        const file = document.querySelector('[data-label="test.txt"]');
        if (!file) return null;
        return {
            left: parseFloat(file.style.left),
            top: parseFloat(file.style.top),
            opacity: file.style.opacity,
            zIndex: file.style.zIndex,
            pointerEvents: file.style.pointerEvents,
        };
    }""")


def _drop_zone_bg(page):
    """Read the drop zone's computed background color."""
    return page.evaluate("""() => {
        const dz = document.querySelector('[data-label="Drop Zone"]');
        return getComputedStyle(dz).backgroundColor;
    }""")


def _drag_overlay_exists(page):
    """Check if the drag overlay (cursor: grabbing) exists in DOM."""
    return page.evaluate("""() => {
        const vp = document.getElementById('viewport');
        const overlay = vp.querySelector('[style*="cursor: grabbing"]');
        return overlay !== null;
    }""")


def test_drag_starts_at_origin(setup, result):
    """File should be at its initial position (100, 250)."""
    page = setup("/test/drag")
    assert result() == "idle"

    pos = _file_pos(page)
    assert abs(pos["left"] - 100) < 1, f"expected left ~100, got {pos['left']}"
    assert abs(pos["top"] - 250) < 1, f"expected top ~250, got {pos['top']}"
    assert pos["opacity"] == "1"
    assert pos["zIndex"] == "10"
    assert pos["pointerEvents"] == "auto"
    assert not _drag_overlay_exists(page)


def test_drag_mousedown_activates_drag(setup, result, vp_offset):
    """Mousedown on the file should activate drag state and change styles."""
    page = setup("/test/drag")
    vp = vp_offset()

    # Mousedown on file center (100+40=140, 250+48=298)
    page.mouse.move(vp["x"] + 140, vp["y"] + 298)
    page.mouse.down()
    page.wait_for_timeout(50)

    # Result should show dragging
    assert result() == "dragging"

    # File style should change: opacity, z-index, pointer-events
    pos = _file_pos(page)
    assert pos["opacity"] == "0.85", f"expected opacity 0.85 during drag, got {pos['opacity']}"
    assert pos["zIndex"] == "200", f"expected z-index 200 during drag, got {pos['zIndex']}"
    assert pos["pointerEvents"] == "none", f"expected pointer-events none during drag, got {pos['pointerEvents']}"

    # Drag overlay should exist
    assert _drag_overlay_exists(page), "drag overlay should exist during drag"

    page.mouse.up()


def test_drag_element_moves_with_mouse(setup, result, vp_offset):
    """During drag, the file's inline left/top should update as mouse moves."""
    page = setup("/test/drag")
    vp = vp_offset()

    before = _file_pos(page)

    # Start drag
    page.mouse.move(vp["x"] + 140, vp["y"] + 298)
    page.mouse.down()
    page.wait_for_timeout(30)

    # Move mouse 200px to the right
    page.mouse.move(vp["x"] + 340, vp["y"] + 298, steps=5)
    page.wait_for_timeout(50)

    during = _file_pos(page)
    assert during["left"] > before["left"] + 100, \
        f"file should have moved right: left {before['left']} -> {during['left']}"

    page.mouse.up()


def test_drag_to_drop_zone_changes_position(setup, result, vp_offset):
    """Dragging file over drop zone should change drop zone background and complete on release."""
    page = setup("/test/drag")
    vp = vp_offset()

    # Start drag from file center
    page.mouse.move(vp["x"] + 140, vp["y"] + 298)
    page.mouse.down()
    page.wait_for_timeout(30)

    # Drag to drop zone center (500+100=600, 200+80=280)
    page.mouse.move(vp["x"] + 600, vp["y"] + 280, steps=10)
    page.wait_for_timeout(50)

    # File should be near drop zone
    pos = _file_pos(page)
    assert pos["left"] > 400, f"file should be over drop zone, left={pos['left']}"

    # Drop zone should have hover background (#eef2ff = rgb(238, 242, 255))
    dz_bg = _drop_zone_bg(page)
    assert dz_bg != "rgb(255, 255, 255)", f"drop zone should highlight on hover, got {dz_bg}"

    # Release
    page.mouse.up()
    assert result() == "dropped"

    # After drop, drag overlay should be gone
    assert not _drag_overlay_exists(page), "overlay should be removed after drop"


def test_drag_miss_snaps_back(setup, result, vp_offset):
    """Dropping outside the zone should snap the file back to its origin."""
    page = setup("/test/drag")
    vp = vp_offset()

    before = _file_pos(page)

    # Drag to an empty area (not the drop zone)
    page.mouse.move(vp["x"] + 140, vp["y"] + 298)
    page.mouse.down()
    page.wait_for_timeout(30)
    page.mouse.move(vp["x"] + 300, vp["y"] + 100, steps=10)
    page.wait_for_timeout(50)

    # File should have moved during drag
    during = _file_pos(page)
    assert during["left"] != before["left"] or during["top"] != before["top"], \
        "file should have moved during drag"

    # Release outside drop zone
    page.mouse.up()

    assert result() == "cancelled"

    # File should snap back to original position
    after = _file_pos(page)
    assert abs(after["left"] - before["left"]) < 2, \
        f"file should snap back: left {before['left']} -> {after['left']}"
    assert abs(after["top"] - before["top"]) < 2, \
        f"file should snap back: top {before['top']} -> {after['top']}"

    # Styles should revert
    assert after["opacity"] == "1"
    assert after["zIndex"] == "10"
    assert after["pointerEvents"] == "auto"
