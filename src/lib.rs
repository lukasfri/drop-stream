use futures_core::Stream;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

/// A stream that wraps another stream with a closure that is called once it is dropped.
/// Very useful for libraries that use streams for data transfer and you need to connect
/// when the opposite site drops the connection, thus dropping the stream.
///
/// Example
/// ```
/// use futures::Stream;
/// use drop_stream::DropStream;
///
/// let test_stream = futures::stream::repeat(true);
/// {
///     let wrapped_stream = DropStream::new(test_stream, move || {
///         println!("Stream has been dropped!");
///     });
///
///     let mut wrapped_stream = Box::pin(wrapped_stream);
///
///     let waker = futures::task::noop_waker();
///     let mut context = futures::task::Context::from_waker(&waker);
///     assert_eq!(
///         wrapped_stream.as_mut().poll_next(&mut context),
///         std::task::Poll::Ready(Some(true))
///     );
/// }
/// ```
pub struct DropStream<S: Stream<Item = T> + Unpin, T, U: FnOnce() + Unpin> {
    stream: S,
    // ManuallyDrop used to support FnOnce since ownership of FnOnce needs to be gained in the Drop::drop() method.
    dropper: Option<U>,
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce() + Unpin> DropStream<S, T, U> {
    pub fn new(stream: S, dropper: U) -> Self {
        Self {
            stream,
            dropper: Some(dropper),
        }
    }
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce() + Unpin> Stream for DropStream<S, T, U> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let stream = Pin::new(&mut self.stream);
        stream.poll_next(cx)
    }
}

impl<S: Stream<Item = T> + Unpin, T, U: FnOnce() + Unpin> Drop for DropStream<S, T, U> {
    fn drop(&mut self) {
        let Some(dropper) = self.dropper.take() else {
            // Only taken in the "drop"-method, and always set in the constructor. Thus it cannot be None here.
            unreachable!()
        };

        dropper()
    }
}

pub trait DropStreamExt<U: FnOnce() + Unpin>: Stream + Unpin + Sized {
    /// Wraps the stream with a closure that is called once it is dropped.
    /// ex:
    /// ```rust
    /// use std::task::Poll;
    /// use futures::{stream::repeat, Stream};
    /// use drop_stream::DropStreamExt;
    ///
    /// fn main() {
    ///     let some_stream = repeat(true);
    ///
    ///     let mut has_run = false;
    ///     let has_run_ref = &mut has_run;
    ///     let drop_stream = some_stream.on_drop(move || {
    ///         *has_run_ref = true;
    ///         println!("Stream has been dropped!")
    ///     });
    ///
    ///     let mut drop_stream = Box::pin(drop_stream);
    ///
    ///     // Some stream work and polling...
    ///
    ///     drop(drop_stream); // Runs the closure
    ///     assert!(has_run);
    /// }
    /// ```
    fn on_drop(self, dropper: U) -> DropStream<Self, Self::Item, U>;
}

impl<T, U: FnOnce() + Unpin> DropStreamExt<U> for T
where
    T: Stream + Unpin + Sized,
{
    fn on_drop(self, dropper: U) -> DropStream<T, T::Item, U> {
        DropStream::new(self, dropper)
    }
}

#[cfg(test)]
mod tests {
    use std::task::Poll;

    use crate::{DropStream, DropStreamExt};
    use futures::{stream::repeat, Stream};

    #[test]
    fn dropper_runs_on_drop() {
        let test_stream = repeat(true);

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
        assert_eq!(
            drop_stream.as_mut().poll_next(&mut context),
            Poll::Ready(Some(true))
        );
    }

    #[test]
    fn dropper_runs_on_drop_after_passing_result() {
        let test_stream = repeat(true);

        let mut has_run = false;

        {
            let has_run_ref = &mut has_run;
            let drop_stream = DropStream::new(test_stream, move || {
                *has_run_ref = true;
            });

            let mut drop_stream = Box::pin(drop_stream);

            let waker = futures::task::noop_waker();
            let mut context = futures::task::Context::from_waker(&waker);
            assert_eq!(
                drop_stream.as_mut().poll_next(&mut context),
                Poll::Ready(Some(true))
            );
        }

        assert!(has_run)
    }

    #[test]
    fn stream_trait_is_implemented() {
        let test_stream = repeat(true);

        let mut has_run = false;

        {
            let has_run_ref = &mut has_run;
            let drop_stream = test_stream.on_drop(move || {
                *has_run_ref = true;
            });

            let mut drop_stream = Box::pin(drop_stream);

            let waker = futures::task::noop_waker();
            let mut context = futures::task::Context::from_waker(&waker);
            assert_eq!(
                drop_stream.as_mut().poll_next(&mut context),
                Poll::Ready(Some(true))
            );
        }

        assert!(has_run)
    }
}
