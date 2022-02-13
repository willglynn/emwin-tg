#[cfg(not(target_arch = "wasm32"))]
pub use self::tokio::Ticker;

#[cfg(target_arch = "wasm32")]
pub use self::wasm::Ticker;

#[cfg(not(target_arch = "wasm32"))]
mod tokio;

#[cfg(target_arch = "wasm32")]
mod wasm;
