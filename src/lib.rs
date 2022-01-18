use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use thread_timer::ThreadTimer;

pub fn cooldown_buffer<T: Clone + Debug + Send>(
    cooldown_time: Duration,
) -> (Sender<T>, Receiver<Vec<T>>)
where
    T: 'static + Clone + Debug + Send,
{
    let (item_tx, item_rx) = channel::<T>();
    let (timer_tx, timer_rx) = channel::<()>();
    let timer = ThreadTimer::new();
    let items = Arc::new(Mutex::new(Vec::new()));
    let (buffered_tx, buffered_rx) = channel::<Vec<T>>();

    let cloned_items = items.clone();
    thread::spawn(move || loop {
        let _ = timer_rx.recv();
        let _ = timer.cancel();
        let items = cloned_items.clone();
        let bx = buffered_tx.clone();
        let _ = timer.start(cooldown_time, move || {
            bx.send(items.lock().expect("poisoned mutex").clone())
                .expect("failed to send buffered items");
            items.lock().expect("poisoned mutex").clear();
        });
    });

    thread::spawn(move || loop {
        if let Ok(item) = item_rx.recv() {
            timer_tx.send(()).unwrap();
            items.lock().expect("poisoned mutex").push(item);
        }
    });

    (item_tx, buffered_rx)
}
