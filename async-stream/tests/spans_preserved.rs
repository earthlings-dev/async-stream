use std::pin::pin;

use async_stream::stream;
use futures_util::stream::StreamExt;

#[tokio::test]
async fn spans_preserved() {
    let s = stream! {
     assert_eq!(line!(), 9);
    };
    let mut s = pin!(s);

    assert!(s.next().await.is_none());
}
