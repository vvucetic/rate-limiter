# rate_limiter
![Continuous integration](https://github.com/vvucetic/rate-limiter/workflows/Continuous%20integration/badge.svg?branch=master)

`rate_limiter` is in-memory rate limiter for generic purposes. It implements leaky bucket/token bucket algorithm with the following characteristics:
- each bucket has maximum number of tokens at the beginning (max capacity, `max_amount`) and can not hold more than that
- x tokens (`refill_amount`) are added every y seconds (`refill_time`)
- if a token arrives when the bucket is full, it is discarded.
- only available tokens can be used or "reduced". If no available tokens to reduce, operation can not be performed. It's what is called rate-limiting.

Developer can use this library to rate-limit different operations, for example rate-limit writing logs of the same type (or we call it key in this library)

## Examples

For example, to rate-limit sending some messages for some user we can implement rate limiter as in this example:

```rust
use rate_limiter;
let mut rate_limiter = rate_limiter::RateLimiter::new(5, 1, 1);
let (success, available_tokens) = rate_limiter.reduce(String::from("send-message-user-id-1"), 1);
if success {
    // send_message();
}
```

We created rate-limiter which will hold unlimited number of buckets, each uniquely identified by `key`. Every bucket will have max capacity of 5 tokens and refill of 1 token every 1 second. Sending one message requires 1 token.
In practice this means that user can send 5 messages at once and then 1 every 1 second because that's the refill speed. By changing those 3 parameters, developer can tweak bursting and refilling for desired output. If `success` was true, message sending can be allowed, otherwise, rate-limiting should be applied.



### Thread-safe rate_limiter

Thread-safe rate_limiter (`AtomicRateLimiter`) can be used across multiple threads like in the example below:

```rust
let atomic_rate_limiter = Arc::new(AtomicRateLimiter::new(30, 1, 1));

let threads: Vec<_> = (0..10)
    .map(|_| {
        let atomic_rate_limiter = Arc::clone(&atomic_rate_limiter);
        thread::spawn(move || atomic_rate_limiter.reduce(String::from("test"), 1))
    })
    .collect();

for t in threads {
    t.join().expect("Thread panicked");
}

assert_eq!(atomic_rate_limiter.get_available_tokens(String::from("test")), 20);
```
Internally, `AtomicRateLimiter` tries to keep buckets locked for reading (opposed to locked completely) so multiple threads can use different buckets. However, when reducing from one bucket (per key), only one thread can reduce tokens at the same time to maintain consistent state. 

### Async thread-safe rate_limiter

Async implementation of `AtomicRateLimiter` is behind `async` crate feature and it's included by default. Difference to thread-safe rate_limiter is use of async locks so thread consuming rate-limiter can be released until able to lock desired resource. Another important change is RWLock used gives priority to write operation, which in this case is when you're trying to reduce from bucket that doesn't exist yet. This prevents writer starvation and it helps a lot with performance. 

```rust
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
```
