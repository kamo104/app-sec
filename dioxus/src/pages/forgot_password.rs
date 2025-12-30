use dioxus::prelude::*;
use crate::api::{request_password_reset, ResetRequestData};
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;
use crate::components::back_button::BackButton;

#[component]
pub fn ForgotPassword() -> Element {
    let mut identifier = use_signal(|| String::new());
    let mut message = use_signal::<Option<String>>(|| None);
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut is_submitted = use_signal(|| false);

    let handle_submit = move |_| async move {
        error_msg.set(None);
        message.set(None);

        if identifier().is_empty() {
            error_msg.set(Some("Please enter your email or username".to_string()));
            return;
        }

        let data = ResetRequestData {
            identifier: identifier(),
        };

        match request_password_reset(data).await {
            Ok(_) => {
                is_submitted.set(true);
                message.set(Some("If an account exists with that email/username, a password reset link has been sent.".to_string()));
            },
            Err(e) => {
                // For security reasons, we might want to show the same success message
                // but for debugging let's log it and show a generic error
                println!("Error requesting password reset: {}", e);
                error_msg.set(Some("An error occurred. Please try again later.".to_string()));
            }
        }
    };

    rsx! {
        AuthLayout {
            title: "Reset your password".to_string(),
            if is_submitted() {
                div {
                    class: "rounded-md bg-green-50 p-4",
                    div {
                        class: "flex",
                        div {
                            class: "flex-shrink-0",
                            svg {
                                class: "h-5 w-5 text-green-400",
                                view_box: "0 0 20 20",
                                fill: "currentColor",
                                path {
                                    fill_rule: "evenodd",
                                    d: "M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z",
                                    clip_rule: "evenodd"
                                }
                            }
                        }
                        div {
                            class: "ml-3",
                            h3 {
                                class: "text-sm font-medium text-green-800",
                                "Request received"
                            }
                            div {
                                class: "mt-2 text-sm text-green-700",
                                p {
                                    "{message.as_ref().unwrap()}"
                                }
                            }
                            div {
                                class: "mt-4",
                                div {
                                    class: "-mx-2 -my-1.5 flex",
                                    Link {
                                        to: crate::Route::Login {},
                                        class: "bg-green-50 px-2 py-1.5 rounded-md text-sm font-medium text-green-800 hover:bg-green-100 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-offset-green-50 focus:ring-green-600",
                                        "Return to login"
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                form {
                    onsubmit: handle_submit,
                    div {
                        class: "mb-4 text-sm text-gray-600",
                        "Enter your email address or username and we'll send you a link to reset your password."
                    }

                    TextInput {
                        label: "Email or Username",
                        value: identifier,
                        required: true,
                    }

                    if let Some(msg) = error_msg() {
                        div {
                            class: "rounded-md bg-red-50 p-4 mb-4",
                            div {
                                class: "flex",
                                div {
                                    class: "ml-3",
                                    h3 {
                                        class: "text-sm font-medium text-red-800",
                                        "Error"
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
                        Button {
                            r#type: "submit",
                            "Send reset link"
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
                                "Or"
                            }
                        }
                    }
                    BackButton {
                        label: "Back to login",
                        to: crate::Route::Login {}
                    }
                }
            }
        }
    }
}
