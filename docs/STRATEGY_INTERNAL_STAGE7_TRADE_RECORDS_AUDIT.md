# Strategy Internal Stage 7 Trade Records Audit

Status: in progress. Slices 0, 1, 2, and 3 closed on 2026-06-02; Slices 4,
5, 6, 7, and 8 closed on 2026-06-03.

Stage 7 enriches strategy reporting and accounting while preserving the current
one-net-long broker and public output contract unless a later slice explicitly
designs a new host schema.

## Slice 0: Closed Trade Field Functions

Closed on 2026-06-02.

Supported script-visible functions:

- `strategy.closedtrades.entry_price(trade_num)`;
- `strategy.closedtrades.exit_price(trade_num)`;
- `strategy.closedtrades.entry_bar_index(trade_num)`;
- `strategy.closedtrades.exit_bar_index(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` is a zero-based integer index into the current closed-trade list;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- values read the existing closed trade records retained by the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_closedtrades_fields.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_closedtrades_fields.pine`,
  `tests/fixtures/sema/unsupported_strategy_closedtrades_fields_indicator.pine`,
  and `tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

Still unsupported:

- `strategy.opentrades.*` namespace functions outside `entry_price`,
  `entry_bar_index`, and `entry_time`;
- closed-trade field functions beyond `entry_price`, `entry_id`, `exit_price`,
  `exit_id`, `entry_bar_index`, `exit_bar_index`, `entry_time`, `exit_time`,
  `commission`, `size`, and `profit`;
- runup, drawdown, and richer reporting metrics;
- public trade namespace schema expansion.

## Slice 1: Closed Trade Size And Profit Functions

Closed on 2026-06-02.

Supported script-visible functions:

- `strategy.closedtrades.size(trade_num)`;
- `strategy.closedtrades.profit(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` follows the same zero-based integer index contract as Slice 0;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- `size` returns the closed trade's absolute filled quantity in the current
  long-only subset;
- `profit` returns the closed trade's realized profit;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

## Slice 2: Closed Trade Time Functions

Closed on 2026-06-02.

Supported script-visible functions:

- `strategy.closedtrades.entry_time(trade_num)`;
- `strategy.closedtrades.exit_time(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` follows the same zero-based integer index contract as Slices 0
  and 1;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- `entry_time` and `exit_time` return the timestamps already retained on the
  closed trade record;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

## Slice 3: Closed Trade Commission Function

Closed on 2026-06-02.

Supported script-visible function:

- `strategy.closedtrades.commission(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` follows the same zero-based integer index contract as Slices 0,
  1, and 2;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- `commission` returns `0.0` for closed trades because the current account model
  has no commission calculation;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

## Slice 4: Closed Trade Entry Id Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.closedtrades.entry_id(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` follows the same zero-based integer index contract as Slices 0
  through 3;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- `entry_id` returns the entry id already retained on the closed trade record;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

## Slice 5: Closed Trade Exit Id Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.closedtrades.exit_id(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` follows the same zero-based integer index contract as Slices 0
  through 4;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- `exit_id` returns the close id for `strategy.close` / `strategy.close_all`
  fills and the pending exit id for `strategy.exit` fills;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields.

## Slice 6: Open Trade Entry Price Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.entry_price(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the current open position average entry price already tracked
  by the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- unsupported namespace fixture keeps other open-trade fields out of scope with
  `strategy.opentrades.entry_id(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 7: Open Trade Entry Bar Index Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.entry_bar_index(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the current open position entry fill bar already tracked by
  the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 8: Open Trade Entry Time Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.entry_time(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the current open position entry fill timestamp already
  tracked by the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Remaining Stage 7 Work

The next slice should choose one explicitly bounded accounting/reporting
addition, such as real commission/slippage modeling, runup/drawdown, or another
closed/open-trade field, only after documenting whether the behavior is
script-only or public-output visible.
