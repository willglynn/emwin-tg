use crate::{Error, FetchState, Fetchable, Product, StreamState};
use std::marker::PhantomData;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::sync::mpsc;

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
    tasks: Vec<tokio::task::JoinHandle<()>>,
    rx: mpsc::Receiver<Result<Product, Error>>,
    pd: PhantomData<S>,
}

impl<S: Source> Stream<S> {
    /// Start a stream using a default HTTP client.
    ///
    /// # Example
    ///
    /// ```
    /// # tokio_test::block_on(async {
    /// let stream = <emwin_tg::Stream<emwin_tg::TextSource>>::new();
    /// # std::mem::drop(stream);
    /// # })
    /// ```
    pub fn new() -> Self {
        Self::new_with_client(crate::default_client())
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
    /// let stream = <emwin_tg::Stream<emwin_tg::TextSource>>::new_with_client(client);
    /// # std::mem::drop(stream);
    /// # })
    /// ```
    pub fn new_with_client(client: reqwest::Client) -> Self {
        let state = Arc::new(Mutex::new(StreamState::new()));

        let (tx, rx) = mpsc::channel(50);

        let tasks = S::spawn_tasks(TaskState { client, state, tx });
        Self {
            tasks,
            rx,
            pd: PhantomData,
        }
    }
}

impl<S: Source> Drop for Stream<S> {
    fn drop(&mut self) {
        for task in self.tasks.iter_mut() {
            task.abort();
        }
    }
}

impl<S: Source> futures::Stream for Stream<S> {
    type Item = Result<Product, Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.rx).poll_recv(cx)
    }
}

/// A source of EMWIN TG data.
pub trait Source: SealedSource {}

// pub trait in private module to keep this trait sealed
mod source {
    pub trait SealedSource: Unpin {
        fn spawn_tasks(task_state: super::TaskState) -> Vec<tokio::task::JoinHandle<()>>;
    }
}
use source::SealedSource;

mod text;
pub use text::TextSource;

/// A `Stream` of text products.
pub type TextStream = Stream<TextSource>;

mod image;
pub use image::ImageSource;

/// A `Stream` of image products.
pub type ImageStream = Stream<ImageSource>;

#[derive(Debug, Clone)]
pub struct TaskState {
    client: reqwest::Client,
    state: Arc<Mutex<StreamState>>,
    tx: mpsc::Sender<Result<Product, Error>>,
}

impl TaskState {
    async fn run<F: Fetchable>(mut self, mut cycles: Option<usize>) {
        let mut resource = <FetchState<F>>::new();

        // Start ticking
        let mut ticker = tokio::time::interval(F::REFETCH_INTERVAL);
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            cycles = match cycles.take() {
                Some(n) if n == 0 => return,
                Some(n) => Some(n - 1),
                None => None,
            };

            ticker.tick().await;
            if self.tx.is_closed() {
                return;
            }

            for attempt in 1..=3 {
                match self.fetch_once(&mut resource).await {
                    Err(e) if attempt == 3 => {
                        log::error!("retries exhausted on {}: {}", F::URL, e);
                        self.tx.send(Err(e.into())).await.ok();
                    }
                    Err(e) => {
                        // Retry soon™
                        log::warn!("error retrieving {}, will retry: {}", F::URL, e);
                        tokio::time::sleep(std::time::Duration::from_secs(4)).await;
                        continue;
                    }
                    Ok(_) => break,
                }
            }
        }
    }

    async fn fetch_once<F: Fetchable>(
        &mut self,
        resource: &mut FetchState<F>,
    ) -> Result<(), Error> {
        let resp = match resource.fetch(&self.client).await? {
            Some(resp) => resp,
            None => return Ok(()),
        };
        let buffer = resp.bytes().await?;

        log::info!("{}: read {} bytes", F::URL, buffer.len());
        decompress::<F>(self.state.clone(), buffer, self.tx.clone()).await;

        Ok(())
    }
}

async fn decompress<F: Fetchable>(
    state: Arc<Mutex<StreamState>>,
    bytes: bytes::Bytes,
    tx: mpsc::Sender<Result<Product, Error>>,
) {
    let result: Result<Vec<Result<Product, Error>>, Error> =
        tokio::task::spawn_blocking(move || {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))?;

            let mut names: Vec<_> = archive.file_names().map(String::from).collect();
            names.sort();

            let names = state.lock().unwrap().add_filenames_in(names);

            log::info!(
                "{}: {} of {} products are new",
                F::URL,
                names.len(),
                archive.len()
            );

            Ok(names
                .into_iter()
                .map(|name| Product::new(archive.by_name(&name)))
                .collect())
        })
        .await
        .unwrap();

    match result {
        Ok(products) => {
            for product_result in products {
                tx.send(product_result).await.ok();
            }
        }
        Err(e) => {
            tx.send(Err(e)).await.ok();
        }
    }
}
