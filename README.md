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

```
use rate_limiter;
let mut rate_limiter = rate_limiter::RateLimiter::new(5, 1, 1);
let (success, available_tokens) = rate_limiter.reduce(String::from("send-message-user-id-1"), 1);
if success {
    // send_message();
}
```

We created rate-limiter which will hold unlimited number of buckets per given key. Each bucket will have max capacity of 5 tokens and refill of 1 token every 1 second. Sending one message requires 1 token.
In practice this means that user can send 5 messages at once and then 1 every 1 second because that's the refill speed. By changing those 3 parameters, developer can tweak bursting and refilling for desired output. If `success` was true, message sending can be allowed, otherwise, rate-limiting should be applied.

