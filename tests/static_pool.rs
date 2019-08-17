//! Test for using a pool in a `lazy_static` variable.

use lazy_static::lazy_static;
use swimmer::Pool;

lazy_static! {
    static ref POOL: Pool<String> = {
        swimmer::builder()
            .with_starting_size(512)
            .with_supplier(|| String::with_capacity(128))
            .build()
    };
}

#[test]
fn lazy_static_pool() {
    let value = POOL.get();

    assert_eq!(*value, "");
    assert_eq!(value.capacity(), 128);

    use_pool();

    drop(value);

    assert_eq!(POOL.size(), 1025);
}

fn use_pool() {
    let mut values = vec![];

    for _ in 0..1024 {
        values.push(POOL.get());
    }

    for val in values.iter_mut() {
        val.push_str("bla");
    }

    for val in values.iter() {
        assert_eq!(**val, "bla");
    }

    drop(values);

    for _ in 0..1024 {
        assert_eq!(*POOL.get(), "");
    }
}
