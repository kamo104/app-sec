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
        "border-red-500 text-red-900 placeholder-red-400 bg-red-50 focus:ring-red-500 focus:border-red-500"
    } else {
        "border-neutral-300 dark:border-neutral-700 placeholder-neutral-400 dark:placeholder-neutral-500 focus:ring-indigo-500 focus:border-indigo-500"
    };

    rsx! {
        div {
            class: "mb-4",
            label {
                class: "block text-sm font-medium text-neutral-700 dark:text-neutral-300 mb-1",
                "{label}"
            }
            div {
                class: "mt-1 relative rounded-md shadow-sm",
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
                    class: "appearance-none block w-full px-3 py-2 border rounded-md shadow-sm sm:text-sm bg-[var(--surface)] text-[var(--text)] {border_class} focus:outline-none",
                    placeholder: "{placeholder}",
                }
            }
            if let Some(err) = error {
                p {
                    class: "mt-2 text-sm text-red-600 dark:text-red-400 font-medium",
                    "{err}"
                }
            }
        }
    }
}
