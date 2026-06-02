# Strategy Internal Stage 7 Trade Records Audit

Status: in progress. Slice 0 closed on 2026-06-02.

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

- all `strategy.opentrades.*` namespace functions;
- closed-trade field functions beyond `entry_price`, `exit_price`,
  `entry_bar_index`, and `exit_bar_index`;
- runup, drawdown, commission, size, ids, times, and richer reporting metrics;
- public trade namespace schema expansion.

## Remaining Stage 7 Work

The next slice should choose one explicitly bounded accounting/reporting
addition, such as commission/slippage modeling or another closed-trade field,
only after documenting whether the behavior is script-only or public-output
visible.
