use parking_lot::Mutex;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

const NANOS_PER_SECOND: u128 = 1_000_000_000;

fn now_ns(py: Python<'_>, time_source: Option<&Py<PyAny>>) -> PyResult<u64> {
    if let Some(source) = time_source {
        let value = source.call0(py)?;
        let seconds: f64 = value.extract(py)?;
        if seconds.is_nan() || seconds.is_sign_negative() {
            return Err(PyValueError::new_err("time source returned invalid value"));
        }
        return Ok((seconds * 1_000_000_000.0) as u64);
    }
    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|_| PyValueError::new_err("system time before unix epoch"))?;
    Ok(duration.as_nanos() as u64)
}

#[pyclass(module = "fastrate")]
pub struct TokenBucket {
    capacity: u64,
    refill_per_sec: u64,
    tokens: AtomicU64,
    last_refill_ns: AtomicU64,
    fractional_nanos: AtomicU64,
    time_source: Option<Py<PyAny>>,
}

impl TokenBucket {
    fn refill(&self, py: Python<'_>) -> PyResult<()> {
        if self.refill_per_sec == 0 {
            return Ok(());
        }

        let now = now_ns(py, self.time_source.as_ref())?;
        let mut last = self.last_refill_ns.load(Ordering::SeqCst);
        loop {
            if now <= last {
                return Ok(());
            }

            let elapsed = now - last;
            let fractional = self.fractional_nanos.load(Ordering::SeqCst) as u128;
            let produced = (elapsed as u128)
                .saturating_mul(self.refill_per_sec as u128)
                .saturating_add(fractional);
            let tokens_to_add = (produced / NANOS_PER_SECOND) as u64;
            let remainder = (produced % NANOS_PER_SECOND) as u64;

            if self
                .last_refill_ns
                .compare_exchange(last, now, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                self.fractional_nanos.store(remainder, Ordering::SeqCst);

                if tokens_to_add > 0 {
                    self.tokens
                        .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |current| {
                            let updated = (current + tokens_to_add).min(self.capacity);
                            Some(updated)
                        })
                        .ok();
                }
                return Ok(());
            }
            last = self.last_refill_ns.load(Ordering::SeqCst);
        }
    }
}

#[pymethods]
impl TokenBucket {
    #[new]
    #[pyo3(signature = (capacity, refill_per_sec, *, time_source=None))]
    fn new(capacity: u64, refill_per_sec: u64, time_source: Option<Py<PyAny>>) -> PyResult<Self> {
        if capacity == 0 {
            return Err(PyValueError::new_err("capacity must be > 0"));
        }
        let now = Python::with_gil(|py| now_ns(py, time_source.as_ref()))?;
        Ok(Self {
            capacity,
            refill_per_sec,
            tokens: AtomicU64::new(capacity),
            last_refill_ns: AtomicU64::new(now),
            fractional_nanos: AtomicU64::new(0),
            time_source,
        })
    }

    #[pyo3(signature = (n=1))]
    fn acquire(&self, py: Python<'_>, n: u64) -> PyResult<bool> {
        if n == 0 {
            return Ok(true);
        }
        if n > self.capacity {
            return Ok(false);
        }
        self.refill(py)?;
        loop {
            let current = self.tokens.load(Ordering::SeqCst);
            if current < n {
                return Ok(false);
            }
            if self
                .tokens
                .compare_exchange(current, current - n, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(true);
            }
        }
    }

    fn remaining(&self, _py: Python<'_>) -> u64 {
        self.tokens.load(Ordering::SeqCst)
    }
}

#[pyclass(module = "fastrate")]
pub struct SlidingWindow {
    max_events: usize,
    window_ms: u64,
    events: Mutex<VecDeque<u64>>,
    time_source: Option<Py<PyAny>>,
}

#[pymethods]
impl SlidingWindow {
    #[new]
    #[pyo3(signature = (max_events, window_ms, *, time_source=None))]
    fn new(max_events: usize, window_ms: u64, time_source: Option<Py<PyAny>>) -> PyResult<Self> {
        if max_events == 0 || window_ms == 0 {
            return Err(PyValueError::new_err(
                "max_events and window_ms must be > 0",
            ));
        }
        Ok(Self {
            max_events,
            window_ms,
            events: Mutex::new(VecDeque::new()),
            time_source,
        })
    }

    fn try_add(&self, py: Python<'_>) -> PyResult<bool> {
        let now_ns = now_ns(py, self.time_source.as_ref())?;
        let now_ms = now_ns / 1_000_000;
        let mut events = self.events.lock();
        while let Some(&front) = events.front() {
            if now_ms - front > self.window_ms {
                events.pop_front();
            } else {
                break;
            }
        }
        if events.len() >= self.max_events {
            return Ok(false);
        }
        events.push_back(now_ms);
        Ok(true)
    }
}

#[pymodule]
fn _fastrate(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<TokenBucket>()?;
    m.add_class::<SlidingWindow>()?;
    Ok(())
}
