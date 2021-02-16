#[test]
/// Test if we can build a pool from an existing vec
fn existing_pool_full() {
    let values = vec![
        "value_1".to_string(),
        "value_2".to_string(),
        "value_3".to_string(),
    ];

    let pool = swimmer::builder().build_with(values);

    // pops off the last value in the vec
    let value = pool.get();

    assert_eq!(*value, "value_3".to_string());
}

#[test]
/// Test if we can build a pool from an existing vec
fn existing_pool_partial() {
    let values = vec![
        "value_1".to_string(),
        "value_2".to_string(),
        "value_3".to_string(),
    ];

    let pool = swimmer::builder().with_starting_size(5).build_with(values);

    // pops off the last value in the vec
    let value_5 = pool.get();

    assert_eq!(*value_5, "".to_string());

    let value_4 = pool.get();

    assert_eq!(*value_4, "".to_string());

    let value_3 = pool.get();

    assert_eq!(*value_3, "value_3".to_string());
}
