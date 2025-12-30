use dioxus::prelude::*;
use crate::components::back_button::BackButton;
use crate::components::dark_mode_toggle::DarkModeToggle;

#[component]
pub fn AuthLayout(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center min-h-screen p-4",
            div {
                class: "absolute top-4 right-4",
                DarkModeToggle {}
            }
            div {
                class: "w-full max-w-md",
                h2 {
                    class: "text-center text-4xl font-extrabold mb-10",
                    "{title}"
                }
            }
            div {
                class: "w-full max-w-md",
                div {
                    class: "card",
                    div {
                        {children}
                    }
                    div {
                        class: "mt-4",
                        BackButton {
                            label: "Back to Home",
                            to: crate::Route::Home {}
                        }
                    }
                }
            }
        }
    }
}
