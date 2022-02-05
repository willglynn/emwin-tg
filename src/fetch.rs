use std::marker::PhantomData;

pub trait Fetchable {
    const URL: &'static str;
    const REFETCH_INTERVAL: std::time::Duration;
}

#[derive(Debug, Clone)]
pub struct FetchState<F: Fetchable> {
    etag: Option<String>,
    last_modified: Option<String>,
    url: PhantomData<F>,
}

impl<F: Fetchable> Default for FetchState<F> {
    fn default() -> Self {
        Self {
            etag: None,
            last_modified: None,
            url: PhantomData::default(),
        }
    }
}

impl<F: Fetchable> FetchState<F> {
    pub fn new() -> Self {
        FetchState::default()
    }

    /// Returns Ok(None) if the resource is not modified
    pub async fn fetch(
        &mut self,
        client: &reqwest::Client,
    ) -> Result<Option<reqwest::Response>, reqwest::Error> {
        let req = client.get(F::URL);
        let req = if let Some(value) = &self.etag {
            req.header(reqwest::header::IF_NONE_MATCH, value)
        } else {
            req
        };
        let req = if let Some(value) = &self.last_modified {
            req.header(reqwest::header::IF_MODIFIED_SINCE, value)
        } else {
            req
        };
        let req = req.build()?;

        log::debug!("GET {}", F::URL);
        let mut resp = client.execute(req).await?.error_for_status().map_err(|e| {
            log::debug!("{}: {}", e, F::URL);
            e
        })?;
        if (self.etag.is_some() || self.last_modified.is_some())
            && resp.status() == reqwest::StatusCode::NOT_MODIFIED
        {
            log::debug!("304 Not Modified: {}", F::URL);
            // Sink the body, if any, to make the connection reusable
            // (Discard errors)
            while let Ok(Some(_)) = resp.chunk().await {}

            Ok(None)
        } else {
            // Copy in the response headers, if any
            self.etag = resp
                .headers()
                .get(reqwest::header::ETAG)
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            self.last_modified = resp
                .headers()
                .get(reqwest::header::LAST_MODIFIED)
                .and_then(|v| v.to_str().ok())
                .map(String::from);

            // Return the response
            log::debug!("200 OK {}", F::URL);
            Ok(Some(resp))
        }
    }
}
