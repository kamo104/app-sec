use dioxus::prelude::*;
use crate::components::auth_layout::AuthLayout;
use crate::components::text_input::TextInput;
use crate::components::button::Button;
use crate::components::link_button::LinkButton;
use crate::api::{register, RegistrationData};

#[component]
pub fn Register() -> Element {
    let mut username = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut confirm_password = use_signal(|| String::new());

    // Error signals
    let mut username_error = use_signal::<Option<String>>(|| None);
    let mut email_error = use_signal::<Option<String>>(|| None);
    let mut password_error = use_signal::<Option<String>>(|| None);
    let mut confirm_password_error = use_signal::<Option<String>>(|| None);
    let mut general_error = use_signal::<Option<String>>(|| None);

    let nav = use_navigator();

    // Helper functions for individual field validation
    let mut validate_username_field = move |val: &str| {
        let u_result = field_validator::validate_username(val, 3, 20, true);
        if !u_result.is_valid {
            let err_msg = u_result.errors.iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            username_error.set(Some(err_msg));
            false
        } else {
            username_error.set(None);
            true
        }
    };

    let mut validate_email_field = move |val: &str| {
        let e_result = field_validator::validate_email(val);
        if !e_result.is_valid {
            let err_msg = e_result.errors.iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            email_error.set(Some(err_msg));
            false
        } else {
            email_error.set(None);
            true
        }
    };

    let mut validate_password_field = move |val: &str| {
        let p_result = field_validator::validate_password(val);
        if !p_result.is_valid {
            let err_msg = p_result.errors.iter()
                .map(|e| format!("{:?}", e))
                .collect::<Vec<_>>()
                .join(", ");
            password_error.set(Some(err_msg));
            false
        } else {
            password_error.set(None);
            true
        }
    };

    let mut validate_confirm_password_field = move |val: &str| {
        if val != password() {
            confirm_password_error.set(Some("Passwords do not match".to_string()));
            false
        } else {
            confirm_password_error.set(None);
            true
        }
    };

    // Client-side validation logic using field-validator directly
    let mut validate_form = move || {
        let mut is_valid = true;

        if !validate_username_field(&username()) { is_valid = false; }
        if !validate_email_field(&email()) { is_valid = false; }
        if !validate_password_field(&password()) { is_valid = false; }
        if !validate_confirm_password_field(&confirm_password()) { is_valid = false; }

        is_valid
    };

    let handle_submit = move |_| async move {
        general_error.set(None);

        if !validate_form() {
            return;
        }

        let data = RegistrationData {
            username: username(),
            email: email(),
            password: password(),
        };

        match register(data).await {
            Ok(_) => {
                // Redirect to success page or login
                // For now, redirect to login
                nav.push(crate::Route::Login {});
            }
            Err(e) => {
                // Ideally we parse the JSON error here if it's a validation error
                // For simplicity, just show generic error
                general_error.set(Some(e.to_string()));
            }
        }
    };

    rsx! {
        AuthLayout {
            title: "Create a new account".to_string(),
            form {
                onsubmit: handle_submit,
                TextInput {
                    label: "Username",
                    value: username,
                    required: true,
                    error: username_error(),
                    oninput: move |evt: FormEvent| { validate_username_field(&evt.value()); },
                }
                TextInput {
                    label: "Email address",
                    value: email,
                    r#type: "email",
                    required: true,
                    error: email_error(),
                    oninput: move |evt: FormEvent| { validate_email_field(&evt.value()); },
                }
                TextInput {
                    label: "Password",
                    value: password,
                    r#type: "password",
                    required: true,
                    error: password_error(),
                    oninput: move |evt: FormEvent| {
                        validate_password_field(&evt.value());
                        // Also re-validate confirm password if it has value
                        if !confirm_password().is_empty() {
                            validate_confirm_password_field(&confirm_password());
                        }
                    },
                }
                TextInput {
                    label: "Confirm Password",
                    value: confirm_password,
                    r#type: "password",
                    required: true,
                    error: confirm_password_error(),
                    oninput: move |evt: FormEvent| { validate_confirm_password_field(&evt.value()); },
                }

                if let Some(msg) = general_error() {
                    div {
                        class: "alert alert-error",
                        div {
                            class: "flex",
                            div {
                                class: "ml-3",
                                h3 { class: "text-sm font-medium", "Registration Failed" }
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
                        "Register"
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
                            class: "px-2 bg-white text-muted",
                            "Or"
                        }
                    }
                }
                div {
                    class: "mt-6",
                    LinkButton {
                        to: crate::Route::Login {},
                        "Sign in to existing account"
                    }
                }
            }
        }
    }
}
