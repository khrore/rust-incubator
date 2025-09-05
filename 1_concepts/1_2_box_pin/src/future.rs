use std::{
    future::{self, Future},
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

#[pin_project::pin_project]
pub struct TimedWrapper<Fut: Future> {
    #[pin]
    future: Fut,
    start: Option<Instant>,
}

impl<Fut: Future> TimedWrapper<Fut> {
    pub fn new(future: Fut) -> Self {
        Self {
            future,
            start: None,
        }
    }
}

impl<Fut: Future> Future for TimedWrapper<Fut> {
    type Output = (Fut::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let mut this = self.project();

        let start = this.start.get_or_insert(Instant::now());
        let inner_poll = this.future.as_mut().poll(cx);
        let elapsed = start.elapsed();
        match inner_poll {
            Poll::Pending => Poll::Pending,
            Poll::Ready(output) => Poll::Ready((output, elapsed)),
        }
    }
}

pub fn test() {
    let mut time_wrap = TimedWrapper::new(future::ready(5));
    let mut time_wrap = Box::pin(&mut time_wrap);

    let mut cx = Context::from_waker(Waker::noop());

    if let Poll::Ready((output, dur)) = time_wrap.as_mut().poll(&mut cx) {
        println!("{output} in {} nanos", dur.as_nanos());
    }
}
