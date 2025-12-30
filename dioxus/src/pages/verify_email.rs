use dioxus::prelude::*;
use crate::api::verify_email;
use crate::components::button::Button;
use crate::components::back_button::BackButton;

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
                    class: "min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8",
                    div {
                        class: "sm:mx-auto sm:w-full sm:max-w-md",
                        div {
                            class: "bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10",
                            div {
                                class: "text-center",
                                div {
                                    class: "mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100",
                                    svg {
                                        class: "h-6 w-6 text-green-600",
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
                                    class: "mt-2 text-xl font-medium text-gray-900",
                                    "Email Verified!"
                                }
                                p {
                                    class: "mt-2 text-sm text-gray-500",
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
                    class: "min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8",
                    div {
                        class: "sm:mx-auto sm:w-full sm:max-w-md",
                        div {
                            class: "bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10",
                            div {
                                class: "text-center",
                                div {
                                    class: "mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-red-100",
                                    svg {
                                        class: "h-6 w-6 text-red-600",
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
                                    class: "mt-2 text-xl font-medium text-gray-900",
                                    "Verification Failed"
                                }
                                p {
                                    class: "mt-2 text-sm text-gray-500",
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
                class: "min-h-screen bg-gray-50 flex flex-col justify-center py-12 sm:px-6 lg:px-8",
                div {
                    class: "sm:mx-auto sm:w-full sm:max-w-md",
                    div {
                        class: "bg-white py-8 px-4 shadow sm:rounded-lg sm:px-10",
                        div {
                            class: "text-center",
                            div {
                                class: "flex flex-col items-center",
                                div {
                                    class: "animate-spin rounded-full h-10 w-10 border-b-2 border-indigo-600 mb-4"
                                }
                                p {
                                    class: "text-gray-500",
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
