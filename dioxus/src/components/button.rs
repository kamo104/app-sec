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
    let base_class = "btn btn-primary w-full";

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
