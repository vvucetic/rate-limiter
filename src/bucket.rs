use std::time::Instant;
use std::convert::TryInto;
use std::cmp::min;

pub struct Bucket {
    max_amount: i32,
    refill_time: i32,
    refill_amount: i32,
    available_tokens: i32,
    last_updated: Instant,
}

impl Bucket {
    /// Initialize new bucket.
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

    /// Tries reducing tokens in bucket for particular key. Returns (success, available_tokens)
    /// tuple. Success is `false` if there is not enough tokens, otherwise `true`. If
    /// success was `false`, tokens weren't removed.
    /// 
    /// # Examples
    /// 
    /// ```
    /// use rate_limiter::bucket;
    /// let mut bucket = bucket::Bucket::new(5, 1, 1);
    /// // reducing more tokens than available returns false
    /// let (success, available_tokens) = bucket.reduce(6);
    /// assert!(!success);
    /// assert_eq!(available_tokens, 5);
    /// 
    /// // reducing fewer tokens than available, returns true
    /// let (success, available_tokens) = bucket.reduce(1);
    /// assert!(success);
    /// assert_eq!(available_tokens, 4);
    /// ```
    pub fn reduce(&mut self, tokens: i32) -> (bool, i32) {
        let refill_tokens = self.get_refill_tokens();
        self.available_tokens += refill_tokens;
        if self.available_tokens > self.max_amount {
            self.reset();
        }
        if tokens > self.available_tokens {
            return (false, self.available_tokens);
        }
        self.available_tokens -= tokens;
        self.last_updated = Instant::now();
        (true, self.available_tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time};

    #[test]
    fn test_reducing_tokens() {
        let mut bucket = Bucket::new(5, 1, 1);
        let (success, available_tokens) = bucket.reduce(6);
        // this should return false because we can't remove 6 tokens when only 5 is available
        assert!(!success);
        // available tokens should remain untouched
        assert_eq!(available_tokens, 5);

        // try with fewer tokens to reduce
        let (success, available_tokens) = bucket.reduce(1);
        // this should return true
        assert!(success);
        assert_eq!(available_tokens, 4);
    }

    #[test]
    fn test_refilling_tokens_max() {
        let max_tokens = 5;
        let mut bucket = Bucket::new(max_tokens, 1, 1);
        // reduce 1 token
        bucket.reduce(1);
        // wait 2 seconds
        thread::sleep(time::Duration::from_secs(2));
        // ensure bucket has maximum number of tokens (and not more)
        assert_eq!(bucket.get_available_tokens(), max_tokens)
    }

    #[test]
    fn test_refill_time() {
        let mut bucket = Bucket::new(5, 2, 1);
        // reduce to 0
        bucket.reduce(5);
        // wait 2 seconds
        thread::sleep(time::Duration::from_secs(2));
        // ensure we got new token available
        assert_eq!(bucket.get_available_tokens(), 1)
    }

    #[test]
    fn test_refill_amount() {
        let mut bucket = Bucket::new(5, 1, 2);
        // reduce to 0
        bucket.reduce(5);
        // wait 1 second
        thread::sleep(time::Duration::from_secs(1));
        // ensure we got 2 new tokens available
        assert_eq!(bucket.get_available_tokens(), 2)
    }
}