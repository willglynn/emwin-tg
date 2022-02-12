use crate::{Error, Product, StreamState};
use bytes::Bytes;
use pin_project_lite::pin_project;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

pin_project! {
/// A stream of products from the EMWIN TG text feed.
///
/// `Stream` implements `futures::stream::Stream`, producing `Result<Product, Error>` when
/// polled.
///
/// The `Stream` will continue attempting to retrieve data until it's dropped, even if it reports
/// errors.
///
/// The sequence of products has no guaranteed order. The sequence may have gaps, either due to
/// EMWIN operational difficulties or due to network failures. `Stream` attempts to produce all
/// available products as they are first detected, but also seeks to avoid overloading the
/// telecommunications gateway with a high rate of requests.
///
/// # Example
///
/// ```rust
/// # tokio_test::block_on(async {
/// use futures::StreamExt;
///
/// let mut stream = emwin_tg::TextStream::new();
///
/// while let Some(event) = stream.next().await {
///     # break;
///     match event {
///         Ok(product) => {
///             // Handle the product
///             assert_eq!(product.mime_type(), Some("text/plain"));
///             println!("{}: {}", product.filename, product.into_string_lossy());
///             # break;
///         }
///         Err(error) => {
///             // Stream continues, automatically retrying as needed
///             eprintln!("uh oh: {}", error);
///         }
///     }
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct Stream<S: Source> {
    #[pin]
    source: S,
    state: StreamState,
    output_buffer: VecDeque<Result<Product, Error>>,
}
}

impl<S: Source + From<reqwest::Client>> Default for Stream<S> {
    fn default() -> Self {
        Self::from_client(crate::default_client())
    }
}

impl<S: Source + From<reqwest::Client>> Stream<S> {
    /// Start a stream using a default HTTP client.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let stream = <emwin_tg::Stream<emwin_tg::TextSource>>::default();
    /// # std::mem::drop(stream);
    /// # })
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a stream using a particular HTTP client.
    ///
    /// The NWS has restricted other APIs by requiring a `User-Agent`. If you supply your own
    /// client, be sure to specify one.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let client = reqwest::Client::builder()
    ///        .user_agent("Your Software v1.0 (author@example.com)")
    ///        .build()
    ///        .unwrap();
    ///
    /// let stream = <emwin_tg::Stream<emwin_tg::TextSource>>::from_client(client);
    /// # std::mem::drop(stream);
    /// # })
    /// ```
    pub fn from_client(client: reqwest::Client) -> Self {
        Self {
            source: S::from(client),
            state: StreamState::new(),
            output_buffer: VecDeque::with_capacity(50),
        }
    }
}

impl<S: Source> futures::Stream for Stream<S> {
    type Item = Result<Product, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();

        loop {
            if let Some(value) = this.output_buffer.pop_front() {
                break Poll::Ready(Some(value));
            }

            match this.source.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => match this.state.new_products_in(bytes) {
                    Ok(vec) => this.output_buffer.extend(vec),
                    Err(e) => break Poll::Ready(Some(Err(e))),
                },
                Poll::Ready(Some(Err(e))) => break Poll::Ready(Some(Err(e))),
                Poll::Ready(None) => break Poll::Ready(None),
                Poll::Pending => break Poll::Pending,
            }
        }
    }
}

/// A source of EMWIN TG data.
pub trait Source: futures::stream::Stream<Item = Result<Bytes, crate::Error>> {}

mod text;
pub use text::TextSource;

/// A `Stream` of text products.
pub type TextStream = Stream<TextSource>;

mod image;
pub use image::ImageSource;

// A `Stream` of image products.
pub type ImageStream = Stream<ImageSource>;
