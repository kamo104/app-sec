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
    let base_class = "w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-primary-600 bg-primary-50 hover:bg-primary-100";

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
