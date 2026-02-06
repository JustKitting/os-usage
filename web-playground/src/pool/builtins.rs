//! Built-in design snippets - starter pool of element variants

use super::kind::ElementKind;
use super::snippet::DesignSnippet;

/// Seed the pool with diverse built-in designs
pub fn builtin_snippets() -> Vec<DesignSnippet> {
    let mut pool = Vec::new();

    // --- Buttons (normal → pressed/darker) ---

    pool.push(DesignSnippet::new(
        "btn-flat-blue",
        ElementKind::Button,
        "flat blue button",
        r#"<button style="
            padding: 10px 24px;
            background: #3b82f6;
            color: white;
            border: none;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Submit</button>"#,
        r#"<button style="
            padding: 10px 24px;
            background: #1d4ed8;
            color: white;
            border: none;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
            box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);
        ">Submit</button>"#,
        100.0, 40.0,
    ));

    pool.push(DesignSnippet::new(
        "btn-outline-light",
        ElementKind::Button,
        "outline light button",
        r#"<button style="
            padding: 10px 24px;
            background: transparent;
            color: #e5e7eb;
            border: 2px solid #e5e7eb;
            border-radius: 4px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Cancel</button>"#,
        r#"<button style="
            padding: 10px 24px;
            background: #e5e7eb;
            color: #1a1a2e;
            border: 2px solid #e5e7eb;
            border-radius: 4px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Cancel</button>"#,
        100.0, 40.0,
    ));

    pool.push(DesignSnippet::new(
        "btn-gradient-purple",
        ElementKind::Button,
        "gradient purple button",
        r#"<button style="
            padding: 12px 32px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-family: system-ui, sans-serif;
            font-weight: 600;
            cursor: pointer;
            box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
        ">Get Started</button>"#,
        r#"<button style="
            padding: 12px 32px;
            background: linear-gradient(135deg, #4f5bd5 0%, #5e3a8a 100%);
            color: white;
            border: none;
            border-radius: 8px;
            font-size: 16px;
            font-family: system-ui, sans-serif;
            font-weight: 600;
            cursor: pointer;
            box-shadow: 0 2px 8px rgba(102, 126, 234, 0.6);
        ">Get Started</button>"#,
        140.0, 48.0,
    ));

    pool.push(DesignSnippet::new(
        "btn-pill-green",
        ElementKind::Button,
        "pill green button",
        r#"<button style="
            padding: 8px 20px;
            background: #22c55e;
            color: white;
            border: none;
            border-radius: 9999px;
            font-size: 13px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Confirm</button>"#,
        r#"<button style="
            padding: 8px 20px;
            background: #16a34a;
            color: white;
            border: none;
            border-radius: 9999px;
            font-size: 13px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
            box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);
        ">Confirm</button>"#,
        100.0, 36.0,
    ));

    pool.push(DesignSnippet::new(
        "btn-danger-red",
        ElementKind::Button,
        "danger red button",
        r#"<button style="
            padding: 10px 24px;
            background: #ef4444;
            color: white;
            border: none;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Delete</button>"#,
        r#"<button style="
            padding: 10px 24px;
            background: #b91c1c;
            color: white;
            border: none;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
            box-shadow: inset 0 2px 4px rgba(0,0,0,0.2);
        ">Delete</button>"#,
        100.0, 40.0,
    ));

    // --- Inputs (browser handles focus natively) ---

    pool.push(DesignSnippet::static_new(
        "input-basic",
        ElementKind::Input,
        "basic text input",
        r#"<input type="text" placeholder="Enter text..." style="
            padding: 10px 14px;
            border: 1px solid #d1d5db;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            outline: none;
            width: 220px;
            background: white;
            color: #111;
        " />"#,
        240.0, 40.0,
    ));

    pool.push(DesignSnippet::static_new(
        "input-underline",
        ElementKind::Input,
        "underline input",
        r#"<input type="text" placeholder="Type here..." style="
            padding: 10px 4px;
            border: none;
            border-bottom: 2px solid #6366f1;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            outline: none;
            width: 200px;
            background: transparent;
            color: #111;
        " />"#,
        220.0, 40.0,
    ));

    pool.push(DesignSnippet::static_new(
        "input-search",
        ElementKind::Input,
        "rounded search input",
        r#"<input type="text" placeholder="Search..." style="
            padding: 10px 16px;
            border: 1px solid #e5e7eb;
            border-radius: 9999px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            outline: none;
            width: 240px;
            background: #f9fafb;
            color: #111;
        " />"#,
        260.0, 42.0,
    ));

    // --- Checkboxes (unchecked → checked) ---

    pool.push(DesignSnippet::new(
        "checkbox-basic",
        ElementKind::Checkbox,
        "basic checkbox",
        r#"<label style="
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            color: #374151;
            cursor: pointer;
        ">
            <div style="
                width: 18px; height: 18px;
                border: 2px solid #d1d5db;
                border-radius: 4px;
                background: white;
            "></div>
            Accept terms
        </label>"#,
        r#"<label style="
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            color: #374151;
            cursor: pointer;
        ">
            <div style="
                width: 18px; height: 18px;
                border: 2px solid #3b82f6;
                border-radius: 4px;
                background: #3b82f6;
                display: flex;
                align-items: center;
                justify-content: center;
                color: white;
                font-size: 12px;
                font-weight: bold;
            ">&#10003;</div>
            Accept terms
        </label>"#,
        140.0, 24.0,
    ));

    // --- Toggles (off → on) ---

    pool.push(DesignSnippet::new(
        "toggle-ios",
        ElementKind::Toggle,
        "iOS-style toggle",
        // OFF state
        r#"<label style="
            display: flex;
            align-items: center;
            gap: 10px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            color: #374151;
            cursor: pointer;
        ">
            <div style="
                width: 44px;
                height: 24px;
                background: #d1d5db;
                border-radius: 12px;
                position: relative;
            ">
                <div style="
                    width: 20px;
                    height: 20px;
                    background: white;
                    border-radius: 50%;
                    position: absolute;
                    top: 2px;
                    left: 2px;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
                "></div>
            </div>
            Dark mode
        </label>"#,
        // ON state
        r#"<label style="
            display: flex;
            align-items: center;
            gap: 10px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            color: #374151;
            cursor: pointer;
        ">
            <div style="
                width: 44px;
                height: 24px;
                background: #3b82f6;
                border-radius: 12px;
                position: relative;
            ">
                <div style="
                    width: 20px;
                    height: 20px;
                    background: white;
                    border-radius: 50%;
                    position: absolute;
                    top: 2px;
                    right: 2px;
                    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
                "></div>
            </div>
            Dark mode
        </label>"#,
        140.0, 28.0,
    ));

    // --- Links (normal → visited color) ---

    pool.push(DesignSnippet::new(
        "link-basic",
        ElementKind::Link,
        "basic underlined link",
        r##"<a href="#" style="
            color: #3b82f6;
            text-decoration: underline;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Learn more</a>"##,
        r##"<a href="#" style="
            color: #7c3aed;
            text-decoration: underline;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            cursor: pointer;
        ">Learn more</a>"##,
        90.0, 20.0,
    ));

    // --- Dropdowns (use static_new - native select handles its own state) ---

    pool.push(DesignSnippet::static_new(
        "dropdown-basic",
        ElementKind::Dropdown,
        "basic select dropdown",
        r#"<select style="
            padding: 10px 32px 10px 14px;
            border: 1px solid #d1d5db;
            border-radius: 6px;
            font-size: 14px;
            font-family: system-ui, sans-serif;
            background: white;
            color: #111;
            appearance: none;
            background-image: url('data:image/svg+xml;utf8,<svg xmlns=%22http://www.w3.org/2000/svg%22 width=%2212%22 height=%2212%22 viewBox=%220 0 24 24%22 fill=%22none%22 stroke=%22%236b7280%22 stroke-width=%222%22><polyline points=%226 9 12 15 18 9%22/></svg>');
            background-repeat: no-repeat;
            background-position: right 10px center;
            cursor: pointer;
            min-width: 160px;
        ">
            <option>Select option</option>
            <option>Option A</option>
            <option>Option B</option>
            <option>Option C</option>
        </select>"#,
        180.0, 42.0,
    ));

    pool
}
