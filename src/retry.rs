use std::cmp::min;

pub trait RetryPolicy {
    fn next_delay(&self, attempts: i16) -> i64;
}

pub struct ExponentialBackoff {
    pub base: i64,
    pub max_delay: i64,
}

pub struct LinearBackoff {
    pub step: i64,
    pub max_delay: i64,
}

impl RetryPolicy for ExponentialBackoff {
    fn next_delay(&self, attempts: i16) -> i64 {
        let jitter: i64 = rand::random_range(0..10);
        let delay = self.base * 2i64.pow(attempts as u32) + jitter;
        min(delay, self.max_delay)
    }
}

impl RetryPolicy for LinearBackoff {
    fn next_delay(&self, attempts: i16) -> i64 {
        let jitter: i64 = rand::random_range(0..10);
        let delay = self.step * attempts as i64 + jitter;
        min(delay, self.max_delay)
    }
}
