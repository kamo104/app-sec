use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct ButtonProps {
    #[props(default = "submit".to_string())]
    r#type: String,
    #[props(default = "".to_string())]
    class: String,
    children: Element,
    #[props(default)]
    onclick: EventHandler<MouseEvent>,
}

#[component]
pub fn Button(props: ButtonProps) -> Element {
    // Default styles for primary button
    let base_class = "w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-primary-500";

    // Allow overriding or appending classes
    let class_name = if props.class.is_empty() {
        base_class.to_string()
    } else {
        format!("{} {}", base_class, props.class)
    };

    rsx! {
        button {
            r#type: "{props.r#type}",
            class: "{class_name}",
            onclick: move |evt| props.onclick.call(evt),
            {props.children}
        }
    }
}
