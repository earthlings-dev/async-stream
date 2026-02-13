use std::pin::pin;

use async_stream::stream;

use futures_core::stream::{FusedStream, Stream};
use futures_util::stream::StreamExt;
use tokio::sync::mpsc;
use tokio_test::assert_ok;

#[tokio::test]
async fn noop_stream() {
    let mut s = pin!(stream! {});

    assert!(s.next().await.is_none());
}

#[tokio::test]
async fn empty_stream() {
    let mut ran = false;

    {
        let r = &mut ran;
        let mut s = pin!(stream! {
            *r = true;
            println!("hello world!");
        });

        assert!(s.next().await.is_none());
    }

    assert!(ran);
}

#[tokio::test]
async fn yield_single_value() {
    let s = stream! {
        yield "hello";
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;

    assert_eq!(1, values.len());
    assert_eq!("hello", values[0]);
}

#[tokio::test]
async fn fused() {
    let mut s = pin!(stream! {
        yield "hello";
    });

    assert!(!s.is_terminated());
    assert_eq!(s.next().await, Some("hello"));
    assert_eq!(s.next().await, None);

    assert!(s.is_terminated());
    // This should return None from now on
    assert_eq!(s.next().await, None);
}

#[tokio::test]
async fn yield_multi_value() {
    let s = stream! {
        yield "hello";
        yield "world";
        yield "dizzy";
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;

    assert_eq!(3, values.len());
    assert_eq!("hello", values[0]);
    assert_eq!("world", values[1]);
    assert_eq!("dizzy", values[2]);
}

#[tokio::test]
async fn unit_yield_in_select() {
    use tokio::select;

    async fn do_stuff_async() {}

    let s = stream! {
        select! {
            _ = do_stuff_async() => yield,
            else => yield,
        }
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;
    assert_eq!(values.len(), 1);
}

#[tokio::test]
async fn yield_with_select() {
    use tokio::select;

    async fn do_stuff_async() {}
    async fn more_async_work() {}

    let s = stream! {
        select! {
            _ = do_stuff_async() => yield "hey",
            _ = more_async_work() => yield "hey",
            else => yield "hey",
        }
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;
    assert_eq!(values, vec!["hey"]);
}

#[tokio::test]
async fn return_stream() {
    fn build_stream() -> impl Stream<Item = u32> {
        stream! {
            yield 1;
            yield 2;
            yield 3;
        }
    }

    let s = pin!(build_stream());

    let values: Vec<_> = s.collect().await;
    assert_eq!(3, values.len());
    assert_eq!(1, values[0]);
    assert_eq!(2, values[1]);
    assert_eq!(3, values[2]);
}

#[tokio::test]
async fn consume_channel() {
    let (tx, mut rx) = mpsc::channel(10);

    let mut s = pin!(stream! {
        while let Some(v) = rx.recv().await {
            yield v;
        }
    });

    for i in 0..3 {
        assert_ok!(tx.send(i).await);
        assert_eq!(Some(i), s.next().await);
    }

    drop(tx);
    assert_eq!(None, s.next().await);
}

#[tokio::test]
async fn borrow_self() {
    struct Data(String);

    impl Data {
        fn stream(&self) -> impl Stream<Item = &str> + '_ {
            stream! {
                yield &self.0[..];
            }
        }
    }

    let data = Data("hello".to_string());
    let mut s = pin!(data.stream());

    assert_eq!(Some("hello"), s.next().await);
}

#[tokio::test]
async fn stream_in_stream() {
    let s = stream! {
        let s = stream! {
            for i in 0..3 {
                yield i;
            }
        };

        let mut s = std::pin::pin!(s);
        while let Some(v) = s.next().await {
            yield v;
        }
    };
    let s = pin!(s);

    let values: Vec<_> = s.collect().await;
    assert_eq!(3, values.len());
}

#[tokio::test]
async fn yield_non_unpin_value() {
    let s = stream! {
        for i in 0..3 {
            yield async move { i };
        }
    };
    let s = pin!(s);
    let s: Vec<_> = s.buffered(1).collect().await;

    assert_eq!(s, vec![0, 1, 2]);
}

#[test]
fn inner_try_stream() {
    use async_stream::try_stream;
    use tokio::select;

    async fn do_stuff_async() {}

    let _ = stream! {
        select! {
            _ = do_stuff_async() => {
                let mut another_s = Box::pin(try_stream! {
                    yield;
                });
                let _: Result<(), ()> = another_s.next().await.unwrap();
            },
            else => {},
        }
        yield
    };
}

#[rustversion::attr(not(stable), ignore)]
#[test]
fn test() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
