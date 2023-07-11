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
