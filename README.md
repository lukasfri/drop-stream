# Drop Stream

![crates.io](https://img.shields.io/crates/v/drop-stream.svg)
![license](https://img.shields.io/crates/l/drop-stream.svg)
![docs](https://img.shields.io/docsrs/drop-stream.svg)
![CI](https://github.com/DreamplaySE/drop-stream/actions/workflows/rust.yml/badge.svg)

A stream that wraps another stream with a closure that is called once it is dropped.
Very useful for libraries that use streams for data transfer and you need to connect
when the opposite site drops the connection, thus dropping the stream.

This crate only depends on futures-core and thus is a minimal dependency, suitable for any type of project utilising futures.

## Example

```rust
use futures::Stream;
use drop_stream::DropStream;
let test_stream = futures::stream::repeat(true);
{
    let wrapped_stream = DropStream::new(test_stream, move || {
        println!("Stream has been dropped!");
    });
    let mut wrapped_stream = Box::pin(wrapped_stream);
    let waker = futures::task::noop_waker();
    let mut context = futures::task::Context::from_waker(&waker);
    assert_eq!(
        wrapped_stream.as_mut().poll_next(&mut context),
        std::task::Poll::Ready(Some(true))
    );
}
```

## Acknowledgement

I thank [Aadam Zocolo](https://github.com/AadamZ5) for letting me take over the crate name "drop-stream" on crates.io and replace his 0.1 version.
