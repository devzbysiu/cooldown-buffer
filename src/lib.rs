use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thread_timer::ThreadTimer;

#[derive(Debug, Clone)]
struct StartTimerMsg;

pub fn cooldown_buffer<T: Clone + Debug + Send>(
    cooldown_time: Duration,
) -> (CooldownSender<T>, Receiver<Vec<T>>)
where
    T: 'static + Clone + Debug + Send,
{
    let (item_tx, _) = channel::<T>();
    let (timer_tx, timer_rx) = channel::<StartTimerMsg>();
    let timer = ThreadTimer::new();
    let results = Arc::new(Mutex::new(Vec::new()));
    let (buffered_tx, buffered_rx) = channel::<Vec<T>>();

    std::thread::spawn(move || loop {
        let _ = timer_rx.recv();
        let _ = timer.cancel();
        let r = results.clone();
        let bx = buffered_tx.clone();
        let _ = timer.start(cooldown_time, move || {
            bx.send(r.lock().unwrap().clone()).unwrap();
            println!("[[-]]: flushing with {:?}", r.lock().unwrap());
            r.lock().unwrap().clear();
        });
    });

    (CooldownSender { item_tx, timer_tx }, buffered_rx)
}

pub struct CooldownSender<T> {
    item_tx: Sender<T>,
    timer_tx: Sender<StartTimerMsg>,
}

impl<T> CooldownSender<T>
where
    T: 'static + Clone + Debug + Send,
{
    pub fn send(&self, item: T) -> Result<(), std::sync::mpsc::SendError<T>> {
        self.timer_tx
            .send(StartTimerMsg)
            .expect("failed to send start timer message");
        self.item_tx.send(item)
    }
}
