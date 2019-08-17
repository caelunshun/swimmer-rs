# swimmer
Thread-safe object pools for Rust.

```rust
use swimmer::Pool;

let pool: Pool<String> = Pool::with_size(10);
assert_eq!(pool.size(), 10);

let value = pool.get()
assert_eq!(pool.size(), 9);
assert_eq!(*value, "");

drop(value);
// Value is returned to pool
assert_eq!(pool.size(), 10);
```

See the [documentation](https://docs.rs/swimmer/) for more.