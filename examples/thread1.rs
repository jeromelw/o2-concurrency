use anyhow::{anyhow, Result};
use std::{sync::mpsc, thread};

const NUM_PRODUCERS: usize = 4;

#[allow(dead_code)]
#[derive(Debug)]
struct Msg {
    idx: usize,
    value: usize,
}

impl Msg {
    fn new(idx: usize, value: usize) -> Self {
        Self { idx, value }
    }
}

fn main() -> Result<()> {
    println!("Hello, world!");
    let (tx, rx) = mpsc::channel();
    for idx in 0..NUM_PRODUCERS {
        let tx = tx.clone();
        thread::spawn(move || producer(idx, tx));
    }
    drop(tx);

    let handle = thread::spawn(move || consumer(rx));
    handle
        .join()
        .map_err(|e| anyhow!("Thread join error : {:?}", e))?;
    Ok(())
}

fn producer(idx: usize, tx: mpsc::Sender<Msg>) -> Result<()> {
    loop {
        let value = rand::random::<usize>();
        tx.send(Msg::new(idx, value))?;
        let sleep_time = rand::random::<u8>() as u64 * 10;
        thread::sleep(std::time::Duration::from_millis(sleep_time));

        if rand::random::<u8>() % 10 == 0_u8 {
            println!("Producer {} exiting", idx);
            break;
        }
    }
    Ok(())
}

fn consumer(rx: mpsc::Receiver<Msg>) {
    for msg in rx {
        println!("Received: {:?}", msg);
    }
    println!("Consumer exiting");
}
