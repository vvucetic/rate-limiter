use std::time::Instant;
use std::convert::TryInto;
use std::cmp::min;
use std::collections::HashMap;

pub struct Bucket {
    max_amount: i32,
    refill_time: i32,
    refill_amount: i32,
    available_tokens: i32,
    last_updated: Instant,
}

impl Bucket {
    pub fn new(max_amount: i32, refill_time: i32, refill_amount: i32) -> Bucket {
        Bucket {
            max_amount,
            refill_amount,
            refill_time,
            available_tokens: max_amount,
            last_updated: Instant::now(),
        }
    }

    /// Reset bucket available tokens to `max_amount`
    pub fn reset(&mut self) {
        self.available_tokens = self.max_amount;
        self.last_updated = Instant::now();
    }

    fn get_refill_tokens(&self) -> i32 {
        let since_last: i32 = self.last_updated.elapsed().as_secs().try_into().unwrap();
        since_last / self.refill_time * self.refill_amount
    }

    /// Get available tokens
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rate_limiter::bucket;
    /// let bucket = bucket::Bucket::new(5, 2, 1);
    /// assert_eq!(bucket.get_available_tokens(), 5);
    /// ```
    pub fn get_available_tokens(&self) -> i32 {
        min(
            self.max_amount,
            self.available_tokens + self.get_refill_tokens()
        )
    }

    /// Tries reducing `token` tokens from `available_tokens`. If tokens available, return `true`,
    /// otherwise return `false`
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rate_limiter::bucket;
    /// let mut bucket = bucket::Bucket::new(5, 1, 1);
    /// assert!(!bucket.reduce(6));
    /// assert!(bucket.reduce(1));
    /// ```
    pub fn reduce(&mut self, tokens: i32) -> bool {
        let refill_tokens = self.get_refill_tokens();
        self.available_tokens += refill_tokens;
        if self.available_tokens > self.max_amount {
            self.reset();
        }
        if tokens > self.available_tokens {
            return false;
        }
        self.available_tokens -= tokens;
        self.last_updated = Instant::now();
        true
    }
}
