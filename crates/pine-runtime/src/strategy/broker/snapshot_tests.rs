use super::*;

#[test]
fn broker_snapshot_restore_rolls_back_fill_and_pending_state() {
    let mut broker = BrokerState::new(100_000.0);
    broker.place_pending_limit_long_order("L".to_owned(), 2.0, 90.0, 0);
    assert_eq!(broker.pending_entry_count(), 1);
    assert_eq!(broker.position_size, 0.0);
    let cash_before = broker.cash;

    let snapshot = broker.snapshot();
    broker.fill_pending_limit_long_entries(1, 10, 89.0);
    assert_eq!(broker.pending_entry_count(), 0);
    assert_eq!(broker.position_size, 2.0);
    assert!(broker.cash < cash_before);

    broker.restore(snapshot);
    assert_eq!(broker.pending_entry_count(), 1);
    assert_eq!(broker.position_size, 0.0);
    assert_eq!(broker.cash, cash_before);
    assert!(broker.result().orders.is_empty());
}

#[test]
fn broker_snapshot_restore_keeps_open_position_and_pending_exit() {
    let mut broker = BrokerState::new(100_000.0);
    assert!(broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0));
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 90.0, 0);
    let snapshot = broker.snapshot();
    let size = broker.position_size;
    let cash = broker.cash;

    broker.close_long("L".to_owned(), 1, 20, 110.0);
    assert_eq!(broker.position_size, 0.0);

    broker.restore(snapshot);
    assert_eq!(broker.position_size, size);
    assert_eq!(broker.cash, cash);
    assert_eq!(broker.pending_exit_count(), 1);
}
