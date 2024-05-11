use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::atomic::AtomicI64;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use std::fmt;

#[derive(Debug)]
pub struct AmapMetrics {
    data: Arc<HashMap<&'static str, AtomicI64>>,
}

impl AmapMetrics {
    pub fn new(names: &[&'static str]) -> Self {
        let map = names
            .iter()
            .map(|&name| (name, AtomicI64::new(0)))
            .collect();
        Self {
            data: Arc::new(map),
        }
    }

    pub fn incr(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self.data.get(key).ok_or_else(|| anyhow!("Key not found"))?;
        counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    pub fn decr(&self, key: impl AsRef<str>) -> Result<()> {
        let key = key.as_ref();
        let counter = self.data.get(key).ok_or_else(|| anyhow!("Key not found"))?;
        counter.fetch_add(-1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

impl Clone for AmapMetrics {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
}

impl fmt::Display for AmapMetrics {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (key, value) in self.data.iter() {
            writeln!(f, "{}: {}", key, value.load(Ordering::Relaxed))?;
        }
        Ok(())
    }
}
