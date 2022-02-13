use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

pub struct Ticker {
    waker: Arc<Mutex<(Option<Waker>, bool)>>,
    _callback: Closure<dyn FnMut()>,
    interval_handle: i32,
}

impl Ticker {
    pub fn new(interval: std::time::Duration) -> Self {
        let waker = Arc::new(Mutex::new((<Option<Waker>>::None, true)));

        let callback_waker = waker.clone();
        let callback = Closure::wrap(Box::new(move || {
            let mut callback_waker = callback_waker.lock().unwrap();
            callback_waker.1 = true;
            if let Some(waker) = callback_waker.0.take() {
                waker.wake();
            }
        }) as Box<dyn FnMut()>);

        let timeout = i32::try_from(interval.as_millis()).expect("interval too long");
        let window = js_sys::global().unchecked_into::<web_sys::Window>();
        let interval_handle = window
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                timeout,
            )
            .expect("setInterval()");

        Self {
            waker,
            _callback: callback,
            interval_handle,
        }
    }
}

impl Drop for Ticker {
    fn drop(&mut self) {
        let window = js_sys::global().unchecked_into::<web_sys::Window>();
        window.clear_interval_with_handle(self.interval_handle);
    }
}

impl futures::Stream for Ticker {
    type Item = ();

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut waker = self.waker.lock().unwrap();
        if waker.1 {
            waker.1 = false;
            Poll::Ready(Some(()))
        } else {
            let _old_waker = waker.0.insert(cx.waker().clone());
            Poll::Pending
        }
    }
}
