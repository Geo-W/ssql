use std::pin::Pin;
use std::task::{Context, Poll};

use futures_lite::{ready, Stream};
use futures_lite::StreamExt;
use tiberius::{QueryItem, QueryStream, Row};
pub use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::SsqlMarker;

/// stream
pub struct RowStream<'a, T: SsqlMarker> {
    query_stream: QueryStream<'a>,
    func: Box<dyn for<'b> Fn(&'b Row) -> T + Send>,
}

impl<'a, T: SsqlMarker> RowStream<'a, T> {
    pub(crate) fn new<F>(stream: QueryStream<'a>, func: F) -> Self
    where
        F: 'static + for<'b> Fn(&'b Row) -> T + Send,
    {
        Self {
            query_stream: stream,
            func: Box::new(func),
        }
    }
}

impl<'a, T: SsqlMarker + Unpin> Stream for RowStream<'a, T> {
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        loop {
            match ready!(this.query_stream.poll_next(cx)) {
                None => {
                    return Poll::Ready(None);
                }
                Some(v) => match v.unwrap() {
                    QueryItem::Row(v) => {
                        return Poll::Ready(Some((this.func)(&v)));
                    }
                    QueryItem::Metadata(_) => {
                        continue;
                    }
                },
            }
        }
    }
}
