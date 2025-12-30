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
            class: "min-h-screen",
            Navbar {}

            main {
                class: "container py-6",
                div {
                    class: "p-4",
                    div {
                        class: "card flex flex-col items-center justify-center h-screen",
                        h1 {
                            class: "text-4xl font-bold mb-10",
                            "Welcome to your Dashboard"
                        }

                        div {
                            class: "text-center",
                            p {
                                class: "text-lg text-muted mb-4",
                                "Your current counter value is:"
                            }
                            p {
                                class: "text-4xl font-extrabold text-primary mb-10",
                                "{counter}"
                            }
                            button {
                                onclick: increment,
                                class: "btn btn-primary",
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
