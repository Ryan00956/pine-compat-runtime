use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_if_else_reassignment_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("if")
x = close
if close > open
    x := close
else
    x := open
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(4.0, 5.0, 4.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
}

#[test]
fn runs_if_reassignment_with_var_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("if var")
var x = 0
if close > open
    x := x + 1
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(4.0, 5.0, 4.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 2.0]);
}

#[test]
fn runs_block_local_var_initializes_when_first_reached() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("local var")
if close > open
    var seen = 10
    seen := seen + 1
    plot(seen)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(4.0, 6.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[11.0, 12.0]);
}

#[test]
fn runs_for_body_var_persists_across_iterations_and_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for var")
out = 0
for i = 0 to 2
    var count = 0
    count := count + 1
    out := count
plot(out)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 6.0, 9.0]);
}

#[test]
fn runs_local_varip_with_var_like_historical_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("local varip")
branch_out = 0
if close >= 3
    varip branch_count = 0
    branch_count := branch_count + 1
    branch_out := branch_count
plot(branch_out)

for_out = 0
for i = 0 to 1
    varip for_count = 0
    for_count := for_count + 1
    for_out := for_count
plot(for_out)

while_out = 0
j = 0
while j < 2
    varip while_count = 0
    while_count := while_count + 1
    while_out := while_count
    j := j + 1
plot(while_out)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[0.0, 0.0, 1.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[2.0, 4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 4.0, 6.0, 8.0]);
}

#[test]
fn runs_udf_local_var_independently_per_callsite() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf var")
counter() =>
    var value = 0
    value := value + 1
    value
plot(counter() + counter())
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}

#[test]
fn runs_udf_local_varip_independently_per_callsite() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf varip")
bump(step) =>
    varip value = 0
    value := value + step
    value
plot(bump(1))
plot(bump(10))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[10.0, 20.0, 30.0]);
}

#[test]
fn runs_else_if_branches() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("else if")
x = close
if close > 6
    x := 10.0
else if close > 3
    x := 5.0
else
    x := 1.0
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(2.0), bar(4.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 5.0, 10.0]);
}

#[test]
fn runs_nested_if_branches() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("nested if")
x = close
if close > open
    if high > close
        x := high
    else
        x := close
else
    x := open
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(4.0, 6.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 3.0, 6.0]);
}

#[test]
fn runs_block_local_declaration_in_if() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("block local")
if close > open
    spread = high - low
    plot(spread)
else
    spread = open - close
    plot(spread)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(4.0, 5.0, 3.0, 2.0),
        bar_ohlc(2.0, 8.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values[..1], &[2.0]);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[4.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..2], &[2.0]);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
}

#[test]
fn runs_block_local_tuple_declaration_in_if() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("block local tuple")
if close > open
    [hi, lo] = [high, low]
    plot(hi - lo)
else
    [hi, lo] = [open, close]
    plot(hi - lo)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(4.0, 5.0, 3.0, 2.0),
        bar_ohlc(2.0, 8.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values[..1], &[2.0]);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[4.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..2], &[2.0]);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
}

#[test]
fn runs_block_local_tuple_declaration_shadowing_outer_symbols() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("tuple shadow")
x = close
y = close
if close > open
    [x, y] = [high, low]
    plot(x - y)
plot(x)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(4.0, 5.0, 3.0, 2.0),
        bar_ohlc(2.0, 8.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 2);
    assert_values_close(&result.plots[0].values[..1], &[2.0]);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[4.0]);
    assert_values_close(&result.plots[1].values, &[2.0, 2.0, 6.0]);
}

#[test]
fn advances_conditional_tuple_builtin_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional bb")
if close > open
    [basis, upper, lower] = ta.bb(close, 2, 2)
    plot(basis)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(4.0, 6.0, 4.0, 6.0),
        bar_ohlc(5.0, 8.0, 5.0, 8.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[4.0, 7.0]);
}

#[test]
fn runs_expression_body_function() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf")
double(x) => x * 2
plot(double(close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}

#[test]
fn runs_function_body_with_global_reference() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf global")
bias = 1.5
add_bias(x) => x + bias
plot(add_bias(close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.5, 3.5, 4.5]);
}

#[test]
fn runs_block_body_function() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf block")
spread(hi, lo) =>
    value = hi - lo
    value * 2
plot(spread(high, low))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(2.0, 6.0, 3.0, 5.0),
        bar_ohlc(5.0, 9.0, 4.0, 7.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 10.0]);
}

#[test]
fn runs_if_reassignment_inside_block_body_function() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf if")
select_value(x, y) =>
    result = y
    if x > y
        result := x
    result
plot(select_value(high, close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(4.0, 4.0, 2.0, 5.0),
        bar_ohlc(2.0, 8.0, 4.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 5.0, 8.0]);
}

#[test]
fn runs_for_loop_reassignment() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for")
sum = 0
for i = 0 to 4 by 2
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_for_loop_with_computed_integer_bound() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("computed for bound")
n = 3
sum = 0
for i = 0 to n - 1
    sum := sum + 1
plot(sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 3.0, 3.0]);
}

#[test]
fn runs_descending_for_loop_reassignment() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for desc")
sum = 0
for i = 4 to 0 by 2
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_for_loop_step_that_overshoots_end() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for overshoot")
sum = 0
for i = 0 to 5 by 2
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_for_loop_signed_step_by_range_direction() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for signed step")
sum = 0
for i = 0 to 4 by -2
    sum := sum + i
down = 0
for j = 4 to 0 by -2
    down := down + j
plot(close + sum + down)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[13.0, 14.0, 15.0]);
}

#[test]
fn runs_for_loop_with_series_na_bounds() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for na bounds")
limit = close > 1 ? 3 : na
sum = close > 0 ? 0.0 : 0.0
for i = 0 to limit by 2
    sum := sum + i
value = for j = limit to 0 by 2
    j
plot(close + sum + nz(value))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 5.0, 6.0]);
}

#[test]
fn runs_for_loop_break_and_continue() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for control")
sum = 0
for i = 0 to 5
    if i == 2
        continue
    if i == 4
        break
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[5.0, 6.0, 7.0]);
}

#[test]
fn runs_nested_for_loop_control_on_nearest_loop() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("nested for control")
sum = 0
for outer = 0 to 1
    for inner = 0 to 3
        if inner == 1
            continue
        if inner == 3
            break
        sum := sum + outer + inner
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_for_loop_inside_block_body_function() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf for")
repeat3(x) =>
    result = x * 0
    for i = 0 to 2
        result := result + x
    result
plot(repeat3(close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 6.0, 9.0]);
}

#[test]
fn runs_udf_local_declaration_shadowing_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf shadow")
bump(x) =>
    x = x + 1
    x
plot(bump(close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 4.0]);
}

#[test]
fn runs_udf_loop_counter_shadowing_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf counter shadow")
mix(x) =>
    total = 0
    for x = 0 to 2
        total := total + x
    total + x
plot(mix(close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[4.0, 5.0, 6.0]);
}

#[test]
fn runs_for_expression_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for expression")
value = for i = 0 to 5
    if i == 2
        continue
    if i == 4
        break
    i * 2
plot(close + value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_tuple_for_expression_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("tuple for expression")
[x, y] = for i = 0 to 3
    if i == 1
        continue
    if i == 3
        break
    [i, i * 2]
plot(close + x + y)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[7.0, 8.0, 9.0]);
}

#[test]
fn runs_for_expression_that_reaches_no_result_as_na() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for no result")
value = for i = 0 to 2
    if i >= 0
        continue
    i
plot(nz(value) + close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
}

#[test]
fn runs_while_loop_reassignment() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while")
i = 0
sum = 0
while i < 5
    i := i + 1
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[16.0, 17.0, 18.0]);
}

#[test]
fn runs_while_loop_break_and_continue() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while control")
i = 0
sum = 0
while i < 6
    i := i + 1
    if i > 1 and i < 3
        continue
    if i > 4
        break
    sum := sum + i
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[9.0, 10.0, 11.0]);
}

#[test]
fn runs_while_loop_with_na_condition() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while na condition")
i = 0
sum = close > 0 ? 0.0 : 0.0
while close > 1 ? i < 3 : na
    sum := sum + i
    i := i + 1
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 5.0, 6.0]);
}

#[test]
fn runs_nested_while_loop_control_on_nearest_loop() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("nested while control")
outer = 0
sum = 0
while outer < 2
    inner = 0
    while inner < 4
        inner := inner + 1
        if inner == 2
            continue
        if inner == 4
            break
        sum := sum + outer + inner
    outer := outer + 1
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[11.0, 12.0, 13.0]);
}

#[test]
fn runs_while_body_var_persists_across_iterations_and_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while local var")
i = 0
total = 0
while i < 2
    var seen = 0
    seen := seen + 1
    total := seen
    i := i + 1
plot(total)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}

#[test]
fn runs_loops_inside_if_branches() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("loops in if")
sum = close > 0 ? 0.0 : 0.0
if close > 1
    for i = 0 to 2
        sum := sum + i
else
    j = 0
    while j < 2
        sum := sum + open
        j := j + 1
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 2.0, 0.0, 1.0),
        bar_ohlc(2.0, 3.0, 1.0, 2.0),
        bar_ohlc(3.0, 4.0, 2.0, 3.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[3.0, 5.0, 6.0]);
}

#[test]
fn runs_switch_inside_for_loop() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("switch in for")
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 2
    value = switch i
        0 => close
        1 => high
        => low
    sum := sum + value
plot(sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 0.0, 2.0),
        bar_ohlc(2.0, 5.0, 1.0, 4.0),
        bar_ohlc(3.0, 7.0, 2.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[5.0, 10.0, 15.0]);
}

#[test]
fn advances_stateful_calls_inside_for_loop_body() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("for stateful")
sum = close > 0 ? 0.0 : 0.0
for i = 0 to 1
    sum := sum + nz(ta.sma(close, 2))
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 5.5, 8.5]);
}

#[test]
fn runs_while_loop_inside_block_body_function() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf while")
repeat_until(src, limit) =>
    i = 0
    total = src * 0.0
    while i < limit
        total := total + src
        i := i + 1
    total
plot(repeat_until(close, 2))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 4.0, 6.0]);
}

#[test]
fn advances_stateful_calls_inside_while_loop_body() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while stateful")
i = 0
sum = close > 0 ? 0.0 : 0.0
while i < 2
    sum := sum + nz(ta.sma(close, 2))
    i := i + 1
plot(close + sum)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 5.5, 8.5]);
}

#[test]
fn rejects_while_loop_that_exceeds_iteration_guard() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("while guard")
while true
    close
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected while guard error");

    assert!(
        error
            .message
            .contains("while loop exceeded maximum iteration count"),
        "{}",
        error.message
    );
}

#[test]
fn runs_condition_switch_expression() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("condition switch")
value = switch
    close > open => high
    close < open => low
    => close
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 5.0, 0.0, 2.0),
        bar_ohlc(3.0, 6.0, 1.0, 2.0),
        bar_ohlc(2.0, 7.0, 4.0, 2.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[5.0, 1.0, 2.0]);
}

#[test]
fn runs_selector_switch_expression() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("selector switch")
direction = close > open ? 1 : close < open ? -1 : 0
value = switch direction
    1 => high
    -1 => low
    => close
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 5.0, 0.0, 2.0),
        bar_ohlc(3.0, 6.0, 1.0, 2.0),
        bar_ohlc(2.0, 7.0, 4.0, 2.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[5.0, 1.0, 2.0]);
}

#[test]
fn switch_returns_na_when_no_arm_matches() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("switch no match")
value = switch
    close > open => high
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(2.0, 5.0, 1.0, 2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values, vec![PineValue::Na]);
}

#[test]
fn advances_switch_sma_only_when_arm_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("switch conditional sma")
value = switch
    close > open => ta.sma(close, 2)
    => close
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 1.0, 0.0, 1.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(3.0, 4.0, 3.0, 4.0),
        bar_ohlc(5.0, 6.0, 5.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 2.5, 5.0]);
}

#[test]
fn runs_stateful_call_as_function_argument_once() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf arg")
duplicate(x) => x + x
plot(duplicate(ta.sma(close, 2)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.0, 5.0, 7.0]);
}

#[test]
fn runs_function_with_named_arguments() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf named args")
spread(hi, lo) => hi - lo
plot(spread(lo=low, hi=high))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 3.0, 1.0, 2.0),
        bar_ohlc(2.0, 6.0, 3.0, 5.0),
        bar_ohlc(5.0, 9.0, 4.0, 7.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
}
