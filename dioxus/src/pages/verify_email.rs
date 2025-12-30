use dioxus::prelude::*;
use crate::api::verify_email;
use crate::components::button::Button;
use crate::components::back_button::BackButton;
use crate::components::dark_mode_toggle::DarkModeToggle;

#[component]
pub fn VerifyEmail() -> Element {
    let route = use_route::<crate::Route>();
    let nav = use_navigator();
    let mut verification_status = use_signal::<Option<Result<(), String>>>(|| None);

    // Extract token from query parameters
    let token = if let crate::Route::VerifyEmail { token } = route {
        token
    } else {
        String::new()
    };

    use_future(move || {
        let token = token.clone();
        async move {
            if token.is_empty() {
                verification_status.set(Some(Err("No verification token provided".to_string())));
                return;
            }

            match verify_email(token).await {
                Ok(_) => verification_status.set(Some(Ok(()))),
                Err(e) => verification_status.set(Some(Err(e.to_string()))),
            }
        }
    });

    match verification_status() {
        Some(result) => match result {
            Ok(_) => rsx! {
                div {
                    class: "flex flex-col items-center justify-center min-h-screen p-4",
                    div {
                        class: "absolute top-4 right-4",
                        DarkModeToggle {}
                    }
                    div {
                        class: "w-full max-w-md",
                        div {
                            class: "card",
                            div {
                                class: "text-center",
                                div {
                                    class: "mx-auto flex items-center justify-center h-12 w-12 rounded-full",
                                    style: "background-color: #dcfce7;", // green-100
                                    svg {
                                        class: "h-6 w-6",
                                        style: "color: #16a34a;", // green-600
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        stroke: "currentColor",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M5 13l4 4L19 7"
                                        }
                                    }
                                }
                                h3 {
                                    class: "mt-2 text-xl font-medium",
                                    "Email Verified!"
                                }
                                p {
                                    class: "mt-2 text-sm text-muted",
                                    "Your email has been successfully verified. You can now access your account."
                                }
                                div {
                                    class: "mt-6",
                                    Button {
                                        onclick: move |_| {
                                            nav.push(crate::Route::Login {});
                                        },
                                        "Sign in"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            Err(msg) => rsx! {
                div {
                    class: "flex flex-col items-center justify-center min-h-screen p-4",
                    div {
                        class: "absolute top-4 right-4",
                        DarkModeToggle {}
                    }
                    div {
                        class: "w-full max-w-md",
                        div {
                            class: "card",
                            div {
                                class: "text-center",
                                div {
                                    class: "mx-auto flex items-center justify-center h-12 w-12 rounded-full",
                                    style: "background-color: #fee2e2;", // red-100
                                    svg {
                                        class: "h-6 w-6",
                                        style: "color: #dc2626;", // red-600
                                        fill: "none",
                                        view_box: "0 0 24 24",
                                        stroke: "currentColor",
                                        path {
                                            stroke_linecap: "round",
                                            stroke_linejoin: "round",
                                            stroke_width: "2",
                                            d: "M6 18L18 6M6 6l12 12"
                                        }
                                    }
                                }
                                h3 {
                                    class: "mt-2 text-xl font-medium",
                                    "Verification Failed"
                                }
                                p {
                                    class: "mt-2 text-sm text-muted",
                                    "{msg}"
                                }
                                div {
                                    class: "mt-6",
                                    BackButton {
                                        label: "Back to Login",
                                        to: crate::Route::Login {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div {
                class: "flex flex-col items-center justify-center min-h-screen p-4",
                div {
                    class: "absolute top-4 right-4",
                    DarkModeToggle {}
                }
                div {
                    class: "w-full max-w-md",
                    div {
                        class: "card",
                        div {
                            class: "text-center",
                            div {
                                class: "flex flex-col items-center",
                                div {
                                    class: "rounded-full h-10 w-10 border-b-2 mb-4",
                                    style: "border-color: var(--primary-color); animation: spin 1s linear infinite;"
                                }
                                p {
                                    class: "text-muted",
                                    "Verifying your email..."
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
