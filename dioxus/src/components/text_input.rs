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
        "appearance-none block w-full px-3 py-2 border border-red-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-red-500 focus:border-red-500 sm:text-sm"
    } else {
        "appearance-none block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm placeholder-gray-400 focus:outline-none focus:ring-primary-500 focus:border-primary-500 sm:text-sm"
    };

    rsx! {
        div {
            class: "form-group space-y-1 mb-4",
            label {
                class: "block text-sm font-medium text-gray-700",
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
