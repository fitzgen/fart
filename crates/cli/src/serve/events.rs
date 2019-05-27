//! Server-sent events.
//!
//! Hopefully this can eventually be replaced with some kind of built-in stuff
//! in tide: https://github.com/rustasync/tide/issues/234

use crate::Result;
use failure::ResultExt;
use futures::{channel::mpsc, task::Context, Poll, SinkExt, Stream};
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
        }
    }
}

impl Stream for EventStream {
    type Item = io::Result<bytes::Bytes>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(
            match unsafe {
                let receiver = Pin::map_unchecked_mut(self, |s| &mut s.receiver);
                futures::ready!(receiver.poll_next(cx))
            } {
                None => None,
                Some(event) => {
                    static EVENT_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
                    let id = EVENT_ID_COUNTER.fetch_add(1, Ordering::AcqRel);

                    let encoded = format!(
                        "id: {}\nevent: {}\ndata: {}\n\n",
                        id, event.event, event.data
                    );

                    Some(Ok(encoded.into()))
                }
            },
        )
    }
}
