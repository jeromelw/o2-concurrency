use anyhow::Result;
use concurrency::AmapMetrics;
use rand::Rng;
use std::{thread, time::Duration};

const THREADS: usize = 2;

const REQUESTS: usize = 5;

fn main() -> Result<()> {
    let metrics = AmapMetrics::new(&[
        "call.thread.worker.0",
        "call.thread.worker.1",
        "req.page.1",
        "req.page.2",
        "req.page.3",
        "req.page.4",
        "req.page.5",
        "req.page.6",
        "req.page.7",
        "req.page.8",
        "req.page.9",
    ]);

    for i in 0..THREADS {
        thread_worker(i, metrics.clone())?;
    }

    for _ in 0..REQUESTS {
        request_worker(metrics.clone())?;
    }

    loop {
        thread::sleep(Duration::from_secs(2));
        println!("{}", metrics);
    }
}

fn thread_worker(idx: usize, metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..5000)));
            metrics.incr(format!("call.thread.worker.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    Ok(())
}

fn request_worker(metrics: AmapMetrics) -> Result<()> {
    thread::spawn(move || {
        loop {
            let mut rng = rand::thread_rng();
            thread::sleep(Duration::from_millis(rng.gen_range(100..800)));
            let idx = rng.gen_range(1..10);
            metrics.incr(format!("req.page.{}", idx))?;
        }
        #[allow(unreachable_code)]
        Ok::<(), anyhow::Error>(())
    });
    Ok(())
}
