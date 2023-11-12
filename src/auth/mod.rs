use leptos::{Action, RwSignal, ServerFnError};
mod api;
#[cfg(feature = "ssr")]
mod server;
pub use api::*;
#[cfg(feature = "ssr")]
pub use server::*;

pub type LogoutSignal = Action<LogoutAction, Result<(), ServerFnError>>;
pub type LoginSignal = Action<LoginAction, Result<LoginMessages, ServerFnError>>;
pub type SignupSignal = Action<SignupAction, Result<SignupResponse, ServerFnError>>;
pub type UsernameSignal = RwSignal<Option<String>>;
