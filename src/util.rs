use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct Ready<T>(Option<T>);

impl<T> Ready<T> {
    pub fn new(x: T) -> Self {
        Self(Some(x))
    }
}

impl<T> Unpin for Ready<T> {}

impl<T> Future for Ready<T> {
    type Output = T;

    #[inline]
    fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.0.take().expect("Ready polled after completion"))
    }
}
