use dioxus::prelude::*;
use crate::components::back_button::BackButton;
use crate::components::dark_mode_toggle::DarkModeToggle;

#[component]
pub fn AuthLayout(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "flex min-h-screen flex-col items-center justify-center py-12 px-4 sm:px-6 lg:px-8 bg-gray-50 dark:bg-neutral-900",
            div {
                class: "absolute top-4 right-4",
                DarkModeToggle {}
            }
            div {
                class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 {
                    class: "mt-6 text-center text-3xl font-extrabold text-gray-900 dark:text-white",
                    "{title}"
                }
            }
            div {
                class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div {
                    class: "bg-white dark:bg-neutral-800 py-8 px-4 shadow sm:rounded-lg sm:px-10 border dark:border-neutral-700",
                    div {
                        {children}
                    }
                    div {
                        class: "mt-6",
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
