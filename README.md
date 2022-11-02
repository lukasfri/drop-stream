# Drop Stream

A stream that wraps another stream with a closure that is called once it is dropped.
Very useful for libraries that use streams for data transfer and you need to connect
when the opposite site drops the connection, thus dropping the stream.

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
