use dioxus::prelude::*;
#[cfg(feature = "server")]
use tracing::Level;

mod api;
mod components;
#[cfg(feature = "server")]
mod db;
#[cfg(feature = "server")]
mod email;
mod pages;

#[derive(Debug, Clone, Routable, PartialEq)]
#[rustfmt::skip]
enum Route {
    #[layout(Navbar)]
    #[route("/")]
    Home {},
    #[route("/login")]
    Login {},
    #[route("/register")]
    Register {},
    #[route("/verify-email")]
    VerifyEmail { token: String },
    #[route("/forgot-password")]
    ForgotPassword {},
    #[route("/reset-password")]
    ResetPassword { token: String },
    #[route("/dashboard")]
    Dashboard {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    #[cfg(feature = "server")]
    {
        dioxus::logger::init(Level::INFO).expect("failed to init logger");

        // Initialize DB
        // We need a runtime to await the DB opening
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            #[cfg(debug_assertions)]
            let db = db::DBHandle::open_dev().await.unwrap();
            #[cfg(not(debug_assertions))]
            let db = db::DBHandle::open("data.db").await.unwrap();

            db::DB.set(db).expect("Failed to set global DB");
        });
    }

    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        Router::<Route> {}
    }
}

/// Home page
use crate::components::dark_mode_toggle::DarkModeToggle;

#[component]
fn Home() -> Element {
    rsx! {
        div {
            class: "flex flex-col items-center justify-center min-h-screen",
            div {
                class: "absolute top-4 right-4",
                DarkModeToggle {}
            }
            div {
                class: "text-center mb-10",
                h1 { class: "text-4xl font-extrabold text-primary", "MemeShark" }
                p { class: "mt-4 text-base text-muted", "The secure platform for your meme needs." }
            }

            div {
                class: "flex gap-4",
                Link {
                    to: Route::Login {},
                    class: "btn btn-primary",
                    "Sign in"
                }
                Link {
                    to: Route::Register {},
                    class: "btn btn-secondary",
                    "Create account"
                }
            }
        }
    }
}

#[component]
pub fn Login() -> Element {
    rsx! {
        pages::login::Login {}
    }
}

#[component]
pub fn Register() -> Element {
    rsx! {
        pages::register::Register {}
    }
}

#[component]
pub fn VerifyEmail(token: String) -> Element {
    rsx! {
        pages::verify_email::VerifyEmail {}
    }
}

#[component]
pub fn ForgotPassword() -> Element {
    rsx! {
        pages::forgot_password::ForgotPassword {}
    }
}

#[component]
pub fn ResetPassword(token: String) -> Element {
    rsx! {
        pages::reset_password::ResetPassword {}
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        pages::dashboard::Dashboard {}
    }
}

/// Blog page
#[component]
pub fn Blog(id: i32) -> Element {
    rsx! {
        div {
            id: "blog",
            h1 { "This is blog #{id}!" }
            Link { to: Route::Blog { id: id - 1 }, "Previous" }
            span { " <---> " }
            Link { to: Route::Blog { id: id + 1 }, "Next" }
        }
    }
}

/// Shared navbar component.
#[component]
fn Navbar() -> Element {
    rsx! {
        Outlet::<Route> {}
    }
}
