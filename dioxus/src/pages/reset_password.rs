use dioxus::prelude::*;
use crate::api::{reset_password, ResetPasswordData};
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;

#[component]
pub fn ResetPassword() -> Element {
    let route = use_route::<crate::Route>();
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());
    let mut error_msg = use_signal::<Option<String>>(|| None);
    let mut success = use_signal(|| false);

    // Extract token from query parameters
    let token = if let crate::Route::ResetPassword { token } = route {
        token
    } else {
        String::new()
    };

    let handle_submit = move |_| {
        let token = token.clone();
        async move {
            error_msg.set(None);

            if token.is_empty() {
                error_msg.set(Some("Invalid or missing reset token".to_string()));
                return;
            }

            if password().len() < 8 {
                error_msg.set(Some("Password must be at least 8 characters long".to_string()));
                return;
            }

            if password() != confirm_password() {
                error_msg.set(Some("Passwords do not match".to_string()));
                return;
            }

            let data = ResetPasswordData {
                token,
                new_password: password(),
            };

            match reset_password(data).await {
                Ok(_) => {
                    success.set(true);
                },
                Err(e) => {
                    error_msg.set(Some(e.to_string()));
                }
            }
        }
    };

    rsx! {
        AuthLayout {
            title: "Set new password".to_string(),
            if success() {
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
                                "Password reset successful"
                            }
                            div {
                                class: "mt-2 text-sm",
                                p {
                                    "Your password has been updated. You can now log in with your new password."
                                }
                            }
                            div {
                                class: "mt-4",
                                div {
                                    class: "-mx-2 -my-1.5 flex",
                                    Link {
                                        to: crate::Route::Login {},
                                        class: "btn btn-secondary text-sm",
                                        "Sign in"
                                    }
                                }
                            }
                        }
                    }
                }
            } else {
                form {
                    onsubmit: handle_submit,
                    TextInput {
                        label: "New Password",
                        value: password,
                        r#type: "password",
                        required: true,
                    }

                    TextInput {
                        label: "Confirm New Password",
                        value: confirm_password,
                        r#type: "password",
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
                            "Reset password"
                        }
                    }
                }
            }
        }
    }
}
