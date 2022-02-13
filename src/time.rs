use std::pin::Pin;
use std::task::{Context, Poll};

#[derive(Debug)]
pub struct Ticker(tokio::time::Interval);

impl Ticker {
    pub fn new(interval: std::time::Duration) -> Self {
        let mut inner = tokio::time::interval(interval);
        inner.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        Self(inner)
    }
}

impl futures::Stream for Ticker {
    type Item = ();

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_tick(cx).map(|_| Some(()))
    }
}
