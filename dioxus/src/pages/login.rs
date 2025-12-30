use dioxus::prelude::*;
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;
use crate::components::link_button::LinkButton;
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
                                h3 { class: "text-sm font-medium text-red-800", "Login Failed" }
                                div {
                                    class: "mt-2 text-sm text-red-700",
                                    p { "{msg}" }
                                }
                            }
                        }
                    }
                }

                div {
                    class: "flex items-center justify-end mb-6",
                    div {
                        class: "text-sm",
                        Link {
                            to: crate::Route::ForgotPassword {},
                            class: "font-medium text-primary-600 hover:text-primary-500",
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
                            class: "w-full border-t border-gray-300"
                        }
                    }
                    div {
                        class: "relative flex justify-center text-sm",
                        span {
                            class: "px-2 bg-white text-gray-500",
                            "Or"
                        }
                    }
                }
                div {
                    class: "mt-6",
                    LinkButton {
                        to: crate::Route::Register {},
                        "Create new account"
                    }
                }
            }
        }
    }
}
