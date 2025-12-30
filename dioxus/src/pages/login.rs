use dioxus::prelude::*;
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;
use crate::api::{login, LoginData};

#[component]
pub fn Login() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut error_message = use_signal::<Option<String>>(|| None);
    let nav = use_navigator();

    let handle_submit = move |_| async move {
        error_message.set(None);
        let data = LoginData {
            username: username(),
            password: password(),
        };

        match login(data).await {
            Ok(_response) => {
                nav.push(crate::Route::Home {}); // Redirect to Dashboard eventually
            }
            Err(e) => {
                error_message.set(Some(e.to_string()));
            }
        }
    };

    rsx! {
        AuthLayout {
            title: "Sign in to your account".to_string(),
            form {
                onsubmit: handle_submit,
                TextInput {
                    label: "Username",
                    value: username,
                    required: true,
                }
                TextInput {
                    label: "Password",
                    value: password,
                    r#type: "password",
                    required: true,
                }

                if let Some(msg) = error_message() {
                    div {
                        class: "rounded-md bg-red-50 p-4 mb-4",
                        div {
                            class: "flex",
                            div {
                                class: "ml-3",
                                h3 {
                                    class: "text-sm font-medium text-red-800",
                                    "Login Failed"
                                }
                                div {
                                    class: "mt-2 text-sm text-red-700",
                                    p { "{msg}" }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center justify-between mb-4",
                    div {
                        class: "text-sm",
                        Link {
                            to: crate::Route::ForgotPassword {}, // Link to forgot password route
                            class: "font-medium text-indigo-600 hover:text-indigo-500",
                            "Forgot your password?"
                        }
                    }
                }

                div {
                    Button {
                        r#type: "submit",
                        "Sign in"
                    }
                }
            }
            div {
                class: "mt-6",
                div {
                    class: "relative",
                    div {
                        class: "absolute inset-0 flex items-center",
                        div {
                            class: "w-full border-t border-neutral-300 dark:border-neutral-700",
                        }
                    }
                    div {
                        class: "relative flex justify-center text-sm",
                        span {
                            class: "px-2 bg-white dark:bg-neutral-900 text-neutral-500 dark:text-neutral-400",
                            "Or continue with"
                        }
                    }
                }
                div {
                    class: "mt-6 grid grid-cols-1 gap-3",
                    Link {
                        to: crate::Route::Register {},
                        class: "w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-indigo-600 bg-indigo-50 hover:bg-indigo-100 dark:bg-neutral-800 dark:text-indigo-400 dark:hover:bg-neutral-700",
                        "Create new account"
                    }
                }
            }
        }
    }
}
