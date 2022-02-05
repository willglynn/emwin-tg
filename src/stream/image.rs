use crate::stream::TaskState;
use crate::Fetchable;
use std::time::Duration;
use tokio::task::JoinHandle;

/// The feed of image products from EMWIN TG.
///
/// See the [image product
/// catalog](https://www.weather.gov/media/emwin/EMWIN_Image_and_Text_Data_Capture_Catalog_v1.3e.pdf)
/// for details.
///
/// `TextSource` retrieves archives from [the operational telecommunications gateway
/// path](https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/), providing on average
/// ~90 seconds of latency, and yielding products up to 3-4 hours old.
pub struct ImageSource;

impl super::Source for ImageSource {}
impl super::SealedSource for ImageSource {
    fn spawn_tasks(task_state: TaskState) -> Vec<JoinHandle<()>> {
        vec![
            tokio::task::spawn(task_state.clone().run::<Image15Min>(None)),
            tokio::task::spawn(task_state.clone().run::<Image3Hour>(None)),
        ]
    }
}

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
