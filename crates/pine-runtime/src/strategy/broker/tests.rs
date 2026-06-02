use super::exits::{
    ExitQuantityRequest, PendingExitQuantity, PendingExitReservationFamily, PendingExitTouch,
    PendingTrailingActivation, PendingTrailingExit, PendingTrailingSpec, PendingTrailingState,
    PendingTrailingUpdate, TrailPointsExitSpec, TrailPriceExitSpec,
};
use super::*;

fn pending_exit_count(broker: &BrokerState) -> usize {
    broker.pending_exit_count()
}

fn pending_exit_ids(broker: &BrokerState) -> Vec<&str> {
    broker
        .pending_exits_in_placement_order()
        .map(|pending_exit| pending_exit.id.as_str())
        .collect()
}

fn broker_with_long_entry() -> BrokerState {
    let mut broker = BrokerState::new(100_000.0);
    broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0);
    broker
}

fn trailing_price_trigger(activation_price: f64, offset_price_distance: f64) -> PendingExitTrigger {
    PendingExitTrigger::Trailing(PendingTrailingExit {
        spec: PendingTrailingSpec {
            activation: PendingTrailingActivation::Price(activation_price),
            offset_price_distance,
        },
        state: PendingTrailingState::Inactive,
    })
}

fn trailing_points_trigger(
    ticks: f64,
    activation_price: f64,
    offset_price_distance: f64,
) -> PendingExitTrigger {
    PendingExitTrigger::Trailing(PendingTrailingExit {
        spec: PendingTrailingSpec {
            activation: PendingTrailingActivation::Points {
                ticks,
                price: activation_price,
            },
            offset_price_distance,
        },
        state: PendingTrailingState::Inactive,
    })
}

fn trailing_price_exit(activation_price: f64, offset_price_distance: f64) -> PendingTrailingExit {
    PendingTrailingExit {
        spec: PendingTrailingSpec {
            activation: PendingTrailingActivation::Price(activation_price),
            offset_price_distance,
        },
        state: PendingTrailingState::Inactive,
    }
}

fn assert_active_trailing_stop(broker: &BrokerState, expected_stop_price: f64) {
    let pending_exit = broker.pending_exit().expect("pending exit");
    let PendingExitTrigger::Trailing(trailing) = &pending_exit.trigger else {
        panic!("expected trailing pending exit");
    };
    assert_eq!(
        trailing.state,
        PendingTrailingState::Active {
            stop_price: expected_stop_price,
        }
    );
}

fn assert_active_trailing_stop_by_id(broker: &BrokerState, id: &str, expected_stop_price: f64) {
    let pending_exit = broker
        .pending_exit_by_identity(id, "L")
        .expect("pending exit");
    let PendingExitTrigger::Trailing(trailing) = &pending_exit.trigger else {
        panic!("expected trailing pending exit");
    };
    assert_eq!(
        trailing.state,
        PendingTrailingState::Active {
            stop_price: expected_stop_price,
        }
    );
}

#[test]
fn exit_trigger_helpers_classify_single_trigger_reservation_family() {
    assert_eq!(
        PendingExitTrigger::Stop(95.0).reservation_family(),
        PendingExitReservationFamily::SingleTrigger
    );
    assert_eq!(
        PendingExitTrigger::Stop(95.0).single_trigger_side(),
        Some(PendingExitSide::Stop)
    );
    assert_eq!(
        PendingExitTrigger::Limit(105.0).reservation_family(),
        PendingExitReservationFamily::SingleTrigger
    );
    assert_eq!(
        PendingExitTrigger::Limit(105.0).single_trigger_side(),
        Some(PendingExitSide::Limit)
    );
}

#[test]
fn exit_trigger_helpers_classify_bracket_and_trailing_reservation_families() {
    let bracket = PendingExitTrigger::Bracket {
        downside: 95.0,
        upside: 105.0,
    };

    assert_eq!(
        bracket.reservation_family(),
        PendingExitReservationFamily::Bracket
    );
    assert_eq!(bracket.single_trigger_side(), None);
    assert_eq!(
        trailing_price_trigger(105.0, 2.0).reservation_family(),
        PendingExitReservationFamily::Trailing
    );
    assert!(trailing_price_trigger(105.0, 2.0).is_trailing_reservation_candidate());
    assert_eq!(
        trailing_price_trigger(105.0, 2.0).single_trigger_side(),
        None
    );
}

#[test]
fn exit_trigger_helpers_select_single_trigger_touched_candidates() {
    let stop = PendingExitTrigger::Stop(95.0);
    let limit = PendingExitTrigger::Limit(105.0);

    assert_eq!(
        stop.touched_candidate(104.0, 94.0),
        Some(PendingExitTouch {
            exit_price: 95.0,
            side: PendingExitSide::Stop,
        })
    );
    assert_eq!(stop.touched_candidate(104.0, 96.0), None);
    assert_eq!(
        limit.touched_candidate(106.0, 96.0),
        Some(PendingExitTouch {
            exit_price: 105.0,
            side: PendingExitSide::Limit,
        })
    );
    assert_eq!(limit.touched_candidate(104.0, 96.0), None);
}

#[test]
fn exit_trigger_helpers_select_bracket_touched_candidates() {
    let bracket = PendingExitTrigger::Bracket {
        downside: 95.0,
        upside: 105.0,
    };

    assert_eq!(
        bracket.touched_candidate(104.0, 94.0),
        Some(PendingExitTouch {
            exit_price: 95.0,
            side: PendingExitSide::Stop,
        })
    );
    assert_eq!(
        bracket.touched_candidate(106.0, 96.0),
        Some(PendingExitTouch {
            exit_price: 105.0,
            side: PendingExitSide::Limit,
        })
    );
    assert_eq!(
        bracket.touched_candidate(106.0, 94.0),
        Some(PendingExitTouch {
            exit_price: 95.0,
            side: PendingExitSide::Stop,
        })
    );
    assert_eq!(bracket.touched_candidate(104.0, 96.0), None);
}

#[test]
fn exit_trigger_helpers_exclude_trailing_from_fixed_touch_selection() {
    let trailing = trailing_price_trigger(105.0, 2.0);

    assert_eq!(trailing.touched_candidate(106.0, 94.0), None);
}

#[test]
fn trailing_update_helper_activates_without_fill_candidate() {
    let trailing = trailing_price_exit(105.0, 2.0);

    assert_eq!(
        trailing.evaluate_update(110.0, 100.0),
        PendingTrailingUpdate::Persist(PendingTrailingExit {
            spec: PendingTrailingSpec {
                activation: PendingTrailingActivation::Price(105.0),
                offset_price_distance: 2.0,
            },
            state: PendingTrailingState::Active { stop_price: 108.0 },
        })
    );
    assert_eq!(
        trailing.evaluate_update(104.0, 100.0),
        PendingTrailingUpdate::NoChange
    );
}

#[test]
fn trailing_update_helper_selects_active_stop_candidate() {
    let mut trailing = trailing_price_exit(105.0, 2.0);
    trailing.state = PendingTrailingState::Active { stop_price: 108.0 };

    assert_eq!(
        trailing.evaluate_update(115.0, 107.0),
        PendingTrailingUpdate::Candidate(PendingExitTouch {
            exit_price: 108.0,
            side: PendingExitSide::Stop,
        })
    );
}

#[test]
fn trailing_update_helper_ratchets_upward_only() {
    let mut trailing = trailing_price_exit(105.0, 2.0);
    trailing.state = PendingTrailingState::Active { stop_price: 108.0 };

    assert_eq!(
        trailing.evaluate_update(113.0, 109.0),
        PendingTrailingUpdate::Persist(PendingTrailingExit {
            spec: PendingTrailingSpec {
                activation: PendingTrailingActivation::Price(105.0),
                offset_price_distance: 2.0,
            },
            state: PendingTrailingState::Active { stop_price: 111.0 },
        })
    );
    assert_eq!(
        trailing.evaluate_update(109.0, 108.5),
        PendingTrailingUpdate::NoChange
    );
}

#[test]
fn trade_counts_start_flat() {
    let broker = BrokerState::new(100_000.0);

    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 0);
}

#[test]
fn trade_counts_track_long_entry_and_no_pyramiding_noop() {
    let mut broker = BrokerState::new(100_000.0);

    broker.entry_long("L".to_owned(), 0, 10, 100.0, 2.0);
    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 1);

    broker.entry_long("L2".to_owned(), 1, 20, 105.0, 1.0);
    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 1);
}

#[test]
fn trade_counts_track_matching_close() {
    let mut broker = broker_with_long_entry();

    broker.close_long("L".to_owned(), 1, 20, 110.0);

    assert_eq!(broker.closed_trade_count(), 1);
    assert_eq!(broker.open_trade_count(), 0);
}

#[test]
fn entry_rejects_non_finite_fill_price() {
    let mut broker = BrokerState::new(100_000.0);

    broker.entry_long("L".to_owned(), 0, 10, f64::NAN, 2.0);

    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 0);
    assert!(broker.orders.is_empty());
    assert!(broker.position.is_empty());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_PRICE");
}

#[test]
fn close_rejects_non_finite_fill_price_without_closing_position() {
    let mut broker = broker_with_long_entry();

    broker.close_long("L".to_owned(), 1, 20, f64::INFINITY);

    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 1);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_PRICE");
}

#[test]
fn trade_counts_track_filled_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(broker.closed_trade_count(), 1);
    assert_eq!(broker.open_trade_count(), 0);
}

#[test]
fn full_quantity_pending_exit_still_closes_whole_position() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(broker.orders[1].qty, 2.0);
    assert_eq!(broker.trades[0].qty, 2.0);
    assert_eq!(broker.position_size, 0.0);
}

#[test]
fn trade_counts_ignore_mismatched_close_and_exit() {
    let mut broker = broker_with_long_entry();

    broker.close_long("OTHER".to_owned(), 1, 20, 110.0);
    broker.place_exit_stop("XL".to_owned(), "OTHER".to_owned(), 95.0, 1);

    assert_eq!(broker.closed_trade_count(), 0);
    assert_eq!(broker.open_trade_count(), 1);
}

#[test]
fn place_exit_while_flat_records_diagnostic_without_pending_state() {
    let mut broker = BrokerState::new(100_000.0);

    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn place_exit_while_long_records_pending_stop() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XL", "L").is_some());
    assert!(broker.pending_exit_by_identity("XL", "OTHER").is_none());
    assert_eq!(pending_exit_ids(&broker), vec!["XL"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn place_exit_replaces_existing_pending_stop() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_stop("XL2".to_owned(), "L".to_owned(), 90.0, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XL", "L").is_none());
    assert!(broker.pending_exit_by_identity("XL2", "L").is_some());
    assert_eq!(pending_exit_ids(&broker), vec!["XL2"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL2".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(90.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
}

#[test]
fn omitted_quantity_single_trigger_with_new_identity_replaces_instead_of_appending() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XS", "L").is_none());
    assert_eq!(pending_exit_ids(&broker), vec!["XL"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(110.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn reservation_helpers_track_reserved_and_available_quantity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 1.25, 0);

    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        1.25
    );
    assert_eq!(
        broker
            .pending_exits
            .available_unreserved_quantity(broker.position_size, "L", None),
        0.75
    );
    assert_eq!(
        broker.pending_exits.available_unreserved_quantity(
            broker.position_size,
            "L",
            Some(("XL", "L"))
        ),
        2.0
    );
}

#[test]
fn reservation_resolver_rejects_zero_available_quantity() {
    let mut broker = broker_with_long_entry();

    let resolved =
        broker.resolve_exit_quantity_request_for_available(ExitQuantityRequest::Full, 0.0);

    assert_eq!(resolved, None);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn reservation_resolver_reserves_full_available_quantity() {
    let mut broker = broker_with_long_entry();

    let resolved =
        broker.resolve_exit_quantity_request_for_available(ExitQuantityRequest::Full, 2.0);

    assert_eq!(resolved, Some((PendingExitQuantity::Full, 2.0)));
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn reservation_resolver_clamps_fixed_quantity_to_available_quantity() {
    let mut broker = broker_with_long_entry();

    let resolved =
        broker.resolve_exit_quantity_request_for_available(ExitQuantityRequest::Fixed(2.0), 0.75);

    assert_eq!(resolved, Some((PendingExitQuantity::Fixed(2.0), 0.75)));
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn reservation_resolver_clamps_percent_quantity_to_available_quantity() {
    let mut broker = broker_with_long_entry();

    let resolved = broker
        .resolve_exit_quantity_request_for_available(ExitQuantityRequest::Percent(50.0), 0.75);

    assert_eq!(resolved, Some((PendingExitQuantity::Fixed(1.0), 0.75)));
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn reservation_resolver_clamps_over_100_percent_to_available_quantity() {
    let mut broker = broker_with_long_entry();

    let resolved = broker
        .resolve_exit_quantity_request_for_available(ExitQuantityRequest::Percent(150.0), 2.0);

    assert_eq!(resolved, Some((PendingExitQuantity::Fixed(3.0), 2.0)));
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn multiple_fixed_stop_exits_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 95.0, 0.75, 0);
    broker.place_exit_stop_qty("XS2".to_owned(), "L".to_owned(), 94.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XS1", "XS2"]);

    broker.evaluate_pending_exits(1, 20, 100.0, 93.0);

    assert_eq!(broker.orders[1].id, "XS1");
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.orders[2].id, "XS2");
    assert_eq!(broker.orders[2].qty, 0.5);
    assert_eq!(broker.orders[2].price, 94.0);
    assert_eq!(broker.trades.len(), 2);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
}

#[test]
fn multiple_fixed_limit_exits_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL1".to_owned(), "L".to_owned(), 105.0, 0.75, 0);
    broker.place_exit_limit_qty("XL2".to_owned(), "L".to_owned(), 106.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XL1", "XL2"]);

    broker.evaluate_pending_exits(1, 20, 107.0, 100.0);

    assert_eq!(broker.orders[1].id, "XL1");
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.orders[1].price, 105.0);
    assert_eq!(broker.orders[2].id, "XL2");
    assert_eq!(broker.orders[2].qty, 0.5);
    assert_eq!(broker.orders[2].price, 106.0);
    assert_eq!(broker.trades.len(), 2);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
}

#[test]
fn replacing_one_fixed_exit_releases_only_that_reservation() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_stop_qty("XS2".to_owned(), "L".to_owned(), 94.0, 0.75, 0);

    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 93.0, 1.25, 1);

    assert_eq!(pending_exit_ids(&broker), vec!["XS1", "XS2"]);
    let replaced = broker.pending_exit_by_identity("XS1", "L").unwrap();
    assert_eq!(replaced.trigger, PendingExitTrigger::Stop(93.0));
    assert_eq!(replaced.quantity, PendingExitQuantity::Fixed(1.25));
    assert_eq!(replaced.reserved_quantity, 1.25);
    let preserved = broker.pending_exit_by_identity("XS2", "L").unwrap();
    assert_eq!(preserved.reserved_quantity, 0.75);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
}

#[test]
fn new_fixed_exit_clamps_to_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL1".to_owned(), "L".to_owned(), 105.0, 1.5, 0);
    broker.place_exit_limit_qty("XL2".to_owned(), "L".to_owned(), 106.0, 1.0, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    let clamped = broker.pending_exit_by_identity("XL2", "L").unwrap();
    assert_eq!(clamped.quantity, PendingExitQuantity::Fixed(1.0));
    assert_eq!(clamped.reserved_quantity, 0.5);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
}

#[test]
fn fixed_exit_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 95.0, 1.0, 0);
    broker.place_exit_stop_qty("XS2".to_owned(), "L".to_owned(), 94.0, 1.0, 0);

    broker.place_exit_stop_qty("XS3".to_owned(), "L".to_owned(), 93.0, 0.5, 0);

    assert_eq!(pending_exit_ids(&broker), vec!["XS1", "XS2"]);
    assert!(broker.pending_exit_by_identity("XS3", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn fixed_exit_after_percent_exit_shares_reservation_pool() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XP".to_owned(), "L".to_owned(), 95.0, 50.0, 0);

    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 94.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XP", "XS"]);
    assert_eq!(
        broker.pending_exit_by_identity("XP", "L").unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XP", "L")
            .unwrap()
            .reserved_quantity,
        1.0
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XS", "L")
            .unwrap()
            .reserved_quantity,
        0.5
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn two_percent_stop_exits_reserve_expected_absolute_quantities() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XP1".to_owned(), "L".to_owned(), 95.0, 25.0, 0);
    broker.place_exit_stop_qty_percent("XP2".to_owned(), "L".to_owned(), 94.0, 50.0, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XP1", "XP2"]);
    let first = broker.pending_exit_by_identity("XP1", "L").unwrap();
    assert_eq!(first.quantity, PendingExitQuantity::Fixed(0.5));
    assert_eq!(first.reserved_quantity, 0.5);
    let second = broker.pending_exit_by_identity("XP2", "L").unwrap();
    assert_eq!(second.quantity, PendingExitQuantity::Fixed(1.0));
    assert_eq!(second.reserved_quantity, 1.0);

    broker.evaluate_pending_exits(1, 20, 100.0, 93.0);

    assert_eq!(broker.orders[1].id, "XP1");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[2].id, "XP2");
    assert_eq!(broker.orders[2].qty, 1.0);
    assert_eq!(broker.position_size, 0.5);
}

#[test]
fn percent_replacement_releases_old_reservation_first() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XP1".to_owned(), "L".to_owned(), 95.0, 25.0, 0);
    broker.place_exit_stop_qty_percent("XP2".to_owned(), "L".to_owned(), 94.0, 25.0, 0);

    broker.place_exit_stop_qty_percent("XP1".to_owned(), "L".to_owned(), 93.0, 75.0, 1);

    assert_eq!(pending_exit_ids(&broker), vec!["XP1", "XP2"]);
    let replaced = broker.pending_exit_by_identity("XP1", "L").unwrap();
    assert_eq!(replaced.trigger, PendingExitTrigger::Stop(93.0));
    assert_eq!(replaced.quantity, PendingExitQuantity::Fixed(1.5));
    assert_eq!(replaced.reserved_quantity, 1.5);
    let preserved = broker.pending_exit_by_identity("XP2", "L").unwrap();
    assert_eq!(preserved.reserved_quantity, 0.5);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
}

#[test]
fn over_100_percent_exit_reserves_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.75, 0);
    broker.place_exit_stop_qty_percent("XP".to_owned(), "L".to_owned(), 94.0, 150.0, 0);

    let percent = broker.pending_exit_by_identity("XP", "L").unwrap();
    assert_eq!(percent.quantity, PendingExitQuantity::Fixed(3.0));
    assert_eq!(percent.reserved_quantity, 1.25);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn percent_exit_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XP1".to_owned(), "L".to_owned(), 95.0, 50.0, 0);
    broker.place_exit_stop_qty_percent("XP2".to_owned(), "L".to_owned(), 94.0, 50.0, 0);

    broker.place_exit_stop_qty_percent("XP3".to_owned(), "L".to_owned(), 93.0, 25.0, 0);

    assert_eq!(pending_exit_ids(&broker), vec!["XP1", "XP2"]);
    assert!(broker.pending_exit_by_identity("XP3", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn mixed_side_both_touched_processes_downside_only() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 94.0);

    assert_eq!(broker.orders[1].id, "XS");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.position_size, 1.5);
    assert_eq!(pending_exit_ids(&broker), vec!["XL"]);
}

#[test]
fn mixed_side_multiple_downside_candidates_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 0.25, 0);
    broker.place_exit_stop_qty("XS2".to_owned(), "L".to_owned(), 94.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 93.0);

    assert_eq!(broker.orders[1].id, "XS1");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[2].id, "XS2");
    assert_eq!(broker.orders[2].qty, 0.75);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_ids(&broker), vec!["XL"]);
}

#[test]
fn mixed_side_partial_then_full_fill_updates_counts_position_and_equity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 1.5, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 94.0);

    assert_eq!(broker.position_size, 1.5);
    assert_eq!(broker.position.last().unwrap().size, 1.5);
    assert_eq!(broker.position.last().unwrap().avg_price, Some(100.0));
    assert_eq!(broker.closed_trade_count(), 1);
    assert_eq!(broker.open_trade_count(), 1);
    assert_eq!(broker.realized_profit(), -2.5);
    assert_eq!(broker.equity_value(106.0), 100_006.5);
    broker.record_equity(1, 106.0);
    let partial_equity = broker.equity.last().unwrap();
    assert_eq!(partial_equity.cash, 99_847.5);
    assert_eq!(partial_equity.market_value, 159.0);
    assert_eq!(partial_equity.equity, 100_006.5);
    assert_eq!(partial_equity.net_profit, 6.5);
    assert_eq!(pending_exit_ids(&broker), vec!["XL"]);

    broker.evaluate_pending_exits(2, 30, 111.0, 100.0);

    assert_eq!(broker.position_size, 0.0);
    assert_eq!(broker.position.last().unwrap().size, 0.0);
    assert_eq!(broker.position.last().unwrap().avg_price, None);
    assert_eq!(broker.closed_trade_count(), 2);
    assert_eq!(broker.open_trade_count(), 0);
    assert_eq!(broker.realized_profit(), 12.5);
    assert_eq!(broker.equity_value(106.0), 100_012.5);
    broker.record_equity(2, 106.0);
    let final_equity = broker.equity.last().unwrap();
    assert_eq!(final_equity.cash, 100_012.5);
    assert_eq!(final_equity.market_value, 0.0);
    assert_eq!(final_equity.equity, 100_012.5);
    assert_eq!(final_equity.net_profit, 12.5);
    assert_eq!(pending_exit_count(&broker), 0);
}

#[test]
fn full_close_cancels_remaining_multiple_pending_exits() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS1".to_owned(), "L".to_owned(), 95.0, 0.75, 0);
    broker.place_exit_stop_qty("XS2".to_owned(), "L".to_owned(), 94.0, 0.75, 0);

    broker.close_long("L".to_owned(), 1, 20, 110.0);

    assert_eq!(broker.position_size, 0.0);
    assert_eq!(pending_exit_count(&broker), 0);
}

#[test]
fn omitted_quantity_exit_replaces_explicit_reservation_pool() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 94.0, 110.0, 0.75, 0);
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    broker.place_exit_limit("XFULL".to_owned(), "L".to_owned(), 111.0, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XS", "L").is_none());
    assert!(broker.pending_exit_by_identity("XB", "L").is_none());
    assert!(broker.pending_exit_by_identity("XT", "L").is_none());
    assert_eq!(pending_exit_ids(&broker), vec!["XFULL"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XFULL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(111.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn explicit_reservation_after_omitted_quantity_replaces_full_then_appends_supported_reservations() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XFULL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 94.0, 0.5, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XFULL", "L").is_none());
    let first_explicit = broker.pending_exit_by_identity("XS", "L").unwrap();
    assert_eq!(first_explicit.quantity, PendingExitQuantity::Fixed(0.5));
    assert_eq!(first_explicit.reserved_quantity, 0.5);
    assert!(first_explicit.multiple_reservation);

    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 93.0, 110.0, 0.75, 1);
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.25,
        1,
    );

    assert_eq!(pending_exit_count(&broker), 3);
    assert_eq!(pending_exit_ids(&broker), vec!["XS", "XB", "XT"]);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        1.5
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn fixed_quantity_is_stored_on_supported_pending_exit_families() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 1.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 1.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_profit_ticks_qty("XP".to_owned(), "L".to_owned(), 10.0, 0.5, 1.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_loss_ticks_qty("XL".to_owned(), "L".to_owned(), 5.0, 0.5, 1.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 1.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_points_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPointsExitSpec {
            activation_ticks: 10.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );
}

#[test]
fn percent_quantity_resolves_on_supported_pending_exit_families() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XS".to_owned(), "L".to_owned(), 95.0, 50.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty_percent("XL".to_owned(), "L".to_owned(), 110.0, 100.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(2.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_profit_ticks_qty_percent("XP".to_owned(), "L".to_owned(), 10.0, 0.5, 50.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_loss_ticks_qty_percent("XL".to_owned(), "L".to_owned(), 5.0, 0.5, 50.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty_percent("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 50.0, 0);
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty_percent(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        50.0,
        0,
    );
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );

    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_points_qty_percent(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPointsExitSpec {
            activation_ticks: 10.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        50.0,
        0,
    );
    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(1.0)
    );
}

#[test]
fn percent_quantity_larger_than_position_closes_full_position() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty_percent("XL".to_owned(), "L".to_owned(), 110.0, 150.0, 0);

    assert_eq!(
        broker.pending_exit().unwrap().quantity,
        PendingExitQuantity::Fixed(3.0)
    );

    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(broker.orders[1].qty, 2.0);
    assert_eq!(broker.trades[0].qty, 2.0);
    assert_eq!(broker.position_size, 0.0);
    assert_eq!(broker.entry_id, None);
}

#[test]
fn unchanged_repeated_quantity_keeps_original_eligibility_bar() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 1.0, 0);
    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 1.0, 1);

    assert_eq!(broker.pending_exit().unwrap().last_update_bar_index, 0);
}

#[test]
fn changed_repeated_quantity_replaces_pending_exit() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 1.0, 0);
    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 0.5, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Fixed(0.5),
            reserved_quantity: 0.5,
            multiple_reservation: true,
            last_update_bar_index: 1,
        })
    );
}

#[test]
fn invalid_fixed_quantity_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 1.0, 0);
    let original_pending_exit = broker.pending_exit().cloned();

    broker.place_exit_stop_qty("BAD".to_owned(), "L".to_owned(), 94.0, 0.0, 1);
    broker.place_exit_limit_qty("BAD2".to_owned(), "L".to_owned(), 110.0, f64::NAN, 2);

    assert_eq!(broker.pending_exit().cloned(), original_pending_exit);
    assert_eq!(broker.diagnostics.len(), 2);
    assert!(
        broker
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "E_STRATEGY_EXIT_QTY")
    );
}

#[test]
fn invalid_percent_quantity_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty_percent("XL".to_owned(), "L".to_owned(), 95.0, 50.0, 0);
    let original_pending_exit = broker.pending_exit().cloned();

    broker.place_exit_stop_qty_percent("BAD".to_owned(), "L".to_owned(), 94.0, 0.0, 1);
    broker.place_exit_limit_qty_percent("BAD2".to_owned(), "L".to_owned(), 110.0, f64::NAN, 2);

    assert_eq!(broker.pending_exit().cloned(), original_pending_exit);
    assert_eq!(broker.diagnostics.len(), 2);
    assert!(
        broker
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "E_STRATEGY_EXIT_QTY_PERCENT")
    );
}

#[test]
fn percent_quantity_while_flat_records_entry_diagnostic_before_percent_resolution() {
    let mut broker = BrokerState::new(100_000.0);

    broker.place_exit_stop_qty_percent("XL".to_owned(), "L".to_owned(), 95.0, f64::NAN, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn percent_quantity_mismatched_entry_records_entry_diagnostic_before_percent_resolution() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("KEEP".to_owned(), "L".to_owned(), 95.0, 0);
    let original_pending_exit = broker.pending_exit().cloned();

    broker.place_exit_stop_qty_percent("BAD".to_owned(), "OTHER".to_owned(), 94.0, f64::NAN, 1);

    assert_eq!(broker.pending_exit().cloned(), original_pending_exit);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn close_long_cancels_matching_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.close_long("L".to_owned(), 1, 20, 110.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
}

#[test]
fn mismatched_entry_id_records_diagnostic_without_pending_state() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XL".to_owned(), "OTHER".to_owned(), 95.0, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn repeated_entry_noop_leaves_pending_exit_untouched() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.entry_long("L2".to_owned(), 1, 20, 105.0, 1.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
}

#[test]
fn pending_stop_is_not_eligible_on_creation_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.evaluate_pending_exits(0, 10, 100.0, 90.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_stop_fills_on_later_crossing_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);

    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].id, "XL");
    assert_eq!(broker.orders[1].direction, "strategy.exit");
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].id, "L");
    assert_eq!(broker.trades[0].exit_bar_index, 1);
    assert_eq!(broker.trades[0].exit_price, 95.0);
    assert_eq!(broker.trades[0].profit, -10.0);
    assert_eq!(broker.position_size, 0.0);
    assert_eq!(broker.position.last().unwrap().avg_price, None);
}

#[test]
fn unchanged_repeated_exit_keeps_original_eligibility_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 1);

    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 95.0);
}

#[test]
fn changed_repeated_exit_replaces_price_and_delays_eligibility() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_stop("XL".to_owned(), "L".to_owned(), 90.0, 1);

    broker.evaluate_pending_exits(1, 20, 100.0, 89.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.trades.is_empty());

    broker.evaluate_pending_exits(2, 30, 100.0, 89.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 90.0);
}

#[test]
fn pending_limit_fills_on_later_crossing_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].direction, "strategy.exit");
    assert_eq!(broker.orders[1].price, 110.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 110.0);
    assert_eq!(broker.trades[0].profit, 20.0);
}

#[test]
fn pending_stop_partial_quantity_reduces_position() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XL".to_owned(), "L".to_owned(), 95.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.trades[0].qty, 0.75);
    assert_eq!(broker.trades[0].profit, -3.75);
    assert_eq!(broker.position_size, 1.25);
    assert_eq!(broker.avg_price, 100.0);
    assert_eq!(broker.entry_id.as_deref(), Some("L"));
    assert_eq!(broker.entry_bar_index, Some(0));
    assert_eq!(broker.entry_time, Some(10));
    assert_eq!(broker.position.last().unwrap().size, 1.25);
    assert_eq!(broker.position.last().unwrap().avg_price, Some(100.0));
    assert_eq!(broker.open_trade_count(), 1);
    assert_eq!(broker.closed_trade_count(), 1);
}

#[test]
fn pending_limit_partial_quantity_reduces_position() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 0.5, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.trades[0].qty, 0.5);
    assert_eq!(broker.trades[0].profit, 5.0);
    assert_eq!(broker.position_size, 1.5);
    assert_eq!(broker.position.last().unwrap().avg_price, Some(100.0));
}

#[test]
fn fixed_quantity_larger_than_position_closes_full_position() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 5.0, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(broker.orders[1].qty, 2.0);
    assert_eq!(broker.trades[0].qty, 2.0);
    assert_eq!(broker.position_size, 0.0);
    assert_eq!(broker.avg_price, 0.0);
    assert_eq!(broker.entry_id, None);
    assert_eq!(broker.open_trade_count(), 0);
    assert_eq!(broker.position.last().unwrap().avg_price, None);
}

#[test]
fn partial_fill_then_final_exit_closes_open_trade_count() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("X1".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.evaluate_pending_exits(1, 20, 100.0, 94.0);

    assert_eq!(broker.open_trade_count(), 1);
    assert_eq!(broker.closed_trade_count(), 1);

    broker.place_exit_limit("X2".to_owned(), "L".to_owned(), 110.0, 2);
    broker.evaluate_pending_exits(3, 40, 111.0, 100.0);

    assert_eq!(broker.open_trade_count(), 0);
    assert_eq!(broker.closed_trade_count(), 2);
    assert_eq!(broker.trades[1].qty, 1.5);
    assert_eq!(broker.trades[1].profit, 15.0);
}

#[test]
fn partial_fill_splits_realized_and_open_profit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 0.5, 0);
    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(broker.realized_profit(), 5.0);
    assert_eq!(broker.open_profit(106.0), 9.0);
    assert_eq!(broker.equity_value(106.0), 100_014.0);

    broker.record_equity(1, 106.0);
    let equity = broker.equity.last().unwrap();
    assert_eq!(equity.cash, 99_855.0);
    assert_eq!(equity.market_value, 159.0);
    assert_eq!(equity.equity, 100_014.0);
    assert_eq!(equity.net_profit, 14.0);
}

#[test]
fn profit_ticks_create_limit_from_average_entry_price() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 0.5, 0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XP".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(105.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
}

#[test]
fn loss_ticks_create_stop_from_average_entry_price() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_loss_ticks("XL".to_owned(), "L".to_owned(), 5.0, 0.5, 0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(97.5),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
}

#[test]
fn place_exit_bracket_records_pending_bracket() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 95.0,
                upside: 110.0,
            },
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn bracket_tick_helpers_resolve_prices_from_average_entry_price() {
    let mut broker = broker_with_long_entry();

    let downside = broker
        .exit_loss_price_from_ticks(5.0, 0.5)
        .expect("loss ticks should resolve");
    let upside = broker
        .exit_profit_price_from_ticks(10.0, 0.5)
        .expect("profit ticks should resolve");
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), downside, upside, 0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 97.5,
                upside: 105.0,
            },
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn place_exit_trail_price_records_pending_trailing_exit() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT".to_owned(),
            from_entry: "L".to_owned(),
            trigger: trailing_price_trigger(105.0, 2.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn place_exit_trail_points_records_entry_relative_activation() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_points("XT".to_owned(), "L".to_owned(), 10.0, 4.0, 0.5, 0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT".to_owned(),
            from_entry: "L".to_owned(),
            trigger: trailing_points_trigger(10.0, 105.0, 2.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn trailing_while_flat_records_diagnostic_without_pending_state() {
    let mut broker = BrokerState::new(100_000.0);

    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn invalid_trailing_activation_price_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), f64::NAN, 4.0, 0.5, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_PRICE");
}

#[test]
fn invalid_trailing_offset_ticks_record_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 0.0, 0.5, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
}

#[test]
fn unchanged_repeated_trailing_keeps_original_eligibility_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 1);

    assert_eq!(broker.pending_exit().unwrap().last_update_bar_index, 0);
}

#[test]
fn unchanged_repeated_trailing_preserves_active_state() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    let pending = broker.pending_exit_mut().expect("trailing pending exit");
    let PendingExitTrigger::Trailing(trailing) = &mut pending.trigger else {
        panic!("expected trailing pending exit");
    };
    trailing.state = PendingTrailingState::Active { stop_price: 103.0 };

    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Trailing(PendingTrailingExit {
                spec: PendingTrailingSpec {
                    activation: PendingTrailingActivation::Price(105.0),
                    offset_price_distance: 2.0,
                },
                state: PendingTrailingState::Active { stop_price: 103.0 },
            }),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
}

#[test]
fn changed_repeated_trailing_replaces_spec_and_delays_eligibility() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 106.0, 4.0, 0.5, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT".to_owned(),
            from_entry: "L".to_owned(),
            trigger: trailing_price_trigger(106.0, 2.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
}

#[test]
fn omitted_quantity_trailing_with_new_identity_replaces_and_resets_eligibility() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price("XT1".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.place_exit_trail_price("XT2".to_owned(), "L".to_owned(), 106.0, 6.0, 0.5, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XT1", "L").is_none());
    assert_eq!(pending_exit_ids(&broker), vec!["XT2"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT2".to_owned(),
            from_entry: "L".to_owned(),
            trigger: trailing_price_trigger(106.0, 3.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );

    broker.evaluate_pending_exits(1, 20, 110.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert_eq!(
        broker.pending_exit().unwrap().trigger,
        trailing_price_trigger(106.0, 3.0)
    );
    assert!(broker.trades.is_empty());
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn close_long_cancels_matching_trailing_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);

    broker.close_long("L".to_owned(), 1, 20, 110.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
}

#[test]
fn multiple_fixed_qty_trailing_exits_reserve_and_activate_independently() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 110.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XT1", "XT2"]);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .reserved_quantity,
        0.5
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .reserved_quantity,
        1.0
    );

    broker.evaluate_pending_exits(1, 20, 106.0, 104.0);

    assert_active_trailing_stop_by_id(&broker, "XT1", 104.0);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .trigger,
        trailing_price_trigger(110.0, 2.0)
    );
    assert!(broker.trades.is_empty());
}

#[test]
fn multiple_fixed_qty_trailing_exits_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 115.0, 107.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders[1].id, "XT1");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 108.0);
    assert_eq!(broker.orders[2].id, "XT2");
    assert_eq!(broker.orders[2].qty, 1.0);
    assert_eq!(broker.orders[2].price, 107.0);
    assert_eq!(broker.position_size, 0.5);
}

#[test]
fn fixed_qty_trailing_replacement_releases_only_that_reservation() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 107.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        1,
    );

    assert_eq!(pending_exit_ids(&broker), vec!["XT1", "XT2"]);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .reserved_quantity,
        1.0
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .reserved_quantity,
        1.0
    );
}

#[test]
fn fixed_qty_trailing_reservation_clamps_to_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );

    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .reserved_quantity,
        0.5
    );
}

#[test]
fn fixed_qty_trailing_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.0,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT3".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 107.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    assert_eq!(pending_exit_count(&broker), 2);
    assert!(broker.pending_exit_by_identity("XT3", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn full_fixed_qty_trailing_fill_clears_remaining_pending_exits() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 115.0, 107.0);

    assert_eq!(broker.position_size, 0.0);
    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 2);
}

#[test]
fn invalid_fixed_qty_trailing_replacement_preserves_existing_pending_trailing_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 107.0,
            offset_ticks: 0.0,
            mintick: 0.5,
        },
        1.0,
        1,
    );

    assert_eq!(pending_exit_ids(&broker), vec!["XT1", "XT2"]);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .trigger,
        trailing_price_trigger(105.0, 2.0)
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
}

#[test]
fn unfilled_fixed_qty_trailing_exits_ratchet_and_persist_state() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_trail_price_qty(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 113.0, 109.0);

    assert_active_trailing_stop_by_id(&broker, "XT1", 111.0);
    assert_active_trailing_stop_by_id(&broker, "XT2", 110.0);
    assert!(broker.trades.is_empty());
}

#[test]
fn two_percent_trailing_exits_reserve_expected_absolute_quantities() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        50.0,
        0,
    );

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .reserved_quantity,
        0.5
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .reserved_quantity,
        1.0
    );
}

#[test]
fn percent_and_fixed_trailing_exits_share_reservation_pool() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XF".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.75,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XP".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        75.0,
        0,
    );

    assert_eq!(
        broker
            .pending_exit_by_identity("XP", "L")
            .expect("XP")
            .reserved_quantity,
        1.25
    );
}

#[test]
fn percent_trailing_replacement_releases_old_reservation_first() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );

    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        75.0,
        1,
    );

    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .reserved_quantity,
        1.5
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XT2", "L")
            .expect("XT2")
            .reserved_quantity,
        0.5
    );
}

#[test]
fn over_100_percent_trailing_reserves_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty(
        "XF".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.75,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XP".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        150.0,
        0,
    );

    assert_eq!(
        broker
            .pending_exit_by_identity("XP", "L")
            .expect("XP")
            .reserved_quantity,
        1.25
    );
}

#[test]
fn percent_trailing_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        50.0,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        50.0,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XT3".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 8.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );

    assert_eq!(pending_exit_count(&broker), 2);
    assert!(broker.pending_exit_by_identity("XT3", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn invalid_percent_trailing_replacement_preserves_existing_pending_trailing_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );
    broker.place_exit_trail_price_qty_percent(
        "XT2".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 6.0,
            mintick: 0.5,
        },
        25.0,
        0,
    );

    broker.place_exit_trail_price_qty_percent(
        "XT1".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 106.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.0,
        1,
    );

    assert_eq!(pending_exit_ids(&broker), vec!["XT1", "XT2"]);
    assert_eq!(
        broker
            .pending_exit_by_identity("XT1", "L")
            .expect("XT1")
            .trigger,
        trailing_price_trigger(105.0, 2.0)
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY_PERCENT");
}

#[test]
fn invalid_bracket_downside_price_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), f64::NAN, 110.0, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_PRICE");
}

#[test]
fn invalid_bracket_upside_price_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, f64::INFINITY, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(110.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_PRICE");
}

#[test]
fn invalid_bracket_ticks_record_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    let price = broker.exit_profit_price_from_ticks(0.0, 0.01);

    assert_eq!(price, None);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
}

#[test]
fn invalid_bracket_mintick_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

    let price = broker.exit_loss_price_from_ticks(5.0, f64::NAN);

    assert_eq!(price, None);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(110.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_MINTICK");
}

#[test]
fn bracket_while_flat_records_diagnostic_without_pending_state() {
    let mut broker = BrokerState::new(100_000.0);

    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn bracket_with_mismatched_entry_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_bracket("XB".to_owned(), "OTHER".to_owned(), 95.0, 110.0, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn unchanged_repeated_bracket_keeps_original_eligibility_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 1);

    assert_eq!(broker.pending_exit().unwrap().last_update_bar_index, 0);
}

#[test]
fn changed_repeated_bracket_replaces_price_and_delays_eligibility() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 94.0, 110.0, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 94.0,
                upside: 110.0,
            },
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
}

#[test]
fn omitted_quantity_bracket_with_new_identity_replaces_instead_of_appending() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_bracket("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 0);
    broker.place_exit_bracket("XB2".to_owned(), "L".to_owned(), 94.0, 111.0, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XB1", "L").is_none());
    assert_eq!(pending_exit_ids(&broker), vec!["XB2"]);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB2".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 94.0,
                upside: 111.0,
            },
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn single_trigger_and_bracket_replace_each_other_and_reset_eligibility() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 95.0,
                upside: 110.0,
            },
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );

    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 111.0, 2);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(111.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 2,
        })
    );
}

#[test]
fn pending_bracket_is_not_eligible_on_creation_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    broker.evaluate_pending_exits(0, 10, 111.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_bracket_downside_only_hit_fills_at_downside_price() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    broker.evaluate_pending_exits(1, 20, 109.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].id, "XB");
    assert_eq!(broker.orders[1].direction, "strategy.exit");
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 95.0);
    assert_eq!(broker.trades[0].profit, -10.0);
    assert_eq!(broker.position_size, 0.0);
}

#[test]
fn pending_bracket_upside_only_hit_fills_at_upside_price() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 96.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].id, "XB");
    assert_eq!(broker.orders[1].direction, "strategy.exit");
    assert_eq!(broker.orders[1].price, 110.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 110.0);
    assert_eq!(broker.trades[0].profit, 20.0);
    assert_eq!(broker.position_size, 0.0);
}

#[test]
fn pending_bracket_both_hit_fills_at_downside_price_without_diagnostic() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    broker.evaluate_pending_exits(1, 20, 111.0, 94.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 95.0);
    assert_eq!(broker.trades[0].profit, -10.0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn pending_bracket_no_hit_remains_pending() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket("XB".to_owned(), "L".to_owned(), 95.0, 110.0, 0);

    broker.evaluate_pending_exits(1, 20, 109.0, 96.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn multiple_fixed_qty_brackets_reserve_and_fill_downside_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 0.75, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XB1", "XB2"]);

    broker.evaluate_pending_exits(1, 20, 109.0, 94.0);

    assert_eq!(broker.orders[1].id, "XB1");
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.orders[2].id, "XB2");
    assert_eq!(broker.orders[2].qty, 0.5);
    assert_eq!(broker.orders[2].price, 96.0);
    assert_eq!(broker.trades.len(), 2);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn multiple_fixed_qty_brackets_reserve_and_fill_upside_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 112.0, 97.0);

    assert_eq!(broker.orders[1].id, "XB1");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 110.0);
    assert_eq!(broker.orders[2].id, "XB2");
    assert_eq!(broker.orders[2].qty, 0.75);
    assert_eq!(broker.orders[2].price, 111.0);
    assert_eq!(broker.trades.len(), 2);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn fixed_qty_bracket_replacement_releases_only_that_reservation() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 1.0, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 0.5, 0);

    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 94.0, 109.0, 1.5, 1);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XB1", "XB2"]);
    let replaced = broker
        .pending_exit_by_identity("XB1", "L")
        .expect("replacement should exist");
    assert_eq!(
        replaced.trigger,
        PendingExitTrigger::Bracket {
            downside: 94.0,
            upside: 109.0,
        }
    );
    assert_eq!(replaced.quantity, PendingExitQuantity::Fixed(1.5));
    assert_eq!(replaced.reserved_quantity, 1.5);
    assert_eq!(replaced.last_update_bar_index, 1);
    assert_eq!(
        broker
            .pending_exit_by_identity("XB2", "L")
            .expect("other bracket should remain")
            .reserved_quantity,
        0.5
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn fixed_qty_bracket_reservation_clamps_to_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 1.5, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 1.0, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(
        broker
            .pending_exit_by_identity("XB2", "L")
            .expect("clamped bracket should exist")
            .reserved_quantity,
        0.5
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn fixed_qty_bracket_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 2.0, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 0.5, 0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.pending_exit_by_identity("XB2", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn full_fixed_qty_bracket_fill_clears_remaining_pending_exits() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 1.0, 0);
    broker.place_exit_bracket_qty("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 1.0, 0);

    broker.evaluate_pending_exits(1, 20, 109.0, 94.0);

    assert_eq!(broker.trades.len(), 2);
    assert_eq!(broker.position_size, 0.0);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn invalid_fixed_qty_bracket_replacement_preserves_existing_pending_bracket() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 1.0, 0);

    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), f64::NAN, 109.0, 1.5, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB1".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 95.0,
                upside: 110.0,
            },
            quantity: PendingExitQuantity::Fixed(1.0),
            reserved_quantity: 1.0,
            multiple_reservation: true,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_PRICE");
}

#[test]
fn fixed_qty_bracket_shares_reservation_pool_with_single_trigger() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 1.0, 0);

    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 94.0, 110.0, 1.0, 1);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XS", "XB"]);
    assert!(broker.pending_exit_by_identity("XS", "L").is_some());
    assert!(broker.pending_exit_by_identity("XB", "L").is_some());
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_stop_and_bracket_downside_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 96.0, 111.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 110.0, 94.0);

    assert_eq!(broker.orders[1].id, "XS");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 95.0);
    assert_eq!(broker.orders[2].id, "XB");
    assert_eq!(broker.orders[2].qty, 0.75);
    assert_eq!(broker.orders[2].price, 96.0);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_limit_and_bracket_upside_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 110.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 95.0, 111.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 112.0, 97.0);

    assert_eq!(broker.orders[1].id, "XL");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 110.0);
    assert_eq!(broker.orders[2].id, "XB");
    assert_eq!(broker.orders[2].qty, 0.75);
    assert_eq!(broker.orders[2].price, 111.0);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_downside_candidate_wins_over_bracket_upside_candidate() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 90.0, 111.0, 0.75, 0);
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 94.0, 0.5, 0);

    broker.evaluate_pending_exits(1, 20, 112.0, 93.0);

    assert_eq!(broker.orders[1].id, "XS");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 94.0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.position_size, 1.5);
    assert_eq!(pending_exit_ids(&broker), vec!["XB"]);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn same_identity_single_trigger_and_bracket_replacement_releases_reservation() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("X".to_owned(), "L".to_owned(), 95.0, 1.5, 0);

    broker.place_exit_bracket_qty("X".to_owned(), "L".to_owned(), 94.0, 110.0, 2.0, 1);

    assert_eq!(pending_exit_count(&broker), 1);
    let pending = broker
        .pending_exit_by_identity("X", "L")
        .expect("replacement should exist");
    assert_eq!(
        pending.trigger,
        PendingExitTrigger::Bracket {
            downside: 94.0,
            upside: 110.0,
        }
    );
    assert_eq!(pending.reserved_quantity, 2.0);
    assert_eq!(pending.last_update_bar_index, 1);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn close_long_cancels_mixed_single_trigger_and_bracket_reservations() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 94.0, 110.0, 0.75, 0);

    broker.close_long("L".to_owned(), 1, 20, 105.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_trailing_and_stop_downside_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 103.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 110.0, 106.0);
    assert_active_trailing_stop_by_id(&broker, "XT", 108.0);

    broker.evaluate_pending_exits(2, 30, 109.0, 102.0);

    assert_eq!(broker.orders[1].id, "XT");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 108.0);
    assert_eq!(broker.orders[2].id, "XS");
    assert_eq!(broker.orders[2].qty, 0.75);
    assert_eq!(broker.orders[2].price, 103.0);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_trailing_and_bracket_downside_fill_in_placement_order() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 107.0, 115.0, 0.75, 0);
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);
    assert_active_trailing_stop_by_id(&broker, "XT", 108.0);

    broker.evaluate_pending_exits(2, 30, 114.0, 106.0);

    assert_eq!(broker.orders[1].id, "XB");
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.orders[1].price, 107.0);
    assert_eq!(broker.orders[2].id, "XT");
    assert_eq!(broker.orders[2].qty, 0.5);
    assert_eq!(broker.orders[2].price, 108.0);
    assert_eq!(broker.position_size, 0.75);
    assert_eq!(pending_exit_count(&broker), 0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_trailing_downside_wins_over_upside_candidates_and_preserves_them() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 111.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 95.0, 112.0, 0.5, 0);
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    broker.evaluate_pending_exits(1, 20, 110.0, 106.0);
    assert_active_trailing_stop_by_id(&broker, "XT", 108.0);

    broker.evaluate_pending_exits(2, 30, 113.0, 107.0);

    assert_eq!(broker.orders[1].id, "XT");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 108.0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.position_size, 1.5);
    assert_eq!(pending_exit_ids(&broker), vec!["XL", "XB"]);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn mixed_inactive_trailing_activation_and_upside_fill_same_bar_preserves_trailing() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );
    broker.place_exit_limit_qty("XL".to_owned(), "L".to_owned(), 109.0, 0.75, 0);

    broker.evaluate_pending_exits(1, 20, 110.0, 103.0);

    assert_eq!(broker.orders[1].id, "XL");
    assert_eq!(broker.orders[1].qty, 0.75);
    assert_eq!(broker.orders[1].price, 109.0);
    assert_eq!(broker.position_size, 1.25);
    assert_eq!(pending_exit_ids(&broker), vec!["XT"]);
    assert_active_trailing_stop_by_id(&broker, "XT", 108.0);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn same_identity_replacement_between_trailing_and_other_families_releases_reservation() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price_qty(
        "X".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.5,
        0,
    );
    broker.place_exit_stop_qty("Y".to_owned(), "L".to_owned(), 94.0, 0.5, 0);

    broker.place_exit_stop_qty("X".to_owned(), "L".to_owned(), 95.0, 1.5, 1);

    let replaced = broker
        .pending_exit_by_identity("X", "L")
        .expect("single-trigger replacement should exist");
    assert_eq!(replaced.trigger, PendingExitTrigger::Stop(95.0));
    assert_eq!(replaced.reserved_quantity, 1.5);
    assert_eq!(replaced.last_update_bar_index, 1);
    assert!(broker.diagnostics.is_empty());

    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("X".to_owned(), "L".to_owned(), 95.0, 110.0, 1.5, 0);
    broker.place_exit_stop_qty("Y".to_owned(), "L".to_owned(), 94.0, 0.5, 0);

    broker.place_exit_trail_price_qty(
        "X".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        1.5,
        1,
    );

    let replaced = broker
        .pending_exit_by_identity("X", "L")
        .expect("trailing replacement should exist");
    assert_eq!(replaced.trigger, trailing_price_trigger(105.0, 2.0));
    assert_eq!(replaced.reserved_quantity, 1.5);
    assert_eq!(replaced.last_update_bar_index, 1);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn close_long_cancels_mixed_single_bracket_and_trailing_reservations() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop_qty("XS".to_owned(), "L".to_owned(), 95.0, 0.5, 0);
    broker.place_exit_bracket_qty("XB".to_owned(), "L".to_owned(), 94.0, 110.0, 0.5, 0);
    broker.place_exit_trail_price_qty(
        "XT".to_owned(),
        "L".to_owned(),
        TrailPriceExitSpec {
            activation_price: 105.0,
            offset_ticks: 4.0,
            mintick: 0.5,
        },
        0.5,
        0,
    );

    broker.close_long("L".to_owned(), 1, 20, 105.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn two_percent_brackets_reserve_expected_absolute_quantities() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty_percent("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 25.0, 0);
    broker.place_exit_bracket_qty_percent("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 50.0, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(pending_exit_ids(&broker), vec!["XB1", "XB2"]);
    let first = broker.pending_exit_by_identity("XB1", "L").unwrap();
    assert_eq!(first.quantity, PendingExitQuantity::Fixed(0.5));
    assert_eq!(first.reserved_quantity, 0.5);
    let second = broker.pending_exit_by_identity("XB2", "L").unwrap();
    assert_eq!(second.quantity, PendingExitQuantity::Fixed(1.0));
    assert_eq!(second.reserved_quantity, 1.0);

    broker.evaluate_pending_exits(1, 20, 112.0, 97.0);

    assert_eq!(broker.orders[1].id, "XB1");
    assert_eq!(broker.orders[1].qty, 0.5);
    assert_eq!(broker.orders[1].price, 110.0);
    assert_eq!(broker.orders[2].id, "XB2");
    assert_eq!(broker.orders[2].qty, 1.0);
    assert_eq!(broker.orders[2].price, 111.0);
    assert_eq!(broker.position_size, 0.5);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn percent_and_fixed_brackets_share_reservation_pool() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 0.75, 0);
    broker.place_exit_bracket_qty_percent("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 75.0, 0);

    assert_eq!(pending_exit_count(&broker), 2);
    assert_eq!(
        broker
            .pending_exit_by_identity("XB2", "L")
            .expect("percent bracket should exist")
            .quantity,
        PendingExitQuantity::Fixed(1.5)
    );
    assert_eq!(
        broker
            .pending_exit_by_identity("XB2", "L")
            .expect("percent bracket should exist")
            .reserved_quantity,
        1.25
    );
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn percent_bracket_replacement_releases_old_reservation_first() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty_percent("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 25.0, 0);
    broker.place_exit_bracket_qty_percent("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 25.0, 0);

    broker.place_exit_bracket_qty_percent("XB1".to_owned(), "L".to_owned(), 94.0, 109.0, 75.0, 1);

    assert_eq!(pending_exit_ids(&broker), vec!["XB1", "XB2"]);
    let replaced = broker.pending_exit_by_identity("XB1", "L").unwrap();
    assert_eq!(
        replaced.trigger,
        PendingExitTrigger::Bracket {
            downside: 94.0,
            upside: 109.0,
        }
    );
    assert_eq!(replaced.quantity, PendingExitQuantity::Fixed(1.5));
    assert_eq!(replaced.reserved_quantity, 1.5);
    assert_eq!(replaced.last_update_bar_index, 1);
    let preserved = broker.pending_exit_by_identity("XB2", "L").unwrap();
    assert_eq!(preserved.reserved_quantity, 0.5);
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn over_100_percent_bracket_reserves_remaining_unreserved_quantity() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 0.75, 0);
    broker.place_exit_bracket_qty_percent("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 150.0, 0);

    let percent = broker.pending_exit_by_identity("XB2", "L").unwrap();
    assert_eq!(percent.quantity, PendingExitQuantity::Fixed(3.0));
    assert_eq!(percent.reserved_quantity, 1.25);
    assert_eq!(
        broker.pending_exits.total_reserved_for_entry("L", None),
        2.0
    );
    assert!(broker.diagnostics.is_empty());
}

#[test]
fn percent_bracket_with_no_unreserved_quantity_is_rejected() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty_percent("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 50.0, 0);
    broker.place_exit_bracket_qty_percent("XB2".to_owned(), "L".to_owned(), 96.0, 111.0, 50.0, 0);

    broker.place_exit_bracket_qty_percent("XB3".to_owned(), "L".to_owned(), 97.0, 112.0, 25.0, 0);

    assert_eq!(pending_exit_ids(&broker), vec!["XB1", "XB2"]);
    assert!(broker.pending_exit_by_identity("XB3", "L").is_none());
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY");
}

#[test]
fn invalid_percent_bracket_replacement_preserves_existing_pending_bracket() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_bracket_qty_percent("XB1".to_owned(), "L".to_owned(), 95.0, 110.0, 25.0, 0);

    broker.place_exit_bracket_qty_percent(
        "XB1".to_owned(),
        "L".to_owned(),
        94.0,
        109.0,
        f64::NAN,
        1,
    );

    assert_eq!(pending_exit_count(&broker), 1);
    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XB1".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Bracket {
                downside: 95.0,
                upside: 110.0,
            },
            quantity: PendingExitQuantity::Fixed(0.5),
            reserved_quantity: 0.5,
            multiple_reservation: true,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_QTY_PERCENT");
}

#[test]
fn pending_trailing_is_not_eligible_on_creation_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);

    broker.evaluate_pending_exits(0, 10, 110.0, 100.0);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XT".to_owned(),
            from_entry: "L".to_owned(),
            trigger: trailing_price_trigger(105.0, 2.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_trailing_activation_does_not_fill_on_activation_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);

    broker.evaluate_pending_exits(1, 20, 110.0, 100.0);

    assert_active_trailing_stop(&broker, 108.0);
    assert!(
        broker
            .orders
            .iter()
            .all(|order| order.direction != "strategy.exit")
    );
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_trailing_ratchets_after_activation_without_filling() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 113.0, 109.0);

    assert_active_trailing_stop(&broker, 111.0);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_trailing_stop_never_decreases() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 109.0, 108.5);

    assert_active_trailing_stop(&broker, 108.0);
    assert!(broker.trades.is_empty());
    assert_eq!(broker.position_size, 2.0);
}

#[test]
fn pending_trailing_active_stop_fills_before_ratchet() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_trail_price("XT".to_owned(), "L".to_owned(), 105.0, 4.0, 0.5, 0);
    broker.evaluate_pending_exits(1, 20, 110.0, 109.0);

    broker.evaluate_pending_exits(2, 30, 115.0, 107.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.orders.len(), 2);
    assert_eq!(broker.orders[1].id, "XT");
    assert_eq!(broker.orders[1].direction, "strategy.exit");
    assert_eq!(broker.orders[1].price, 108.0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 108.0);
    assert_eq!(broker.trades[0].profit, 16.0);
    assert_eq!(broker.position_size, 0.0);
}

#[test]
fn invalid_profit_ticks_record_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);

    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 0.0, 0.01, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XS".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_TICKS");
}

#[test]
fn invalid_exit_mintick_records_diagnostic_without_changing_pending_exit() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_limit("XL".to_owned(), "L".to_owned(), 110.0, 0);

    broker.place_exit_loss_ticks("XS".to_owned(), "L".to_owned(), 5.0, f64::NAN, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(110.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 0,
        })
    );
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_MINTICK");
}

#[test]
fn profit_ticks_without_matching_entry_record_diagnostic_without_pending_state() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_profit_ticks("XP".to_owned(), "OTHER".to_owned(), 10.0, 0.01, 0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.diagnostics.len(), 1);
    assert_eq!(broker.diagnostics[0].code, "E_STRATEGY_EXIT_ENTRY");
}

#[test]
fn unchanged_repeated_profit_ticks_keep_original_eligibility_bar() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 0);
    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 1);

    broker.evaluate_pending_exits(1, 20, 111.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 110.0);
}

#[test]
fn changed_repeated_profit_ticks_delay_eligibility() {
    let mut broker = broker_with_long_entry();
    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 0);
    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 11.0, 1.0, 1);

    broker.evaluate_pending_exits(1, 20, 112.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 1);
    assert!(broker.trades.is_empty());

    broker.evaluate_pending_exits(2, 30, 112.0, 100.0);

    assert_eq!(pending_exit_count(&broker), 0);
    assert_eq!(broker.trades.len(), 1);
    assert_eq!(broker.trades[0].exit_price, 111.0);
}

#[test]
fn profit_ticks_replace_stop_and_loss_ticks_replace_limit() {
    let mut broker = broker_with_long_entry();

    broker.place_exit_stop("XS".to_owned(), "L".to_owned(), 95.0, 0);
    broker.place_exit_profit_ticks("XP".to_owned(), "L".to_owned(), 10.0, 1.0, 1);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XP".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Limit(110.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 1,
        })
    );

    broker.place_exit_loss_ticks("XL".to_owned(), "L".to_owned(), 5.0, 1.0, 2);

    assert_eq!(
        broker.pending_exit().cloned(),
        Some(PendingExit {
            id: "XL".to_owned(),
            from_entry: "L".to_owned(),
            trigger: PendingExitTrigger::Stop(95.0),
            quantity: PendingExitQuantity::Full,
            reserved_quantity: 2.0,
            multiple_reservation: false,
            last_update_bar_index: 2,
        })
    );
}
