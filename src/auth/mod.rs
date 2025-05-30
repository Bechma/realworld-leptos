use leptos::prelude::*;
mod api;
#[cfg(feature = "ssr")]
mod server;
pub use api::*;
#[cfg(feature = "ssr")]
pub use server::*;

pub type LogoutSignal = ServerAction<LogoutAction>;
pub type LoginSignal = ServerAction<LoginAction>;
pub type SignupSignal = ServerAction<SignupAction>;
pub type UsernameSignal = RwSignal<Option<String>>;
