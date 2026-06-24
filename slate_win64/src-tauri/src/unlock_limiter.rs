use std::sync::Mutex;
use std::time::{Duration, Instant};

const MAX_FAILED_ATTEMPTS: u32 = 5;
const LOCKOUT_DURATION: Duration = Duration::from_secs(30);
const ATTEMPT_RESET_AFTER: Duration = Duration::from_secs(300);

pub struct UnlockRateLimiter {
    failed_attempts: u32,
    locked_until: Option<Instant>,
    last_failure: Option<Instant>,
}

impl UnlockRateLimiter {
    pub fn new() -> Self {
        Self {
            failed_attempts: 0,
            locked_until: None,
            last_failure: None,
        }
    }

    fn maybe_reset_attempts(&mut self) {
        if let Some(last) = self.last_failure {
            if last.elapsed() > ATTEMPT_RESET_AFTER {
                self.failed_attempts = 0;
                self.locked_until = None;
            }
        }
    }

    pub fn check_allowed(&mut self) -> Result<(), String> {
        self.maybe_reset_attempts();

        if let Some(until) = self.locked_until {
            if Instant::now() < until {
                let secs = (until - Instant::now()).as_secs().max(1);
                return Err(format!(
                    "Too many failed attempts. Try again in {} seconds.",
                    secs
                ));
            }
            self.locked_until = None;
            self.failed_attempts = 0;
        }

        Ok(())
    }

    pub fn record_failure(&mut self) {
        self.maybe_reset_attempts();
        self.failed_attempts += 1;
        self.last_failure = Some(Instant::now());

        if self.failed_attempts >= MAX_FAILED_ATTEMPTS {
            self.locked_until = Some(Instant::now() + LOCKOUT_DURATION);
        }
    }

    pub fn record_success(&mut self) {
        self.failed_attempts = 0;
        self.locked_until = None;
        self.last_failure = None;
    }
}

pub type UnlockRateLimiterState = Mutex<UnlockRateLimiter>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lockout_after_five_failures() {
        let mut limiter = UnlockRateLimiter::new();
        for _ in 0..5 {
            limiter.record_failure();
        }
        assert!(limiter.check_allowed().is_err());
    }

    #[test]
    fn success_resets_lockout() {
        let mut limiter = UnlockRateLimiter::new();
        for _ in 0..5 {
            limiter.record_failure();
        }
        limiter.record_success();
        assert!(limiter.check_allowed().is_ok());
    }

    #[test]
    fn failures_below_threshold_allowed() {
        let mut limiter = UnlockRateLimiter::new();
        for _ in 0..4 {
            limiter.record_failure();
        }
        assert!(limiter.check_allowed().is_ok());
    }
}
