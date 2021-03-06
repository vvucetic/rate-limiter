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
    ) -> Self {
        Self {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: HashMap::new(),
        }
    }

    /// Returns `available_tokens` in bucket for given key. If bucket is not found, it returns
    /// `default_max_amount`.
    ///
    /// # Examples
    /// ```
    /// use rate_limiter;
    /// let mut rate_limiter = rate_limiter::RateLimiter::new(5, 1, 1);
    /// rate_limiter.reduce(String::from("some key"), 1);
    /// assert_eq!(rate_limiter.get_available_tokens(String::from("some key")), 4);
    /// ```
    pub fn get_available_tokens(&self, key: String) -> i32 {
        match self.buckets.get(&key) {
            Some(bucket) => bucket.get_available_tokens(),
            None => self.default_max_amount,
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
    ) -> Self {
        Self {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: RwLock::new(HashMap::new()),
        }
    }

    /// Returns `available_tokens` in bucket for given key. If bucket is not found, it returns
    /// `default_max_amount`.
    ///
    /// # Examples
    /// ```
    /// use rate_limiter;
    /// let mut rate_limiter = rate_limiter::AtomicRateLimiter::new(5, 1, 1);
    /// rate_limiter.reduce(String::from("some key"), 1);
    /// assert_eq!(rate_limiter.get_available_tokens(String::from("some key")), 4);
    /// ```
    pub fn get_available_tokens(&self, key: String) -> i32 {
        let buckets = self.buckets.read().expect("RWLock poisoned.");
        match buckets.get(&key) {
            Some(bucket) => bucket
                .lock()
                .expect("Mutex poisoned")
                .get_available_tokens(),
            None => self.default_max_amount,
        }
    }

    /// Tries reducing tokens in bucket for particular key. Returns (success, available_tokens)
    /// tuple. Success is `false` if there is not enough tokens, otherwise `true`. If
    /// success was `false`, tokens weren't removed.
    /// If key is not present in rate limiter, bucket is added with default parameters.
    /// # Examples
    /// ```
    /// use rate_limiter;
    /// let mut rate_limiter = rate_limiter::AtomicRateLimiter::new(5, 2, 1);
    /// assert!(rate_limiter.reduce(String::from("some key"), 5).0);
    /// assert!(!rate_limiter.reduce(String::from("some key"), 1).0);
    ///
    /// assert!(rate_limiter.reduce(String::from("some other key"), 5).0);
    /// assert!(!rate_limiter.reduce(String::from("some other key"), 1).0);
    /// ```
    pub fn reduce(&self, key: String, reduce_tokens: i32) -> (bool, i32) {
        // assume bucket exists so lock HashMap for reading
        let buckets = self.buckets.read().expect("RWLock poisoned.");
        if let Some(bucket) = buckets.get(&key) {
            let mut bucket = bucket.lock().expect("Mutex poisoned");
            return bucket.reduce(reduce_tokens);
        }
        // drop read lock
        drop(buckets);
        // make write lock
        let mut buckets = self.buckets.write().expect("RWLock poisoned.");
        // try reducing again this time with write lock
        if let Some(bucket) = buckets.get(&key) {
            let mut bucket = bucket.lock().expect("Mutex poisoned");
            return bucket.reduce(reduce_tokens);
        }
        // if still no key, insert one
        let mut bucket = bucket::Bucket::new(
            self.default_max_amount,
            self.default_refill_time,
            self.default_refill_amount,
        );
        let result = bucket.reduce(reduce_tokens);
        buckets.insert(key, Mutex::new(bucket));
        result
    }
}

#[cfg(feature = "async")]
#[derive(Debug)]
pub struct AsyncAtomicRateLimiter {
    default_max_amount: i32,
    default_refill_time: i32,
    default_refill_amount: i32,
    buckets: tokio::sync::RwLock<HashMap<String, Mutex<bucket::Bucket>>>,
}

impl AsyncAtomicRateLimiter {
    /// Initialize AtomicRateLimiter with default parameters used when bucket for particular key is
    /// not present.
    pub fn new(
        default_max_amount: i32,
        default_refill_time: i32,
        default_refill_amount: i32,
    ) -> Self {
        Self {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Returns `available_tokens` in bucket for given key. If bucket is not found, it returns
    /// `default_max_amount`.
    ///
    /// # Examples
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     use rate_limiter;
    ///     let mut rate_limiter = rate_limiter::AsyncAtomicRateLimiter::new(5, 1, 1);
    ///     rate_limiter.reduce(String::from("some key"), 1).await;
    ///     assert_eq!(rate_limiter.get_available_tokens(String::from("some key")).await, 4);
    /// }
    /// ```
    pub async fn get_available_tokens(&self, key: String) -> i32 {
        let buckets = self.buckets.read().await;
        match buckets.get(&key) {
            Some(bucket) => bucket
                .lock()
                .expect("Mutex poisoned")
                .get_available_tokens(),
            None => self.default_max_amount,
        }
    }

    /// Tries reducing tokens in bucket for particular key. Returns (success, available_tokens)
    /// tuple. Success is `false` if there is not enough tokens, otherwise `true`. If
    /// success was `false`, tokens weren't removed.
    /// If key is not present in rate limiter, bucket is added with default parameters.
    /// # Examples
    /// ```
    /// #[tokio::main]
    /// async fn main() {
    ///     use rate_limiter;
    ///     let mut rate_limiter = rate_limiter::AsyncAtomicRateLimiter::new(5, 2, 1);
    ///     assert!(rate_limiter.reduce(String::from("some key"), 5).await.0);
    ///     assert!(!rate_limiter.reduce(String::from("some key"), 1).await.0);
    ///
    ///     assert!(rate_limiter.reduce(String::from("some other key"), 5).await.0);
    ///     assert!(!rate_limiter.reduce(String::from("some other key"), 1).await.0);
    /// }
    /// ```
    pub async fn reduce(&self, key: String, reduce_tokens: i32) -> (bool, i32) {
        // assume bucket exists so lock HashMap for reading
        let buckets = self.buckets.read().await;
        if let Some(bucket) = buckets.get(&key) {
            let mut bucket = bucket.lock().expect("Mutex poisoned");
            return bucket.reduce(reduce_tokens);
        }
        // drop read lock
        drop(buckets);
        // make write lock
        let mut buckets = self.buckets.write().await;
        // try reducing again this time with write lock
        if let Some(bucket) = buckets.get(&key) {
            let mut bucket = bucket.lock().expect("Mutex poisoned");
            return bucket.reduce(reduce_tokens);
        }
        // if still no key, insert one
        let mut bucket = bucket::Bucket::new(
            self.default_max_amount,
            self.default_refill_time,
            self.default_refill_amount,
        );
        let result = bucket.reduce(reduce_tokens);
        buckets.insert(key, Mutex::new(bucket));
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{sync::Arc, thread};

    #[test]
    fn test_reducing_tokens_atomic() {
        let data = Arc::new(AtomicRateLimiter::new(30, 1, 1));

        let threads: Vec<_> = (0..10)
            .map(|_| {
                let data = Arc::clone(&data);
                thread::spawn(move || data.reduce(String::from("test"), 1))
            })
            .collect();

        for t in threads {
            t.join().expect("Thread panicked");
        }

        assert_eq!(data.get_available_tokens(String::from("test")), 20);
    }

    #[cfg(feature = "async")]
    #[tokio::test]
    async fn test_reducing_tokens_async_atomic() {
        let data = Arc::new(AsyncAtomicRateLimiter::new(30, 1, 1));

        let threads: Vec<_> = (0..10)
            .map(|_| {
                let data = Arc::clone(&data);
                tokio::spawn(async move {
                    data.reduce(String::from("test"), 1).await;
                })
            })
            .collect();

        for t in threads {
            t.await.unwrap();
        }

        assert_eq!(data.get_available_tokens(String::from("test")).await, 20);
    }
}
