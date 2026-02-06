"""Test: drag-to-reorder list component — verifies real drag with DOM changes."""


def _get_list_order(page):
    """Read the text content of each list item in DOM order."""
    return page.evaluate("""() => {
        const container = document.getElementById('list-container');
        const items = container.querySelectorAll('button.target');
        // Sort by current top position to get visual order
        const sorted = Array.from(items).sort((a, b) => {
            return parseFloat(a.style.top) - parseFloat(b.style.top);
        });
        return sorted.map(btn => {
            const label = btn.querySelector('.item-label');
            return label ? label.textContent.trim() : btn.textContent.trim();
        });
    }""")


def _get_item_style(page, label):
    """Get computed styles for a specific list item."""
    return page.evaluate("""(label) => {
        const btn = document.querySelector(`[data-label="${label}"]`);
        if (!btn) return null;
        return {
            top: btn.style.top,
            opacity: btn.style.opacity,
            zIndex: btn.style.zIndex,
            pointerEvents: btn.style.pointerEvents,
        };
    }""", label)


def _item_page_center(page, label):
    """Get the page-coordinate center of a list item."""
    return page.evaluate("""(label) => {
        const btn = document.querySelector(`[data-label="${label}"]`);
        if (!btn) return null;
        const r = btn.getBoundingClientRect();
        return { x: r.x + r.width / 2, y: r.y + r.height / 2 };
    }""", label)


def test_reorder_initial_order(setup, result):
    page = setup("/test/reorder")
    assert result() == "idle"

    order = _get_list_order(page)
    assert order == ["Alpha", "Beta", "Gamma", "Delta"], f"unexpected initial order: {order}"


def test_reorder_drag_activates(setup, result):
    """Mousedown on an item should activate drag state and change styles."""
    page = setup("/test/reorder")

    center = _item_page_center(page, "Alpha")
    page.mouse.move(center["x"], center["y"])
    page.mouse.down()
    page.wait_for_timeout(50)

    assert result() == "dragging:Alpha"

    style = _get_item_style(page, "Alpha")
    assert style["opacity"] == "0.85", f"expected 0.85, got {style['opacity']}"
    assert style["zIndex"] == "200", f"expected 200, got {style['zIndex']}"
    assert style["pointerEvents"] == "none", f"expected none, got {style['pointerEvents']}"

    page.mouse.up()


def test_reorder_drag_moves_item(setup, result):
    """During drag, the item's inline top should change as mouse moves."""
    page = setup("/test/reorder")

    before = _get_item_style(page, "Alpha")
    before_top = float(before["top"].replace("px", ""))

    center = _item_page_center(page, "Alpha")
    page.mouse.move(center["x"], center["y"])
    page.mouse.down()
    page.wait_for_timeout(30)

    # Move down 60px
    page.mouse.move(center["x"], center["y"] + 60, steps=5)
    page.wait_for_timeout(50)

    during = _get_item_style(page, "Alpha")
    during_top = float(during["top"].replace("px", ""))
    assert during_top > before_top + 30, \
        f"item should have moved down: top {before_top} -> {during_top}"

    page.mouse.up()


def test_reorder_drag_swap(setup, result):
    """Dragging Alpha past Gamma should swap them in the DOM order."""
    page = setup("/test/reorder")

    assert _get_list_order(page) == ["Alpha", "Beta", "Gamma", "Delta"]

    alpha_center = _item_page_center(page, "Alpha")
    gamma_center = _item_page_center(page, "Gamma")

    # Start drag on Alpha
    page.mouse.move(alpha_center["x"], alpha_center["y"])
    page.mouse.down()
    page.wait_for_timeout(30)

    # Drag past Gamma (move down past Beta and Gamma centers)
    page.mouse.move(alpha_center["x"], gamma_center["y"] + 30, steps=15)
    page.wait_for_timeout(100)

    # Release
    page.mouse.up()
    page.wait_for_timeout(100)

    # DOM order should have changed — Alpha should be after Gamma
    order = _get_list_order(page)
    # Alpha was at 0, dragged past 1 (Beta) and 2 (Gamma), so it should be at position 2
    assert order[0] == "Beta", f"expected Beta first, got: {order}"
    assert order[1] == "Gamma", f"expected Gamma second, got: {order}"
    assert order[2] == "Alpha", f"expected Alpha third, got: {order}"
    assert order[3] == "Delta", f"expected Delta fourth, got: {order}"


def test_reorder_drag_release_snaps(setup, result):
    """After releasing, the dragged item should snap to its grid position."""
    page = setup("/test/reorder")

    alpha_center = _item_page_center(page, "Alpha")

    # Drag Alpha down a bit (not enough to swap)
    page.mouse.move(alpha_center["x"], alpha_center["y"])
    page.mouse.down()
    page.wait_for_timeout(30)
    page.mouse.move(alpha_center["x"], alpha_center["y"] + 20, steps=3)
    page.wait_for_timeout(50)

    # Item should be at a non-grid position during drag
    during = _get_item_style(page, "Alpha")
    during_top = float(during["top"].replace("px", ""))

    page.mouse.up()
    page.wait_for_timeout(200)

    # After release, item should snap to grid position (item_y(0) = 0)
    after = _get_item_style(page, "Alpha")
    after_top = float(after["top"].replace("px", ""))
    assert abs(after_top - 0.0) < 1, f"expected top ~0 after snap, got {after_top}"

    # Styles should revert
    assert after["opacity"] == "1"
    assert after["zIndex"] == "1"
    assert after["pointerEvents"] == "auto"


def test_reorder_multiple_drags(setup, result):
    """Multiple drag operations should chain correctly."""
    page = setup("/test/reorder")

    assert _get_list_order(page) == ["Alpha", "Beta", "Gamma", "Delta"]

    # Drag Alpha down past Beta → [Beta, Alpha, Gamma, Delta]
    alpha_c = _item_page_center(page, "Alpha")
    beta_c = _item_page_center(page, "Beta")
    page.mouse.move(alpha_c["x"], alpha_c["y"])
    page.mouse.down()
    page.wait_for_timeout(30)
    page.mouse.move(alpha_c["x"], beta_c["y"] + 25, steps=10)
    page.wait_for_timeout(50)
    page.mouse.up()
    page.wait_for_timeout(150)

    order = _get_list_order(page)
    assert order[0] == "Beta", f"after first drag: {order}"
    assert order[1] == "Alpha", f"after first drag: {order}"

    # Drag Delta up past Gamma → [Beta, Alpha, Delta, Gamma]
    delta_c = _item_page_center(page, "Delta")
    gamma_c = _item_page_center(page, "Gamma")
    page.mouse.move(delta_c["x"], delta_c["y"])
    page.mouse.down()
    page.wait_for_timeout(30)
    page.mouse.move(delta_c["x"], gamma_c["y"] - 25, steps=10)
    page.wait_for_timeout(50)
    page.mouse.up()
    page.wait_for_timeout(150)

    order = _get_list_order(page)
    assert order[2] == "Delta", f"after second drag: {order}"
    assert order[3] == "Gamma", f"after second drag: {order}"
