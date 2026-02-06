"""Test: button click component — verifies DOM/CSS state changes."""


def test_button_starts_idle(setup, result):
    page = setup("/test/button")
    assert result() == "idle"

    # Verify initial DOM state
    btn = page.locator("button.target")
    assert btn.text_content().strip() == "Click me"
    bg = btn.evaluate("el => getComputedStyle(el).backgroundColor")
    assert bg == "rgb(59, 130, 246)", f"expected blue bg, got: {bg}"


def test_button_click_changes_text_and_color(setup, result, vp_offset):
    page = setup("/test/button")
    vp = vp_offset()

    # Before click
    btn = page.locator("button.target")
    assert btn.text_content().strip() == "Click me"

    # Click button
    page.mouse.click(vp["x"] + 400, vp["y"] + 290)

    # After click — text should change
    assert btn.text_content().strip() == "Clicked!"
    assert result() == "clicked"

    # Background should change to green (#22c55e = rgb(34, 197, 94))
    bg = btn.evaluate("el => getComputedStyle(el).backgroundColor")
    assert bg == "rgb(34, 197, 94)", f"expected green bg, got: {bg}"

    # Cursor should change to default
    cursor = btn.evaluate("el => getComputedStyle(el).cursor")
    assert cursor == "default", f"expected default cursor, got: {cursor}"

    # data-label should update
    assert btn.get_attribute("data-label") == "Clicked!"


def test_button_click_fires_events(setup, vp_offset, console_events):
    page = setup("/test/button")
    vp = vp_offset()
    page.mouse.click(vp["x"] + 400, vp["y"] + 290)

    event_types = [e["event"] for e in console_events]
    assert "mousedown" in event_types
    assert "mouseup" in event_types
    assert "click" in event_types
