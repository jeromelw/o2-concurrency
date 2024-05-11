use std::{thread, time::Duration};

use anyhow::Result;
use concurrency::CmapMetrics;
use rand::Rng;

const THREADS: usize = 2;

const REQUESTS: usize = 5;

fn main() -> Result<()> {
    let metrics = CmapMetrics::new();

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

fn thread_worker(idx: usize, metrics: CmapMetrics) -> Result<()> {
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

fn request_worker(metrics: CmapMetrics) -> Result<()> {
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
