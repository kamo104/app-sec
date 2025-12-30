use dioxus::prelude::*;

#[component]
pub fn TextInput(
    label: String,
    value: Signal<String>,
    #[props(default = "text".to_string())]
    r#type: String,
    #[props(default = None)]
    error: Option<String>,
    #[props(default = false)]
    required: bool,
    #[props(default = String::new())]
    placeholder: String,
    #[props(default = None)]
    oninput: Option<EventHandler<FormEvent>>,
) -> Element {
    let border_class = if error.is_some() {
        "input border-red-500 focus:border-red-500 focus:ring-red-500"
    } else {
        "input"
    };

    rsx! {
        div {
            class: "form-group mb-4",
            label {
                class: "label",
                "{label}"
            }
            div {
                class: "mt-1",
                input {
                    r#type: "{r#type}",
                    value: "{value}",
                    oninput: move |evt| {
                        value.set(evt.value());
                        if let Some(handler) = oninput {
                            handler.call(evt);
                        }
                    },
                    required: required,
                    class: "{border_class}",
                    placeholder: "{placeholder}",
                }
            }
            if let Some(err) = error {
                p {
                    class: "mt-2 text-sm text-red-600",
                    "{err}"
                }
            }
        }
    }
}
