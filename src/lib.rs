use futures::{Stream, StreamExt};
use std::mem::ManuallyDrop;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// A stream that wraps another stream with a closure that is called once it is dropped.
/// Very useful for libraries that use streams for data transfer and you need to connect
/// when the opposite site drops the connection, thus dropping the stream.
pub struct DropStream<S: Stream<Item = T> + Unpin, T, U: FnOnce()> {
    stream: S,
    // ManuallyDrop used to support FnOnce since ownership of FnOnce needs to be gained in the Drop::drop() method.
    dropper: ManuallyDrop<Box<U>>,
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce()> DropStream<S, T, U> {
    pub fn new(stream: S, dropper: U) -> Self {
        Self {
            stream,
            dropper: ManuallyDrop::new(Box::new(dropper)),
        }
    }
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce()> Stream for DropStream<S, T, U> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = &mut self.stream;
        stream.poll_next_unpin(cx)
    }
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce()> Drop for DropStream<S, T, U> {
    fn drop(&mut self) {
        // Safety: field is taken, and must not be accessed again
        // Since this is in Drop::drop, it is not possible to access it afterwards.
        unsafe {
            let val = ManuallyDrop::take(&mut self.dropper);
            val();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DropStream;
    use futures::{stream::repeat, Stream};

    #[test]
    fn dropper_runs_on_drop() {
        let test_stream = repeat(());

        let mut has_run = false;

        {
            let has_run_ref = &mut has_run;
            let _drop_stream = DropStream::new(test_stream, move || {
                *has_run_ref = true;
            });
        }

        assert!(has_run)
    }

    #[test]
    fn stream_passes_through_result() {
        let test_stream = repeat(true);

        let drop_stream = DropStream::new(test_stream, || {});

        let mut drop_stream = Box::pin(drop_stream);

        let waker = futures::task::noop_waker();
        let mut context = futures::task::Context::from_waker(&waker);
        assert!(drop_stream.as_mut().poll_next(&mut context).is_ready());
    }

    #[test]
    fn dropper_runs_on_drop_after_passing_result() {
        let test_stream = repeat(());

        let mut has_run = false;

        {
            let has_run_ref = &mut has_run;
            let drop_stream = DropStream::new(test_stream, move || {
                *has_run_ref = true;
            });

            let mut drop_stream = Box::pin(drop_stream);

            let waker = futures::task::noop_waker();
            let mut context = futures::task::Context::from_waker(&waker);
            assert!(drop_stream.as_mut().poll_next(&mut context).is_ready());
        }

        assert!(has_run)
    }
}
