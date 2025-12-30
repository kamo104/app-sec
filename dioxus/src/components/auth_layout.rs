use dioxus::prelude::*;
use crate::components::back_button::BackButton;

#[component]
pub fn AuthLayout(title: String, children: Element) -> Element {
    rsx! {
        div {
            class: "min-h-screen flex flex-col justify-center items-center py-12 sm:px-6 lg:px-8",
            div {
                class: "sm:mx-auto sm:w-full sm:max-w-md",
                h2 {
                    class: "mt-6 text-center text-3xl font-extrabold text-[var(--text)]",
                    "{title}"
                }
            }
            div {
                class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div {
                    class: "bg-[var(--surface)] py-8 px-4 shadow-2xl sm:rounded-lg sm:px-10 border border-[var(--border)]",
                    {children}
                }
                BackButton {
                    label: "Back to Home",
                    to: crate::Route::Home {}
                }
            }
        }
    }
}
