mod async_fn_trait;
#[cfg(not(target_arch = "wasm32"))]
mod cache_res;
mod context;
#[cfg(not(target_arch = "wasm32"))]
mod decode_time_str;
mod log;
mod path_prefix;
mod test;

/// Cookies impl for Request type
pub mod cookies;

pub(crate) use async_fn_trait::AsyncFnReturn;
#[cfg(not(target_arch = "wasm32"))]
pub use cache_res::{cache_fallible_res, cache_res};
pub(crate) use context::provide_context_signal_replace;
#[cfg(not(target_arch = "wasm32"))]
pub use decode_time_str::{ComputedDuration, Duration, InvalidDuration};
pub use path_prefix::*;
