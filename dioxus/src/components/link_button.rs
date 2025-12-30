use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct LinkButtonProps {
    #[props(default = "".to_string())]
    class: String,
    children: Element,
    to: crate::Route,
}

#[component]
pub fn LinkButton(props: LinkButtonProps) -> Element {
    // Default styles for the link button
    let base_class = "btn btn-secondary w-full text-primary";

    // Allow overriding or appending classes
    let class_name = if props.class.is_empty() {
        base_class.to_string()
    } else {
        format!("{} {}", base_class, props.class)
    };

    rsx! {
        Link {
            to: props.to,
            class: "{class_name}",
            {props.children}
        }
    }
}
