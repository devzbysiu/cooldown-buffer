use std::fmt::Debug;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use thread_timer::ThreadTimer;

#[derive(Debug, Clone)]
struct StartTimerMsg;

pub fn cooldown_buffer<T: Clone + Debug + Send>(
    cooldown_time: Duration,
) -> (Sender<T>, Receiver<Vec<T>>)
where
    T: 'static + Clone + Debug + Send,
{
    let (item_tx, item_rx) = channel::<T>();
    let (timer_tx, timer_rx) = channel::<StartTimerMsg>();
    let timer = ThreadTimer::new();
    let items = Arc::new(Mutex::new(Vec::new()));
    let (buffered_tx, buffered_rx) = channel::<Vec<T>>();

    let res = items.clone();
    std::thread::spawn(move || loop {
        let _ = timer_rx.recv();
        let _ = timer.cancel();
        let r = res.clone();
        let bx = buffered_tx.clone();
        let _ = timer.start(cooldown_time, move || {
            bx.send(r.lock().unwrap().clone()).unwrap();
            println!("[[-]]: flushing with {:?}", r.lock().unwrap());
            r.lock().unwrap().clear();
        });
    });

    std::thread::spawn(move || loop {
        if let Ok(num) = item_rx.recv() {
            println!("[[<<]]: get num: {:?}", num);
            timer_tx.send(StartTimerMsg).unwrap();
            items.lock().unwrap().push(num);
        }
    });

    (item_tx, buffered_rx)
}
