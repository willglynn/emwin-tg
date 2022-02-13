use futures::StreamExt;

#[cfg(target_arch = "wasm32")]
fn main() {
    // ?
}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main]
async fn main() {
    env_logger::builder()
        .filter(None, log::LevelFilter::Info)
        .filter_module("emwin", log::LevelFilter::Info)
        .parse_default_env()
        .init();

    let mut stream = emwin_tg::TextStream::new();
    while let Some(result) = stream.next().await {
        match result {
            Ok(product) => {
                println!(
                    "{}:\n    {:.100}",
                    product.filename,
                    format!("{:?}", product.string_contents())
                );
            }
            Err(error) => {
                log::error!("error: {}", error)
            }
        }
    }
}
