use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_global_price_and_derived_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("global series")
plot(open)
plot(high)
plot(low)
plot(close)
plot(volume)
plot(time)
plot(time_close)
plot(hl2)
plot(hlc3)
plot(hlcc4)
plot(ohlc4)
plot(bar_index)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        Bar {
            time: 1000,
            open: 1.0,
            high: 5.0,
            low: -1.0,
            close: 3.0,
            volume: 10.0,
        },
        Bar {
            time: 2000,
            open: 2.0,
            high: 8.0,
            low: 0.0,
            close: 4.0,
            volume: 20.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[5.0, 8.0]);
    assert_values_close(&result.plots[2].values, &[-1.0, 0.0]);
    assert_values_close(&result.plots[3].values, &[3.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[10.0, 20.0]);
    assert_values_close(&result.plots[5].values, &[1000.0, 2000.0]);
    assert_values_close(&result.plots[6].values, &[61_000.0, 62_000.0]);
    assert_values_close(&result.plots[7].values, &[2.0, 4.0]);
    assert_values_close(&result.plots[8].values, &[7.0 / 3.0, 4.0]);
    assert_values_close(&result.plots[9].values, &[2.5, 4.0]);
    assert_values_close(&result.plots[10].values, &[2.0, 3.5]);
    assert_values_close(&result.plots[11].values, &[0.0, 1.0]);
}

#[test]
fn runs_syminfo_metadata() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("syminfo")
identity = syminfo.tickerid == "NASDAQ:AAPL" and syminfo.ticker == "AAPL" and syminfo.prefix == "NASDAQ"
details = syminfo.description == "Apple Inc." and syminfo.type == "stock" and syminfo.currency == "USD" and syminfo.basecurrency == "USD"
session = syminfo.session == "regular" and syminfo.timezone == "Etc/UTC" and syminfo.root == "AAPL" and syminfo.volumetype == "base"
plot(identity ? 1 : 0)
plot(details ? 1 : 0)
plot(session ? 1 : 0)
plot(syminfo.mintick)
plot(syminfo.pointvalue)
plot(syminfo.minmove)
plot(syminfo.pricescale)
plot(syminfo.mintick == syminfo.minmove / syminfo.pricescale and syminfo.pointvalue == 1 ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result =
        run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)]).expect("result");

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[0.01, 0.01]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
}

#[test]
fn runs_type_casts() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("casts")
truncated = int(close / 2)
from_bool = int(close > open)
as_float = float(truncated) + float(close > open)
truth = bool(close - 2)
text_number = string(close / 2)
text_bool = string(close > open)
text_string = string("ok")
shade = color(close > open ? color.green : color.red)
missing_color = color(na)
missing_int = int(na)
missing_float = float(na)
missing_bool = bool(na)
missing_string = string(na)
plot(truncated)
plot(from_bool)
plot(as_float)
plot(truth ? 1 : 0)
plot(str.length(text_number))
plot(text_bool == "true" ? 1 : 0)
plot(text_string == "ok" ? 1 : 0)
plot(shade == color.green ? 1 : 0)
plot(na(missing_int) and na(missing_float) and not missing_bool and na(missing_string) and na(missing_color) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 1.0, 1.0, 1.0),
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 3.0, 3.0),
        bar_ohlc(2.0, 5.0, 2.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[0.0, 1.0, 1.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 2.0, 1.0, 3.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 0.0, 1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[3.0, 1.0, 3.0, 3.0]);
    assert_values_close(&result.plots[5].values, &[0.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[0.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_fixnan_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("fixnan")
source = close > open ? close : na
fixed = fixnan(source)
late = bar_index > 1 ? close : na
fixed_late = fixnan(late)
color_source = close > open ? color.green : na
fixed_color = fixnan(color_source)
plot(fixed)
plot(fixed_late)
plot(fixed_color == color.green ? 1 : 0)
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
        bar_ohlc(5.0, 5.0, 5.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 6.0, 6.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[6.0, 5.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn advances_conditional_fixnan_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional fixnan")
value = close
if close > open
    source = close > 4 ? close : na
    value := fixnan(source)
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
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(4.0, 6.0, 4.0, 6.0),
        bar_ohlc(5.0, 8.0, 5.0, 8.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 6.0, 8.0]);
}
