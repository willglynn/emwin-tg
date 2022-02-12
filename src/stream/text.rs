use crate::{Error, FetchStream, Fetchable};
use bytes::Bytes;
use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pin_project! {
/// The feed of text products from EMWIN TG.
///
/// See the [text product
/// catalog](https://www.weather.gov/media/emwin/EMWIN_Text_Product_Catalog_210525-1448.pdf)
/// for details.
///
/// `TextSource` retrieves archives from [the operational telecommunications gateway
/// path](https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/), providing on average
/// ~90 seconds of latency, and yielding products up to 3-4 hours old.
pub struct TextSource {
    #[pin]
    text2min: FetchStream<Text2Min>,
    #[pin]
    text6min: FetchStream<Text6Min>,
    #[pin]
    text20min: FetchStream<Text20Min>,
    #[pin]
    text3hour: FetchStream<Text3Hour>,
}
}

impl Stream for TextSource {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if let Poll::Ready(value) = this.text2min.poll_next(cx) {
            return Poll::Ready(value);
        }
        if let Poll::Ready(value) = this.text6min.poll_next(cx) {
            return Poll::Ready(value);
        }
        if let Poll::Ready(value) = this.text20min.poll_next(cx) {
            return Poll::Ready(value);
        }
        if let Poll::Ready(value) = this.text3hour.poll_next(cx) {
            return Poll::Ready(value);
        }
        return Poll::Pending;
    }
}

impl From<reqwest::Client> for TextSource {
    fn from(c: reqwest::Client) -> Self {
        Self {
            text2min: FetchStream::from(c.clone()),
            text6min: FetchStream::from(c.clone()),
            text20min: FetchStream::from(c.clone()),
            text3hour: FetchStream::from(c),
        }
    }
}

impl super::Source for TextSource {}

struct Text3Hour;
impl Fetchable for Text3Hour {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txthrs03.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(60 * 60); // it's regenerated hourly
}

struct Text20Min;
impl Fetchable for Text20Min {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txtmin20.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(20 * 60);
}

struct Text6Min;
impl Fetchable for Text6Min {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txtmin06.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(6 * 60);
}

#[derive(Debug, Clone)]
struct Text2Min;
impl Fetchable for Text2Min {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/txtmin02.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(47);
}
