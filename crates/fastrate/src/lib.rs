use pyo3::prelude::*;
use std::time::{Duration, Instant};

#[pyclass(module = "fastrate")]
pub struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_interval: Duration,
    last_refill: Instant,
}

#[pymethods]
impl TokenBucket {
    #[new]
    fn new(capacity: u64, refill_rate: u64) -> Self {
        let rate = refill_rate.max(1);
        Self {
            capacity,
            tokens: capacity,
            refill_interval: Duration::from_secs_f64(1.0 / rate as f64),
            last_refill: Instant::now(),
        }
    }

    fn acquire(&mut self) -> PyResult<bool> {
        let now = Instant::now();
        let elapsed = now.saturating_duration_since(self.last_refill);
        let increments = (elapsed.as_secs_f64() / self.refill_interval.as_secs_f64()).floor() as u64;
        if increments > 0 {
            self.tokens = (self.tokens + increments).min(self.capacity);
            self.last_refill = now;
        }
        if self.tokens > 0 {
            self.tokens -= 1;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

#[pymodule]
fn _fastrate(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<TokenBucket>()?;
    Ok(())
}
