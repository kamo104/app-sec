use dioxus::prelude::*;
use crate::api::logout;

#[component]
pub fn Navbar() -> Element {
    let nav = use_navigator();

    let handle_logout = move |_| async move {
        let _ = logout().await;
        nav.push(crate::Route::Login {});
    };

    rsx! {
        nav {
            class: "bg-white dark:bg-neutral-900 shadow border-b border-neutral-200 dark:border-neutral-800",
            div {
                class: "max-w-7xl mx-auto px-4 sm:px-6 lg:px-8",
                div {
                    class: "flex justify-between h-16",
                    div {
                        class: "flex",
                        div {
                            class: "flex-shrink-0 flex items-center",
                            span {
                                class: "font-bold text-xl text-indigo-600 dark:text-indigo-400",
                                "MemeShark"
                            }
                        }
                    }
                    div {
                        class: "flex items-center",
                        button {
                            onclick: handle_logout,
                            class: "ml-3 inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 dark:bg-neutral-800 dark:hover:bg-neutral-700 dark:text-neutral-200 dark:border-neutral-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                            "Logout"
                        }
                    }
                }
            }
        }
    }
}
