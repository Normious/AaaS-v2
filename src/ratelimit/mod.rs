use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Bucket {
    tokens: f64,
    max: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        let limiter = Self {
            buckets: Arc::new(Mutex::new(HashMap::new())),
        };
        limiter.clone().start_cleanup();
        limiter
    }

    /// Returns Ok(()) if allowed, Err(retry_after_seconds) if not.
    pub fn check(&self, key: &str, max_per_window: u32, window_seconds: u64) -> Result<(), u64> {
        if max_per_window == 0 {
            return Ok(());
        }

        let mut buckets = self.buckets.lock().unwrap();
        let now = Instant::now();

        let b = buckets.entry(key.to_string()).or_insert_with(|| Bucket {
            tokens: max_per_window as f64,
            max: max_per_window as f64,
            refill_per_sec: max_per_window as f64 / window_seconds as f64,
            last_refill: now,
        });

        // Refill
        let elapsed = now.duration_since(b.last_refill).as_secs_f64();
        b.tokens = (b.tokens + elapsed * b.refill_per_sec).min(b.max);
        b.last_refill = now;

        if b.tokens >= 1.0 {
            b.tokens -= 1.0;
            Ok(())
        } else {
            let needed = 1.0 - b.tokens;
            let wait = (needed / b.refill_per_sec).ceil() as u64;
            Err(wait.max(1))
        }
    }

    fn start_cleanup(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300));
            loop {
                interval.tick().await;
                let mut buckets = self.buckets.lock().unwrap();
                let now = Instant::now();
                buckets
                    .retain(|_, b| now.duration_since(b.last_refill) < Duration::from_secs(600));
            }
        });
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_up_to_limit_then_blocks() {
        let rl = RateLimiter::new();
        assert!(rl.check("test-key", 2, 60).is_ok());
        assert!(rl.check("test-key", 2, 60).is_ok());
        assert!(rl.check("test-key", 2, 60).is_err());
    }
}
