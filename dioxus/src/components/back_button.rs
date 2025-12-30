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
                to: props.to,
                class: "font-medium text-indigo-600 hover:text-indigo-500 dark:text-indigo-400 dark:hover:text-indigo-300",
                "{props.label}"
            }
        }
    }
}
