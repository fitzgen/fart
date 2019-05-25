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
    peanut_gallery: &Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>,
    event: Event,
) -> Result<()> {
    let senders = {
        let pg = peanut_gallery.lock().unwrap();
        pg.values().cloned().collect::<Vec<_>>()
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
/// Automatically registers itself in the peanut gallery, and removes itself
/// from the peanut gallery on drop.
pub struct EventStream {
    id: usize,
    peanut_gallery: Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>,
    receiver: mpsc::Receiver<Event>,
}

impl Drop for EventStream {
    fn drop(&mut self) {
        let mut peanut_gallery = self.peanut_gallery.lock().unwrap();
        peanut_gallery.remove(&self.id);
    }
}

impl EventStream {
    pub fn new(peanut_gallery: Arc<Mutex<HashMap<usize, mpsc::Sender<Event>>>>) -> Self {
        static EVENT_STREAM_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = EVENT_STREAM_ID_COUNTER.fetch_add(1, Ordering::AcqRel);

        let (sender, receiver) = mpsc::channel(16);

        {
            let mut peanut_gallery = peanut_gallery.lock().unwrap();
            peanut_gallery.insert(id, sender);
        }

        EventStream {
            id,
            peanut_gallery,
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
