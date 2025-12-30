use dioxus::prelude::*;

#[component]
pub fn DarkModeToggle() -> Element {
    let mut is_dark = use_signal(|| false);

    // Initialize state
    use_effect(move || {
        // Access window and document directly using web-sys
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let root = document.document_element().expect("document should have a root element");
        let local_storage = window.local_storage().ok().flatten();

        let dark_mode_preferred = if let Some(storage) = &local_storage {
            match storage.get_item("theme") {
                Ok(Some(theme)) => theme == "dark",
                _ => {
                    // Check system preference
                    match window.match_media("(prefers-color-scheme: dark)") {
                        Ok(Some(media_query)) => {
                            let media_query: web_sys::MediaQueryList = media_query;
                            media_query.matches()
                        },
                        _ => false,
                    }
                }
            }
        } else {
            false
        };

        if dark_mode_preferred {
            let _ = root.class_list().add_1("dark");
            is_dark.set(true);
        } else {
            let _ = root.class_list().remove_1("dark");
            is_dark.set(false);
        }
    });

    let toggle_theme = move |_| {
        let new_state = !is_dark();
        is_dark.set(new_state);

        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let root = document.document_element().expect("document should have a root element");
        let local_storage = window.local_storage().ok().flatten();

        if new_state {
            let _ = root.class_list().add_1("dark");
            if let Some(storage) = local_storage {
                let _ = storage.set_item("theme", "dark");
            }
        } else {
            let _ = root.class_list().remove_1("dark");
            if let Some(storage) = local_storage {
                let _ = storage.set_item("theme", "light");
            }
        }
    };

    rsx! {
        button {
            onclick: toggle_theme,
            class: "btn-icon",
            aria_label: "Toggle dark mode",
            if is_dark() {
                // Sun icon for dark mode (switch to light)
                svg {
                    class: "h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z"
                    }
                }
            } else {
                // Moon icon for light mode (switch to dark)
                svg {
                    class: "h-6 w-6",
                    fill: "none",
                    view_box: "0 0 24 24",
                    stroke: "currentColor",
                    path {
                        stroke_linecap: "round",
                        stroke_linejoin: "round",
                        stroke_width: "2",
                        d: "M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z"
                    }
                }
            }
        }
    }
}
