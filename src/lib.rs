#![deny(missing_docs)]
#![doc(html_root_url = "https://docs.rs/cooldown-buffer/0.1.0")]

//! This library allows buffering items sent through the channel until no more action is happening
//! on this channel for a specified amount of time.
//!
//! # Example
//! ```
//! use std::time::Duration;
//! use cooldown_buffer::cooldown_buffer;
//! use std::thread;
//! use anyhow::Result;
//! use std::sync::mpsc::channel;
//!
//! fn main() -> Result<()> {
//!     // we set our buffer to cool down after 100 milliseconds
//!     let (tx, rx) = cooldown_buffer::<u32>(channel(), Duration::from_millis(100));
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
//! # Details
//!
//! It uses one thread and two channels. First channel is used to send and receive single items.
//! The Sender of this channel is handed to the user. Second channel is used to send and receive
//! vector of items (buffered items). Its Receiver is handed to the user.
//!
//! The thread receives items one at a time and pushes it to the vector. It then stops the timer
//! and starts it again each time (this lib is using [ThreadTimer](thread_timer::ThreadTimer)).
//! If the timer wasn't running then the error is ignored. If no more item appears for a specified
//! amount of time, the timer sends the buffered items via second channel and clears the vector to
//! make it ready for next buffering.

use doc_comment::doctest;
use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, RecvError, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thread_timer::ThreadTimer;

doctest!("../README.md");

/// Main and only function of this library. It starts the thread which receives the items, buffers
/// them and controls the timer. Returns tuple `(Sender<T>, Receiver<Vec<T>>)`. You can send single
/// item via `Sender<T>` and when the specified `cooldown_time` passes, you will get vector of
/// buffered items back via `Receiver<Vec<T>>`.
///
/// # Arguments
///
/// - `cooldown_time` - amount of time needed to "cool down" the receiving channel. After this time
/// passes, the buffered items are sent through the `Receiver`
#[must_use]
pub fn cooldown_buffer<T>(
    (item_tx, item_rx): (Sender<T>, Receiver<T>),
    cooldown_time: Duration,
) -> (Sender<T>, Receiver<Vec<T>>)
where
    T: 'static + Clone + Debug + Send,
{
    let timer = ThreadTimer::new();
    let items = Arc::new(Mutex::new(Vec::new()));
    let (buffered_tx, buffered_rx) = channel::<Vec<T>>();

    thread::spawn(move || -> Result<(), RecvError> {
        loop {
            let item = item_rx.recv()?;
            // I don't care if the cancel failed. It can fail only if there is no running
            // timer, which is fine from cancelling point of view - I just want
            // to have not running timer.
            let _ = timer.cancel();
            items.lock().expect("poisoned mutex").push(item);

            let cloned_items = items.clone();
            let btx = buffered_tx.clone();

            let _ = timer.start(cooldown_time, move || {
                btx.send(cloned_items.lock().expect("poisoned mutex").clone())
                    .expect("failed to send buffered items");
                cloned_items.lock().expect("poisoned mutex").clear();
            });
        }
    });

    (item_tx, buffered_rx)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_works() {
        // given
        let (tx, rx) = cooldown_buffer(channel(), Duration::from_millis(100));

        // when
        tx.send(1).unwrap();
        thread::sleep(Duration::from_millis(90));
        tx.send(2).unwrap();
        thread::sleep(Duration::from_millis(90));
        tx.send(3).unwrap();

        thread::sleep(Duration::from_millis(110)); // cooled down -> first buffer

        tx.send(4).unwrap();

        thread::sleep(Duration::from_millis(110)); // cooled down -> second buffer

        tx.send(5).unwrap();
        thread::sleep(Duration::from_millis(90));
        tx.send(6).unwrap();

        thread::sleep(Duration::from_millis(110)); // cooled down -> third buffer

        let buf1 = rx.recv().unwrap();
        let buf2 = rx.recv().unwrap();
        let buf3 = rx.recv().unwrap();

        tx.send(7).unwrap();
        tx.send(8).unwrap();
        // cooldown time didn't pass yet, so we won't have items in rx
        let res = rx.try_recv();

        // then
        assert_eq!(buf1, vec![1, 2, 3]);
        assert_eq!(buf2, vec![4]);
        assert_eq!(buf3, vec![5, 6]);
        assert!(res.is_err());
    }
}
