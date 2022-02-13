use crate::{Error, FetchStream, Fetchable};
use bytes::Bytes;
use futures::Stream;
use pin_project_lite::pin_project;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;

pin_project! {
/// The feed of image products from EMWIN TG.
///
/// See the [image product
/// catalog](https://www.weather.gov/media/emwin/EMWIN_Image_and_Text_Data_Capture_Catalog_v1.3e.pdf)
/// for details.
///
/// `TextSource` retrieves archives from [the operational telecommunications gateway
/// path](https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/), providing on average
/// ~90 seconds of latency, and yielding products up to 3-4 hours old.
pub struct ImageSource {
    #[pin]
    image15min: FetchStream<Image15Min>,
    #[pin]
    image3hour: FetchStream<Image3Hour>,
}
}

impl Stream for ImageSource {
    type Item = Result<Bytes, Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.project();
        if let Poll::Ready(value) = this.image15min.poll_next(cx) {
            return Poll::Ready(value);
        }
        if let Poll::Ready(value) = this.image3hour.poll_next(cx) {
            return Poll::Ready(value);
        }
        Poll::Pending
    }
}

impl super::Source for ImageSource {}

struct Image3Hour;
impl Fetchable for Image3Hour {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/imghrs03.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(60 * 60); // it's regenerated hourly
}

struct Image15Min;
impl Fetchable for Image15Min {
    const URL: &'static str =
        "https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/imgmin15.zip";
    const REFETCH_INTERVAL: Duration = Duration::from_secs(352);
}
