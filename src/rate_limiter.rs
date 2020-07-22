use crate::bucket;
use std::{
    collections::HashMap,
    sync::{Mutex, RwLock},
};

#[derive(Debug)]
pub struct RateLimiter {
    default_max_amount: i32,
    default_refill_time: i32,
    default_refill_amount: i32,
    buckets: HashMap<String, bucket::Bucket>,
}

impl RateLimiter {
    /// Initialize RateLimiter with default parameters used when bucket for particular key is not
    /// present.
    pub fn new(
        default_max_amount: i32,
        default_refill_time: i32,
        default_refill_amount: i32,
    ) -> RateLimiter {
        RateLimiter {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: HashMap::new(),
        }
    }

    /// Tries reducing tokens in bucket for particular key. Returns (success, available_tokens)
    /// tuple. Success is `false` if there is not enough tokens, otherwise `true`. If
    /// success was `false`, tokens weren't removed.
    /// If key is not present in rate limiter, bucket is added with default parameters.
    ///
    /// # Examples
    /// ```
    /// use rate_limiter;
    /// let mut rate_limiter = rate_limiter::RateLimiter::new(5, 2, 1);
    /// assert!(rate_limiter.reduce(String::from("some key"), 5).0);
    /// assert!(!rate_limiter.reduce(String::from("some key"), 1).0);
    ///
    /// assert!(rate_limiter.reduce(String::from("some other key"), 5).0);
    /// assert!(!rate_limiter.reduce(String::from("some other key"), 1).0);
    /// ```
    pub fn reduce(&mut self, key: String, reduce_tokens: i32) -> (bool, i32) {
        if self.buckets.contains_key(&key) {
            return self.buckets.get_mut(&key).unwrap().reduce(reduce_tokens);
        }
        let mut bucket = bucket::Bucket::new(
            self.default_max_amount,
            self.default_refill_time,
            self.default_refill_amount,
        );
        let result = bucket.reduce(reduce_tokens);
        self.buckets.insert(key, bucket);
        result
    }
}

#[derive(Debug)]
pub struct AtomicRateLimiter {
    default_max_amount: i32,
    default_refill_time: i32,
    default_refill_amount: i32,
    buckets: RwLock<HashMap<String, Mutex<bucket::Bucket>>>,
}

impl AtomicRateLimiter {
    /// Initialize AtomicRateLimiter with default parameters used when bucket for particular key is 
    /// not present.
    pub fn new(
        default_max_amount: i32,
        default_refill_time: i32,
        default_refill_amount: i32,
    ) -> AtomicRateLimiter {
        AtomicRateLimiter {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Tries reducing tokens in bucket for particular key. Returns (success, available_tokens)
    /// tuple. Success is `false` if there is not enough tokens, otherwise `true`. If
    /// success was `false`, tokens weren't removed.
    /// If key is not present in rate limiter, bucket is added with default parameters.

    pub fn reduce(&self, key: String, reduce_tokens: i32) -> (bool, i32) {
        // assume bucket exists so lock HashMap for reading
        let buckets = self.buckets.read().expect("RWLock poisoned.");
        if let Some(bucket) = buckets.get(&key) {
            let mut bucket = bucket.lock().expect("Mutex poisoned");
            return bucket.reduce(reduce_tokens);
        }
        // drop read lock
        drop(buckets);
        let mut bucket = bucket::Bucket::new(
                self.default_max_amount,
                self.default_refill_time,
                self.default_refill_amount,
            );
        let result = bucket.reduce(reduce_tokens);
        let mut buckets = self.buckets.write().expect("RWLock poisoned.");
        // "upsert" in case other thread got here before
        buckets.entry(key).or_insert_with(|| Mutex::new(bucket));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::Arc,
        thread,
    };

    #[test]
    fn test_reducing_tokens_atomic() {
        let data = Arc::new(AtomicRateLimiter::new(5,1,1));

        let threads: Vec<_> = (0..10)
            .map(|_| {
                let data = Arc::clone(&data);
                thread::spawn(move || data.reduce(String::from("test"), 1))
            })
            .collect();
    
        for t in threads {
            t.join().expect("Thread panicked");
        }
    }
}