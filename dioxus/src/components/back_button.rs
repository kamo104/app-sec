use dioxus::prelude::*;

#[derive(PartialEq, Clone, Props)]
pub struct BackButtonProps {
    #[props(default = "Back".to_string())]
    label: String,
    #[props(default = crate::Route::Login {})]
    to: crate::Route,
}

#[component]
pub fn BackButton(props: BackButtonProps) -> Element {
    rsx! {
        div {
            class: "mt-6 text-center",
            Link {
                class: "font-medium text-primary hover:text-primary-hover",
                to: props.to,
                "{props.label}"
            }
        }
    }
}
