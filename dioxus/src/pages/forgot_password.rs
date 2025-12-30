use dioxus::prelude::*;
use crate::api::{request_password_reset, ResetRequestData};
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;
use crate::components::link_button::LinkButton;

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
            error_msg.set(Some("Please enter your email".to_string()));
            return;
        }

        // Simple email validation
        if !identifier().contains('@') {
            error_msg.set(Some("Please enter a valid email address".to_string()));
            return;
        }

        let data = ResetRequestData {
            identifier: identifier(),
        };

        match request_password_reset(data).await {
            Ok(_) => {
                is_submitted.set(true);
                message.set(Some("If an account exists with that email, a password reset link has been sent.".to_string()));
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
                    class: "alert alert-success",
                    div {
                        class: "flex",
                        div {
                            class: "flex-shrink-0",
                            svg {
                                class: "h-5 w-5",
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
                                class: "text-sm font-medium",
                                "Request received"
                            }
                            div {
                                class: "mt-2 text-sm",
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
                                        class: "btn btn-secondary text-sm",
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
                        class: "mb-4 text-sm text-muted",
                        "Enter your email address and we'll send you a link to reset your password."
                    }

                    TextInput {
                        label: "Email",
                        value: identifier,
                        r#type: "email",
                        required: true,
                    }

                    if let Some(msg) = error_msg() {
                        div {
                            class: "alert alert-error",
                            div {
                                class: "flex",
                                div {
                                    class: "ml-3",
                                    h3 {
                                        class: "text-sm font-medium",
                                        "Error"
                                    }
                                    div {
                                        class: "mt-2 text-sm",
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
                                class: "w-full border-t border-gray-300",
                            }
                        }
                        div {
                            class: "relative flex justify-center text-sm",
                            span {
                                class: "px-2 bg-white text-muted",
                                "Or"
                            }
                        }
                    }
                    div {
                        class: "mt-6",
                        LinkButton {
                            to: crate::Route::Login {},
                            "Back to login"
                        }
                    }
                }
            }
        }
    }
}
