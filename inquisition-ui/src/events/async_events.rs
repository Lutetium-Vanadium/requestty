#[cfg(feature = "async-std")]
use async_std_dep::task::spawn_blocking;
#[cfg(feature = "smol")]
use smol_dep::unblock as spawn_blocking;
#[cfg(feature = "tokio")]
use tokio_dep::task::spawn_blocking;

use std::{
    pin::Pin,
    sync::{mpsc, Arc},
    task::{Context, Poll},
};

use futures::{task::AtomicWaker, Stream};

use crate::{error, events};

type Receiver = mpsc::Receiver<error::Result<events::KeyEvent>>;

pub struct AsyncEvents {
    events: Receiver,
    waker: Arc<AtomicWaker>,
}

impl AsyncEvents {
    pub async fn new() -> error::Result<Self> {
        let res = spawn_blocking(|| {
            let (tx, rx) = mpsc::sync_channel(16);
            let waker = Arc::new(AtomicWaker::new());
            let events = AsyncEvents {
                events: rx,
                waker: Arc::clone(&waker),
            };

            std::thread::spawn(move || {
                let events = super::Events::new();

                for event in events {
                    if tx.send(event).is_err() {
                        break;
                    }

                    waker.wake();
                }
            });

            events
        })
        .await;

        #[cfg(feature = "tokio")]
        return res.map_err(|_| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                "failed to spawn event thread",
            )
            .into()
        });

        #[cfg(not(feature = "tokio"))]
        Ok(res)
    }
}

impl Stream for AsyncEvents {
    type Item = error::Result<events::KeyEvent>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.events.try_recv() {
            Ok(e) => Poll::Ready(Some(e)),
            Err(mpsc::TryRecvError::Empty) => {
                self.waker.register(cx.waker());
                Poll::Pending
            }
            Err(mpsc::TryRecvError::Disconnected) => unreachable!(),
        }
    }
}
