// #![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/cooldown-buffer/0.1.0")]

//! This library allows to buffer items sent through the channel until no more action is happening
//! on this channel for a specified amount of time.
//!
//! # Details
//!
//! It uses two threads and three channels to communicate between threads. First channel is used to
//! send and receive single items. The Sender of this channel is handed to the user. Second
//! channel is used to send and receive vector of items (buffered items). It's Receiver is handed
//! to the user. Third channel is used to communicate between those two threads, let's call
//! it the controll channel.
//!
//! First thread receives items one at a time and pushes it to the vector (shared between threads).
//! It then uses controll channel to inform the second thread that the item was received.
//!
//! Second thread starts the timer using [ThreadTimer](thread_timer::ThreadTimer) every time, it
//! gets the signal from the first thread via controll channel. When the time is up, the timer
//! sends the cloned vector via third channel and clears the original one to make it ready for next
//! buffering.
//!
//! # Example
//! ```
//! use std::time::Duration;
//! use cooldown_buffer::cooldown_buffer;
//! use std::thread;
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     // we set our buffer to cool down after 100 milliseconds
//!     let (tx, rx) = cooldown_buffer::<u32>(Duration::from_millis(100));
//!
//!     thread::spawn(move || -> Result<()> {
//!       // then we send 4 items with delay of 90 milliseconds between each,
//!       // this means that our buffer kept buffering those items because it didn't
//!       // have a time to cool down
//!
//!       tx.send(1)?;
//!       thread::sleep(Duration::from_millis(90));
//!       tx.send(2)?;
//!       thread::sleep(Duration::from_millis(90));
//!       tx.send(3)?;
//!       thread::sleep(Duration::from_millis(90));
//!       tx.send(4)?;
//!
//!       Ok(())
//!     });
//!
//!     // now we are allowing it to cool down by waiting more than 100 milliseconds,
//!     // so it will send all buffered items at once
//!     thread::sleep(Duration::from_millis(200));
//!
//!     let buffered = rx.recv()?;
//!     assert_eq!(buffered.len(), 4);
//!
//!     Ok(())
//! }
//! ```

use doc_comment::doctest;
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, RecvError, SendError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thiserror::Error;
use thread_timer::ThreadTimer;

doctest!("../README.md");

/// Specifies possible errors. Errors happen only when there is an issue with receiving single
/// item, or if there is an issue with sending buffered items.
#[derive(Debug, Error)]
pub enum CooldownError {
    /// Happens when the receiving thread couldn't receive an item. This is just the wrapper over
    /// [RecvError](std::sync::mpsc::RecvError). This error can occur for the same reasons as in
    /// case of `RecvError`.
    #[error("failed to receive item")]
    ItemRecvError(#[from] RecvError),

    #[error("failed to send item")]
    ItemSendError(#[from] SendError<()>),
}

#[must_use]
pub fn cooldown_buffer<T>(cooldown_time: Duration) -> (Sender<T>, Receiver<Vec<T>>)
where
    T: 'static + Clone + Debug + Send,
{
    let (item_tx, item_rx) = channel::<T>();
    let (timer_tx, timer_rx) = channel::<()>();
    let timer = ThreadTimer::new();
    let items = Arc::new(Mutex::new(Vec::new()));
    let (buffered_tx, buffered_rx) = channel::<Vec<T>>();

    let cloned_items = items.clone();
    thread::spawn(move || -> Result<(), CooldownError> {
        loop {
            timer_rx.recv()?;

            // I don't care if the cancel failed. It can fail only if there is no running
            // timer, which is fine from cancelling point of view - I just want
            // to have not running timer.
            let _ = timer.cancel();
            let items = cloned_items.clone();
            let btx = buffered_tx.clone();
            let _ = timer.start(cooldown_time, move || {
                btx.send(items.lock().expect("poisoned mutex").clone())
                    .expect("failed to send buffered items");
                items.lock().expect("poisoned mutex").clear();
            });
        }
    });

    thread::spawn(move || -> Result<(), CooldownError> {
        loop {
            if let Ok(item) = item_rx.recv() {
                timer_tx.send(())?;
                items.lock().expect("poisoned mutex").push(item);
            }
        }
    });

    (item_tx, buffered_rx)
}
