use std::pin::pin;

use async_stream::stream;

use futures_util::stream::StreamExt;

#[tokio::test]
async fn test() {
    let s = stream! {
        yield "hello";
        yield "world";
    };

    let s = stream! {
        for await x in s {
            yield x.to_owned() + "!";
        }
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;

    assert_eq!(2, values.len());
    assert_eq!("hello!", values[0]);
    assert_eq!("world!", values[1]);
}
