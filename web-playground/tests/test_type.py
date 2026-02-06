"""Test: text input component — verifies DOM/CSS state changes."""


def test_input_starts_empty(setup, result):
    page = setup("/test/text-input")
    assert result() == "empty"

    # Verify input element is actually empty
    inp = page.locator("input.target")
    assert inp.input_value() == ""
    assert inp.get_attribute("placeholder") == "Type here..."


def test_input_typing_updates_dom_value(setup, result, vp_offset):
    page = setup("/test/text-input")
    vp = vp_offset()

    # Click to focus
    page.mouse.click(vp["x"] + 400, vp["y"] + 298)
    page.keyboard.type("hel")

    # Verify the input element's actual value
    inp = page.locator("input.target")
    assert inp.input_value() == "hel", f"expected 'hel', got: {inp.input_value()}"

    # Result should reflect typing
    assert result().startswith("typing:")
    assert "hel" in result()


def test_input_correct_value_matches_dom(setup, result, vp_offset):
    page = setup("/test/text-input")
    vp = vp_offset()

    page.mouse.click(vp["x"] + 400, vp["y"] + 298)
    page.keyboard.type("hello")

    # Verify input element value matches
    inp = page.locator("input.target")
    assert inp.input_value() == "hello"

    # Result should be correct
    assert result() == "correct"
