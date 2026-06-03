# Strategy Internal Stage 7 Trade Records Audit

Status: in progress. Slices 0, 1, 2, and 3 closed on 2026-06-02; Slices 4,
5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
25, and 26 closed on 2026-06-03.

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
  `entry_id`, `entry_bar_index`, `entry_time`, `size`, `profit`, and
  `commission`, `max_runup`, and `max_drawdown`;
- closed-trade field functions beyond `entry_price`, `entry_id`, `exit_price`,
  `exit_id`, `entry_bar_index`, `exit_bar_index`, `entry_time`, `exit_time`,
  `commission`, `size`, `profit`, `max_runup`, and `max_drawdown`;
- richer reporting metrics;
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
- `commission` returns `0.0` without configured commission, and later cost
  slices may wire supported commission models into the same script-visible
  function;
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
  `strategy.opentrades.entry_comment(0)`;
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

## Slice 9: Open Trade Size Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.size(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the current open position size already tracked by the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 10: Open Trade Profit Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.profit(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the current close-based floating profit already tracked by
  the broker for the current open position;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 11: Open Trade Entry Id Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.entry_id(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value reads the retained current entry id already tracked by the broker;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other open-trade fields out of scope with
  `strategy.opentrades.entry_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 12: Open Trade Commission Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.commission(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value is `0.0` without configured commission, and later cost slices may
  wire supported commission models into the same script-visible function;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other open-trade fields out of scope with
  `strategy.opentrades.entry_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 13: Open Trade Max Runup Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.max_runup(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value is the largest high-based favorable excursion seen so far for the
  current supported open long position;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other open-trade fields out of scope with
  `strategy.opentrades.entry_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 14: Open Trade Max Drawdown Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.opentrades.max_drawdown(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num == 0` addresses the current supported single open long position;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- flat state returns `na`;
- the value is the largest low-based adverse excursion seen so far for the
  current supported open long position;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or open-trade records.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_opentrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_opentrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other open-trade fields out of scope with
  `strategy.opentrades.entry_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 15: Closed Trade Max Runup Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.closedtrades.max_runup(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` is a zero-based integer index into the current closed-trade list;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- the value is the largest high-based favorable excursion retained for the
  closed trade quantity;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_closedtrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_closedtrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other closed-trade fields out of scope
  with `strategy.closedtrades.exit_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 16: Closed Trade Max Drawdown Function

Closed on 2026-06-03.

Supported script-visible function:

- `strategy.closedtrades.max_drawdown(trade_num)`.

Contract:

- strategy-mode scripts only;
- `trade_num` is a zero-based integer index into the current closed-trade list;
- missing, negative, out-of-range, or non-integer indexes return `na`;
- the value is the largest low-based adverse excursion retained for the closed
  trade quantity;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_closedtrades_fields.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_closedtrades_fields.pine`;
- requested-context negative fixture:
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`;
- unsupported namespace fixture keeps other closed-trade fields out of scope
  with `strategy.closedtrades.exit_comment(0)`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 17: Cash Per Contract Commission

Closed on 2026-06-03.

Supported declaration subset:

- `strategy(..., commission_type=strategy.commission.cash_per_contract,
  commission_value=N)`.

Contract:

- strategy-mode scripts only;
- `commission_value` must be a finite non-negative const numeric value;
- each supported entry and exit debits `qty * commission_value`;
- closed trade `profit`, `strategy.netprofit`, and trade-count outcomes use net
  realized profit after entry-plus-exit commission for the closed quantity;
- `strategy.closedtrades.commission(trade_num)` returns entry-plus-exit
  cash-per-contract commission for the closed quantity;
- `strategy.opentrades.commission(trade_num)` returns the current open
  cash-per-contract entry commission for `trade_num == 0`;
- equity snapshots and `strategy.equity` include supported commission cash
  debits;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixture:
  `tests/fixtures/runtime/strategy_commission_cash_per_contract.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_commission_cash_per_contract.pine`
  and `tests/fixtures/sema/unsupported_strategy_commission_unknown.pine`;
- host parity tests cover CLI snapshots plus Python plot and trade/equity
  values.

## Slice 18: Cash Per Order Commission

Closed on 2026-06-03.

Supported declaration subset:

- `strategy(..., commission_type=strategy.commission.cash_per_order,
  commission_value=N)`.

Contract:

- strategy-mode scripts only;
- `commission_value` must be a finite non-negative const numeric value;
- each supported entry and exit fill debits one fixed `commission_value`;
- closed trade `profit`, `strategy.netprofit`, and trade-count outcomes use net
  realized profit after allocated entry commission plus exit commission for the
  closed quantity;
- partial exits allocate the original entry-order commission proportionally to
  the closed quantity and leave the remainder attached to the open trade;
- `strategy.closedtrades.commission(trade_num)` returns allocated entry
  cash-per-order commission plus exit cash-per-order commission for the closed
  quantity;
- `strategy.opentrades.commission(trade_num)` returns the remaining open
  cash-per-order entry commission for `trade_num == 0`;
- equity snapshots and `strategy.equity` include supported commission cash
  debits;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixture:
  `tests/fixtures/runtime/strategy_commission_cash_per_order.pine`;
- semantic fixture:
  `tests/fixtures/sema/supported_strategy_commission_cash_per_order.pine`;
- unsupported commission modes remain covered by
  `tests/fixtures/sema/unsupported_strategy_commission_unknown.pine`;
- host parity tests cover CLI snapshots plus Python plot and trade/equity
  values.

## Slice 19: Fixed Tick Slippage

Closed on 2026-06-03.

Supported declaration subset:

- `strategy(..., slippage=N)`.

Contract:

- strategy-mode scripts only;
- `slippage` must be a finite non-negative integer const tick count;
- ticks convert through the current fixed `syminfo.mintick` subset;
- supported long entry fill prices are worsened upward after trigger selection;
- supported long close and `strategy.exit` fill prices are worsened downward
  after trigger selection;
- trigger conditions are unchanged by slippage;
- order fill prices, trade entry/exit prices, realized profit, floating profit,
  and equity snapshots use the adjusted fill prices;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixtures: `tests/fixtures/runtime/strategy_slippage.pine` and
  `tests/fixtures/runtime/strategy_exit_slippage.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_slippage.pine` and
  `tests/fixtures/sema/unsupported_strategy_slippage.pine`;
- host parity tests cover CLI snapshots plus Python plot and trade/equity
  values.

## Slice 20: Fixed Tick Limit Verification

Closed on 2026-06-03.

Supported declaration subset:

- `strategy(..., backtest_fill_limits_assumption=N)`.

Contract:

- strategy-mode scripts only;
- `backtest_fill_limits_assumption` must be a finite non-negative integer const
  tick count;
- ticks convert through the current fixed `syminfo.mintick` subset;
- supported long limit entries and stop-limit entry limit legs require
  `low <= limit - ticks * syminfo.mintick`;
- supported long limit/profit exit fills and bracket upside fills require
  `high >= limit_or_profit_price + ticks * syminfo.mintick`;
- verified limit orders still fill at the original limit/profit price, not the
  verification threshold;
- stop/loss/trailing triggers and slippage direction are unchanged;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixtures:
  `tests/fixtures/runtime/strategy_limit_verification_entry.pine` and
  `tests/fixtures/runtime/strategy_limit_verification_exit.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_limit_verification.pine` and
  `tests/fixtures/sema/unsupported_strategy_limit_verification.pine`;
- host parity tests cover CLI snapshots plus Python trade/order values.

## Slice 21: Percent Commission

Closed on 2026-06-03.

Supported declaration subset:

- `strategy(..., commission_type=strategy.commission.percent,
  commission_value=N)`.

Contract:

- strategy-mode scripts only;
- `commission_value` must be a finite non-negative const numeric value;
- each supported entry and exit fill debits
  `qty * fill_price * commission_value / 100`;
- closed trade `profit`, `strategy.netprofit`, and trade-count outcomes use net
  realized profit after allocated entry commission plus exit commission for the
  closed quantity;
- partial exits allocate the original entry percentage commission
  proportionally to the closed quantity and leave the remainder attached to the
  open trade;
- `strategy.closedtrades.commission(trade_num)` returns allocated entry
  percent commission plus exit percent commission for the closed quantity;
- `strategy.opentrades.commission(trade_num)` returns the remaining open entry
  percent commission for `trade_num == 0`;
- equity snapshots and `strategy.equity` include supported percent commission
  cash debits;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing strategy
  output shape with no new top-level fields or public trade metric fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_commission_percent.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_commission_percent.pine` and
  `tests/fixtures/sema/unsupported_strategy_commission_unknown.pine`;
- host parity tests cover CLI snapshots plus Python plot and trade/equity
  values.

## Slice 22: Gross Profit State Variable

Closed on 2026-06-03.

Supported variable:

- `strategy.grossprofit`.

Contract:

- strategy-mode scripts only;
- read-only `series float`;
- returns cumulative positive realized closed-trade profit;
- losing, flat, and current open trades do not change it;
- supported commission, slippage, and limit verification feed into realized
  closed-trade profit before the positive-only sum;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing
  strategy output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_trade_outcome_counts.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_profit_state.pine`,
  `tests/fixtures/sema/unsupported_strategy_state_indicator.pine`,
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`, and
  `tests/fixtures/sema/unsupported_strategy_state_mutation.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 23: Gross Loss State Variable

Closed on 2026-06-03.

Supported variable:

- `strategy.grossloss`.

Contract:

- strategy-mode scripts only;
- read-only `series float`;
- returns cumulative realized closed-trade loss as a positive value;
- winning, flat, and current open trades do not change it;
- supported commission, slippage, and limit verification feed into realized
  closed-trade profit/loss before the loss-only sum;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing
  strategy output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_trade_outcome_counts.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_profit_state.pine`,
  `tests/fixtures/sema/unsupported_strategy_state_indicator.pine`,
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`, and
  `tests/fixtures/sema/unsupported_strategy_state_mutation.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 24: Average Trade State Variable

Closed on 2026-06-03.

Supported variable:

- `strategy.avg_trade`.

Contract:

- strategy-mode scripts only;
- read-only `series float`;
- returns average realized profit/loss per closed trade;
- returns `na` before the first closed trade;
- current open trades do not affect it;
- supported commission, slippage, and limit verification feed into realized
  closed-trade profit/loss before the average;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing
  strategy output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_trade_outcome_counts.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_profit_state.pine`,
  `tests/fixtures/sema/unsupported_strategy_state_indicator.pine`,
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`, and
  `tests/fixtures/sema/unsupported_strategy_state_mutation.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 25: Average Winning Trade State Variable

Closed on 2026-06-03.

Supported variable:

- `strategy.avg_winning_trade`.

Contract:

- strategy-mode scripts only;
- read-only `series float`;
- returns average realized profit among winning closed trades only;
- returns `na` before the first winning closed trade;
- losing, flat, and current open trades do not affect it;
- supported commission, slippage, and limit verification feed into realized
  closed-trade profit/loss before filtering and averaging;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing
  strategy output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_trade_outcome_counts.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_profit_state.pine`,
  `tests/fixtures/sema/unsupported_strategy_state_indicator.pine`,
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`, and
  `tests/fixtures/sema/unsupported_strategy_state_mutation.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Slice 26: Average Losing Trade State Variable

Closed on 2026-06-03.

Supported variable:

- `strategy.avg_losing_trade`.

Contract:

- strategy-mode scripts only;
- read-only `series float`;
- returns average realized loss among losing closed trades only as a positive
  value;
- returns `na` before the first losing closed trade;
- winning, flat, and current open trades do not affect it;
- supported commission, slippage, and limit verification feed into realized
  closed-trade profit/loss before filtering and averaging;
- public CLI JSON, Python dictionaries, and WASM JSON keep the existing
  strategy output shape with no new top-level fields.

Evidence:

- runtime fixture: `tests/fixtures/runtime/strategy_trade_outcome_counts.pine`;
- semantic fixtures:
  `tests/fixtures/sema/supported_strategy_profit_state.pine`,
  `tests/fixtures/sema/unsupported_strategy_state_indicator.pine`,
  `tests/fixtures/sema/unsupported_request_strategy_state.pine`, and
  `tests/fixtures/sema/unsupported_strategy_state_mutation.pine`;
- host parity tests cover CLI snapshots plus Python and WASM plot values.

## Remaining Stage 7 Work

The next slice should choose one explicitly bounded accounting/reporting
addition, such as a richer fill-model setting or another closed/open-trade
field, only after documenting whether the behavior is script-only or
public-output visible.
