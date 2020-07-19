pub mod bucket;
pub mod rate_limiter;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut bucket = bucket::Bucket::new(5, 1, 1);
        // this should return false because we can't remove 6 tokens when only 5 is available
        assert!(!bucket.reduce(6));
        // this should return true
        assert!(bucket.reduce(1));
    }
}
