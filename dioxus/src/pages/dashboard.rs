use dioxus::prelude::*;
use crate::api::{get_counter, set_counter};
use crate::components::navbar::Navbar;

#[component]
pub fn Dashboard() -> Element {
    let mut counter = use_signal(|| 0i64);
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let nav = use_navigator();

    // Fetch initial counter value
    use_effect(move || {
        spawn(async move {
            match get_counter().await {
                Ok(val) => counter.set(val),
                Err(e) => {
                    // If unauthorized, redirect to login
                    // checking error string is brittle but works for now
                    if e.to_string().contains("No session") || e.to_string().contains("Auth error") {
                         nav.push(crate::Route::Login {});
                    } else {
                         error_msg.set(Some(e.to_string()));
                    }
                }
            }
        });
    });

    let increment = move |_| async move {
        let new_val = counter() + 1;
        match set_counter(new_val).await {
            Ok(_) => counter.set(new_val),
            Err(e) => error_msg.set(Some(e.to_string())),
        }
    };

    rsx! {
        div {
            class: "min-h-screen bg-neutral-100 dark:bg-black",
            Navbar {}

            main {
                class: "max-w-7xl mx-auto py-6 sm:px-6 lg:px-8",
                div {
                    class: "px-4 py-6 sm:px-0",
                    div {
                        class: "border-4 border-dashed border-neutral-200 dark:border-neutral-800 rounded-lg h-96 flex flex-col items-center justify-center",
                        h1 {
                            class: "text-3xl font-bold text-neutral-900 dark:text-white mb-8",
                            "Welcome to your Dashboard"
                        }

                        div {
                            class: "text-center",
                            p {
                                class: "text-lg text-neutral-500 dark:text-neutral-400 mb-4",
                                "Your current counter value is:"
                            }
                            p {
                                class: "text-6xl font-extrabold text-indigo-600 dark:text-indigo-400 mb-8",
                                "{counter}"
                            }
                            button {
                                onclick: increment,
                                class: "inline-flex items-center px-6 py-3 border border-transparent text-base font-medium rounded-md text-indigo-700 bg-indigo-100 hover:bg-indigo-200 dark:bg-neutral-800 dark:text-indigo-400 dark:hover:bg-neutral-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                                "Increment Counter"
                            }
                        }

                        if let Some(msg) = error_msg() {
                            div {
                                class: "mt-4 text-red-600 dark:text-red-400",
                                "{msg}"
                            }
                        }
                    }
                }
            }
        }
    }
}
