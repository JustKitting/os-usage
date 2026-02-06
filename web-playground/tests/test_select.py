"""Test: dropdown select component — verifies DOM/CSS state changes."""
from playwright.sync_api import Page


def test_dropdown_starts_none(setup, result):
    page = setup("/test/dropdown")
    assert result() == "none"

    # Verify trigger displays placeholder text
    trigger = page.locator("[data-label='Choose...']")
    assert trigger.count() == 1
    assert trigger.text_content().strip() == "Choose..."

    # Panel should not be visible
    assert page.locator("[data-label='Apple']").count() == 0


def test_dropdown_opens_panel(setup, result, vp_offset, page: Page):
    setup("/test/dropdown")
    vp = vp_offset()

    # Click trigger to open
    page.mouse.click(vp["x"] + 400, vp["y"] + 288)

    # All three options should now be visible
    page.wait_for_selector("[data-label='Apple']", state="visible", timeout=3000)
    assert page.locator("[data-label='Apple']").is_visible()
    assert page.locator("[data-label='Banana']").is_visible()
    assert page.locator("[data-label='Cherry']").is_visible()


def test_dropdown_select_updates_trigger(setup, result, vp_offset, page: Page):
    setup("/test/dropdown")
    vp = vp_offset()

    # Open dropdown
    page.mouse.click(vp["x"] + 400, vp["y"] + 288)
    page.wait_for_selector("[data-label='Banana']", state="visible", timeout=3000)

    # Click Banana
    page.locator("[data-label='Banana']").click()

    # Result should update
    assert result() == "selected:Banana"

    # Trigger text should now show "Banana" (not "Choose...")
    trigger = page.locator("[data-label='Banana']")
    # The trigger's data-label should have updated from "Choose..." to "Banana"
    # and the trigger text should display "Banana"
    banana_els = page.locator("[data-label='Banana']")
    # After selecting, the panel closes and the trigger shows "Banana"
    assert banana_els.count() >= 1
    # Verify the trigger (not option) shows Banana
    trigger_text = page.evaluate("""() => {
        const vp = document.getElementById('viewport');
        // The trigger is a div inside the custom select, with the selected text
        const trigger = vp.querySelector('[data-label="Banana"]');
        return trigger ? trigger.textContent.trim() : null;
    }""")
    assert trigger_text == "Banana", f"expected trigger to show 'Banana', got: {trigger_text}"

    # Panel should be closed (options not visible)
    assert page.locator("[data-label='Apple']").count() == 0
