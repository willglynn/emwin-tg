use crate::stream::TaskState;
use crate::Fetchable;
use std::time::Duration;
use tokio::task::JoinHandle;

/// The feed of text products from EMWIN TG.
///
/// See the [text product
/// catalog](https://www.weather.gov/media/emwin/EMWIN_Text_Product_Catalog_210525-1448.pdf)
/// for details.
///
/// `TextSource` retrieves archives from [the operational telecommunications gateway
/// path](https://tgftp.nws.noaa.gov/SL.us008001/CU.EMWIN/DF.xt/DC.gsatR/OPS/), providing on average
/// ~90 seconds of latency, and yielding products up to 3-4 hours old.
pub struct TextSource;

impl super::Source for TextSource {}
impl super::SealedSource for TextSource {
    fn spawn_tasks(task_state: TaskState) -> Vec<JoinHandle<()>> {
        vec![
            tokio::task::spawn(task_state.clone().run::<Text2Min>(None)),
            tokio::task::spawn(task_state.clone().run::<Text6Min>(Some(3))),
            tokio::task::spawn(task_state.clone().run::<Text20Min>(Some(3))),
            tokio::task::spawn(task_state.clone().run::<Text3Hour>(None)),
        ]
    }
}

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
