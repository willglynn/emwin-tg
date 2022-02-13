#![cfg(target_arch = "wasm32")]

use futures::StreamExt;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn text_stream() {
    let mut stream = emwin_tg::TextStream::new();
    for _ in 0..100 {
        stream
            .next()
            .await
            .expect("stream must not end")
            .expect("must be ok");
    }
}
