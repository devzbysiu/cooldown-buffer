use cooldown_buffer::cooldown_buffer;
// use doc_comment::doctest;
use std::thread;
use std::time::Duration;

#[test]
fn it_works() {
    // given
    let (tx, rx) = cooldown_buffer(Duration::from_millis(100));

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
