use polterware_core::domains::logs;

#[test]
fn test_stream_bounds() {
    let buffer = logs::Buffer::new(3);
    for _ in 0..10 {
        buffer.append("line");
    }
    assert!(
        buffer.len() == 3 && buffer.dropped() > 0,
        "bounded drop-oldest"
    );

    let queue = logs::EventQueue::new(2);
    assert!(queue.try_send("a".to_string()));
    assert!(queue.try_send("b".to_string()));
    assert!(!queue.try_send("c".to_string()));
    assert!(
        !queue.try_send("c".to_string()),
        "resource_exhausted expected"
    );

    let long = "x".repeat(logs::DEFAULT_MAX_LINE_BYTES + 5);
    assert!(buffer.append(&long), "expect truncate");
}
