use std::time::Duration;

use super::*;

fn strings(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

#[test]
fn bus_coalesces_and_maps_tables() {
    let bus = Bus::new();
    bus.set_window(Duration::from_millis(10));
    let mut subscription = bus.subscribe(0);
    assert_eq!(subscription.current, 0);
    assert!(!subscription.resync);
    assert_eq!(bus.subscribers(), 1);

    bus.record(
        &strings(&["work_cards", "change_ops"]),
        &strings(&["proj-a"]),
        &strings(&["cli"]),
    );
    bus.record(
        &strings(&["projects", "store_health"]),
        &strings(&["proj-b"]),
        &strings(&["desktop"]),
    );
    bus.record(&strings(&["store_health"]), &[], &strings(&["core"]));

    let change = subscription
        .recv_timeout(Duration::from_secs(1))
        .expect("no change event");
    assert_eq!(change.seq, 1);
    assert_eq!(change.kinds, strings(&[KIND_PROJECTS, KIND_WORK]));
    assert_eq!(change.project_ids, strings(&["proj-a", "proj-b"]));
    assert_eq!(
        change.origins,
        strings(&["cli", "desktop"]),
        "unmapped-only transactions must not add origins"
    );
    assert!(!change.resync);

    subscription.cancel();
    assert_eq!(bus.subscribers(), 0);
}

#[test]
fn bus_subscribe_backlog_and_resync() {
    let bus = Bus::new();
    bus.set_window(Duration::from_millis(10));
    for _ in 0..3 {
        bus.record(&strings(&["work_cards"]), &[], &[]);
        bus.flush();
    }
    let mut subscription = bus.subscribe(1);
    assert_eq!(subscription.current, 3);
    assert!(!subscription.resync);
    assert_eq!(subscription.backlog.len(), 2);
    assert_eq!(subscription.backlog[0].seq, 2);
    subscription.cancel();

    // A seq from a previous core is ahead of this one.
    let ahead = bus.subscribe(99);
    assert!(ahead.resync);
    assert_eq!(ahead.current, 3);
}

#[test]
fn slow_subscribers_get_a_resync() {
    let bus = Bus::new();
    let subscription = bus.subscribe(0);
    for _ in 0..SUBSCRIBER_BUF + 2 {
        bus.record(&strings(&["work_cards"]), &[], &[]);
        bus.flush();
    }
    // The queue overflowed on event 65: it was drained and replaced by a resync.
    let first = subscription.try_recv().expect("a change");
    assert_eq!(first.seq, (SUBSCRIBER_BUF + 1) as i64);
    assert!(first.resync, "the overflow is a resync event");
    assert!(first.kinds.is_empty());
    let second = subscription.try_recv().expect("the next change");
    assert_eq!(second.seq, (SUBSCRIBER_BUF + 2) as i64);
    assert!(!second.resync);
    assert_eq!(second.kinds, strings(&[KIND_WORK]));
    assert!(subscription.try_recv().is_none());
}
