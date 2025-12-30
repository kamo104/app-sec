use dioxus::prelude::*;
use crate::api::logout;
use crate::components::dark_mode_toggle::DarkModeToggle;

#[component]
pub fn Navbar() -> Element {
    let nav = use_navigator();

    let handle_logout = move |_| async move {
        let _ = logout().await;
        nav.push(crate::Route::Login {});
    };

    rsx! {
        nav {
            class: "navbar",
            div {
                class: "container",
                div {
                    class: "flex justify-between items-center h-16",
                    div {
                        class: "flex",
                        div {
                            class: "items-center",
                            span {
                                class: "font-bold text-xl text-primary",
                                "MemeShark"
                            }
                        }
                    }
                    div {
                        class: "flex items-center gap-4",
                        DarkModeToggle {}
                        button {
                            onclick: handle_logout,
                            class: "btn btn-primary",
                            "Logout"
                        }
                    }
                }
            }
        }
    }
}
