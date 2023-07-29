use leptos::{Action, RwSignal, ServerFnError};
mod api;
#[cfg(feature = "ssr")]
mod server;
#[cfg(feature = "ssr")]
pub use server::*;
#[cfg(not(feature = "ssr"))]
mod client;
pub use api::*;
#[cfg(not(feature = "ssr"))]
pub use client::*;

pub static AUTH_COOKIE: &str = "token";

pub type LogoutSignal = Action<LogoutAction, Result<(), ServerFnError>>;
pub type LoginSignal = Action<LoginAction, Result<LoginMessages, ServerFnError>>;
pub type SignupSignal = Action<SignupAction, Result<SignupResponse, ServerFnError>>;
pub type UsernameSignal = RwSignal<Option<String>>;
