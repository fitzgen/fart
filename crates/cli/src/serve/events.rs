//! Server-sent events.
//!
//! Hopefully this can eventually be replaced with some kind of built-in stuff
//! in tide: https://github.com/rustasync/tide/issues/234

use crate::Result;
use failure::ResultExt;
use futures::{
    channel::mpsc,
    task::{Context, Poll},
    SinkExt, Stream,
};
use std::collections::HashMap;
use std::io;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

/// An event that we will send to the client.
#[derive(Debug, Clone)]
pub struct Event {
    event: String,
    data: String,
}

impl Event {
    /// Create a new event of the given event type and data. The data is
    /// automatically serialized to JSON.
    pub fn new<S>(event: String, data: &S) -> Result<Self>
    where
        S: ?Sized + serde::Serialize,
    {
        assert!(!event.contains(char::is_whitespace));
        let data = serde_json::to_string(data)?;
        Ok(Event { event, data })
    }
}

pub async fn broadcast(
    subscribers: &Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>,
    event: Event,
) -> Result<()> {
    let senders = {
        let subscribers = subscribers.lock().unwrap();
        subscribers.values().cloned().collect::<Vec<_>>()
    };
    futures::future::join_all(senders.into_iter().map(|mut s| {
        let event = event.clone();
        async move {
            s.send(event)
                .await
                .context("failed to send a server-sent event to a client")?;
            Ok(())
        }
    }))
    .await
    .into_iter()
    .collect::<Result<()>>()?;
    Ok(())
}

/// A stream of server-sent events.
///
/// Automatically registers itself in the subscribers set, and removes itself
/// from the subscribers set on drop.
pub struct EventStream {
    id: usize,
    subscribers: Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>,
    receiver: mpsc::Receiver<Event>,
    buf: String,
    index: usize,
}

impl Drop for EventStream {
    fn drop(&mut self) {
        let mut subscribers = self.subscribers.lock().unwrap();
        subscribers.remove(&self.id);
    }
}

impl EventStream {
    pub fn new(subscribers: Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>) -> Self {
        static EVENT_STREAM_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = EVENT_STREAM_ID_COUNTER.fetch_add(1, Ordering::AcqRel);

        let (sender, receiver) = mpsc::channel(16);

        {
            let mut subscribers = subscribers.lock().unwrap();
            subscribers.insert(id, sender);
        }

        EventStream {
            id,
            subscribers,
            receiver,
            buf: String::new(),
            index: 0,
        }
    }
}

impl futures::io::AsyncRead for EventStream {
    /// Attempt to read from the `AsyncRead` into `buf`.
    ///
    /// On success, returns `Poll::Ready(Ok(num_bytes_read))`.
    ///
    /// If no data is available for reading, the method returns
    /// `Poll::Pending` and arranges for the current task (via
    /// `cx.waker().wake_by_ref()`) to receive a notification when the object becomes
    /// readable or is closed.
    ///
    /// # Implementation
    ///
    /// This function may not return errors of kind `WouldBlock` or
    /// `Interrupted`.  Implementations must convert `WouldBlock` into
    /// `Poll::Pending` and either internally retry or convert
    /// `Interrupted` into another error kind.
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        use futures::io::AsyncBufRead;

        loop {
            let data = futures::ready!(self.as_mut().poll_fill_buf(cx))?;
            let n = std::cmp::min(buf.len(), data.len());
            if n == 0 {
                continue;
            }
            buf[..n].copy_from_slice(&data[..n]);
            self.consume(n);
            return Poll::Ready(Ok(n));
        }
    }
}

impl futures::io::AsyncBufRead for EventStream {
    fn poll_fill_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<&[u8]>> {
        use std::fmt::Write;

        let EventStream {
            buf,
            index,
            receiver,
            ..
        } = unsafe { self.get_unchecked_mut() };

        if *index < buf.len() {
            return Poll::Ready(Ok(&buf.as_bytes()[*index..]));
        }

        match unsafe {
            let receiver = Pin::new_unchecked(receiver);
            futures::ready!(receiver.poll_next(cx))
        } {
            None => Poll::Ready(Ok(&[])),
            Some(event) => {
                static EVENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
                let id = EVENT_ID_COUNTER.fetch_add(1, Ordering::AcqRel);

                *index = 0;
                buf.clear();
                write!(
                    buf,
                    "id: {}\nevent: {}\ndata: {}\n\n",
                    id, event.event, event.data
                )
                .unwrap();

                Poll::Ready(Ok(&buf.as_bytes()[*index..]))
            }
        }
    }

    fn consume(mut self: Pin<&mut Self>, amt: usize) {
        self.index += amt;
        assert!(self.index <= self.buf.len());
    }
}
