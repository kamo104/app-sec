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
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

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
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }
        Router::<Route> {}
    }
}

/// Home page
#[component]
fn Home() -> Element {
    rsx! {
        div {
            class: "min-h-screen bg-white dark:bg-black flex flex-col justify-center py-12 sm:px-6 lg:px-8",
            div {
                class: "sm:mx-auto sm:w-full sm:max-w-md text-center",
                h1 {
                    class: "text-4xl font-extrabold text-neutral-900 dark:text-white tracking-tight",
                    "MemeShark"
                }
                p {
                    class: "mt-2 text-base text-neutral-500 dark:text-neutral-400",
                    "The secure platform for your meme needs."
                }
            }

            div {
                class: "mt-8 sm:mx-auto sm:w-full sm:max-w-md",
                div {
                    class: "bg-white dark:bg-neutral-900 py-8 px-4 shadow sm:rounded-lg sm:px-10 border border-neutral-200 dark:border-neutral-800",
                    div {
                        class: "space-y-4",
                        Link {
                            to: Route::Login {},
                            class: "w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                            "Sign in"
                        }
                        Link {
                            to: Route::Register {},
                            class: "w-full flex justify-center py-2 px-4 border border-neutral-300 dark:border-neutral-600 rounded-md shadow-sm text-sm font-medium text-neutral-700 dark:text-neutral-200 bg-white dark:bg-neutral-800 hover:bg-neutral-50 dark:hover:bg-neutral-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500",
                            "Create account"
                        }
                    }
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
