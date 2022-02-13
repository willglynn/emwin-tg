use crate::time::Ticker;
use crate::Error;
use bytes::Bytes;
use futures::future::BoxFuture;
use pin_project_lite::pin_project;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Fetchable {
    const URL: &'static str;
    const REFETCH_INTERVAL: std::time::Duration;
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
struct FetchState {
    etag: Option<String>,
    last_modified: Option<String>,
}

type FetchResult = Result<Option<(Bytes, FetchState)>, Error>;

pin_project! {
pub struct FetchStream<F: Fetchable> {
    client: reqwest::Client,
    url: PhantomData<F>,
    fetch_state: FetchState,
    #[pin]
    ticker: Ticker,
    fetches: Vec<BoxFuture<'static, FetchResult>>,
    fetch_results: Vec<Result<Bytes, Error>>,
}
}

impl<F: Fetchable> From<reqwest::Client> for FetchStream<F> {
    fn from(client: reqwest::Client) -> Self {
        Self {
            client,
            fetch_state: FetchState::default(),
            url: PhantomData::default(),
            ticker: Ticker::new(F::REFETCH_INTERVAL),
            fetches: Vec::with_capacity(2),
            fetch_results: Vec::with_capacity(2),
        }
    }
}

impl<F: Fetchable> futures::Stream for FetchStream<F> {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();

        match this.ticker.poll_next(cx) {
            Poll::Ready(_) => {
                this.fetches.push(Box::pin(fetch(
                    F::URL,
                    this.client.clone(),
                    this.fetch_state.clone(),
                )));
            }
            Poll::Pending => (),
        }

        let mut to_remove = Vec::new();
        for (i, fetch) in this.fetches.iter_mut().enumerate() {
            match fetch.as_mut().poll(cx) {
                Poll::Ready(result) => {
                    to_remove.push(i);
                    match result {
                        Ok(Some((bytes, fetch_state))) => {
                            *this.fetch_state = fetch_state;
                            this.fetch_results.push(Ok(bytes));
                        }
                        Ok(None) => {}
                        Err(e) => this.fetch_results.push(Err(e)),
                    }
                }
                Poll::Pending => (),
            }
        }

        for index in to_remove.into_iter().rev() {
            this.fetches.remove(index);
        }

        match this.fetch_results.pop() {
            Some(result) => Poll::Ready(Some(result)),
            None => Poll::Pending,
        }
    }
}

/// Returns Ok(None) if the resource is not modified
async fn fetch(
    url: &'static str,
    client: reqwest::Client,
    fetch_state: FetchState,
) -> Result<Option<(Bytes, FetchState)>, crate::Error> {
    let req = client.get(url);
    let req = if let Some(value) = &fetch_state.etag {
        req.header(reqwest::header::IF_NONE_MATCH, value)
    } else {
        req
    };
    let req = if let Some(value) = &fetch_state.last_modified {
        req.header(reqwest::header::IF_MODIFIED_SINCE, value)
    } else {
        req
    };
    let req = req.build()?;

    log::debug!("GET {}", url);
    let mut resp = client.execute(req).await?.error_for_status().map_err(|e| {
        log::debug!("{}: {}", e, url);
        e
    })?;
    if (fetch_state.etag.is_some() || fetch_state.last_modified.is_some())
        && resp.status() == reqwest::StatusCode::NOT_MODIFIED
    {
        log::debug!("304 Not Modified: {}", url);
        // Sink the body, if any, to make the connection reusable
        // (Discard errors)
        while let Ok(Some(_)) = resp.chunk().await {}

        Ok(None)
    } else {
        // Copy in the response headers, if any
        let etag = resp
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let last_modified = resp
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        let new_fetch_state = FetchState {
            etag,
            last_modified,
        };
        log::debug!("200 OK {}", url);
        let body = resp.bytes().await?;

        // Return the response
        Ok(Some((body, new_fetch_state)))
    }
}
