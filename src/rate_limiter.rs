use std::collections::HashMap;
use crate::bucket;

pub struct RateLimiter {
    default_max_amount: i32,
    default_refill_time: i32,
    default_refill_amount: i32,
    buckets: HashMap<String, bucket::Bucket>,
}

impl RateLimiter {
    pub fn new(
            default_max_amount: i32,
            default_refill_time: i32,
            default_refill_amount: i32) -> RateLimiter {
        RateLimiter {
            default_max_amount,
            default_refill_time,
            default_refill_amount,
            buckets: HashMap::new(),
        }
    }

    pub fn reduce(&mut self, key: String, reduce_tokens: i32) -> bool {
        if self.buckets.contains_key(&key) {
            return self.buckets.get_mut(&key).unwrap().reduce(reduce_tokens);
        }
        let mut bucket = bucket::Bucket::new(
            self.default_max_amount,
            self.default_refill_time,
            self.default_refill_amount);
        let result = bucket.reduce(reduce_tokens);
        self.buckets.insert(
            key,
            bucket);
        result
    }
}
