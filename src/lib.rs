//! # `emwin-tg`
//!
//! A Rust client for the [NWS Emergency Managers Weather Information
//! Network](https://www.weather.gov/emwin/) telecommunications gateway.
//!
//! EMWIN is [one of several](https://www.weather.gov/nwws/dissemination) platforms through which
//! the National Weather Service distributes text products. EMWIN is transmitted via radio to [GOES
//! satellites](https://noaasis.noaa.gov/GOES/HRIT/broadcast.html) for re-broadcast, and published
//! as files on the [NWS telecommunications gateway](https://www.weather.gov/tg/anonymous) service.
//!
//! The NWS telecommunications gateway permits anonymous access from the general public over the
//! Internet. Usage of this crate therefore requires no setup.
//!
//! # Example
//!
//! ```rust
//! # tokio_test::block_on(async {
//! use futures::StreamExt;
//!
//! let mut stream = emwin_tg::TextStream::new();
//!
//! while let Some(event) = stream.next().await {
//!     # break;
//!     match event {
//!         Ok(product) => {
//!             // Handle the product
//!             assert_eq!(product.mime_type(), Some("text/plain"));
//!             println!("{}: {}", product.filename, product.into_string_lossy());
//!             # break;
//!         }
//!         Err(error) => {
//!             // Stream continues, automatically retrying as needed
//!             eprintln!("uh oh: {}", error);
//!         }
//!     }
//! }
//! # })
//! ```

#![forbid(unsafe_code)]

mod error;
mod fetch;
mod product;
mod state;
mod stream;
mod time;

pub use error::Error;
pub use product::Product;
pub use stream::{ImageSource, ImageStream, Source, Stream, TextSource, TextStream};

pub(crate) use fetch::*;
pub(crate) use state::*;

pub(crate) fn default_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("https://github.com/willglynn/emwin")
        .build()
        .expect("build client")
}
