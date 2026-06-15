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
plot(last_bar_index)
plot(last_bar_time)
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
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[2000.0, 2000.0]);
}

#[test]
fn runs_syminfo_metadata() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("syminfo")
identity = syminfo.tickerid == "NASDAQ:AAPL" and syminfo.main_tickerid == "NASDAQ:AAPL" and syminfo.ticker == "AAPL" and syminfo.prefix == "NASDAQ"
details = syminfo.description == "Apple Inc." and syminfo.type == "stock" and syminfo.currency == "USD" and syminfo.basecurrency == "USD"
session = syminfo.session == "regular" and syminfo.timezone == "Etc/UTC" and syminfo.root == "AAPL" and syminfo.volumetype == "base"
classification = syminfo.sector == "Electronic Technology" and syminfo.industry == "Telecommunications Equipment" and syminfo.country == "US"
helpers = syminfo.prefix("NASDAQ:AAPL") == "NASDAQ" and syminfo.ticker("NASDAQ:AAPL") == "AAPL" and syminfo.prefix("AAPL") == "" and syminfo.ticker("AAPL") == "AAPL"
plot(identity ? 1 : 0)
plot(details ? 1 : 0)
plot(session ? 1 : 0)
plot(classification ? 1 : 0)
plot(helpers ? 1 : 0)
plot(syminfo.mintick)
plot(syminfo.mincontract)
plot(syminfo.pointvalue)
plot(syminfo.minmove)
plot(syminfo.pricescale)
plot(syminfo.mintick == syminfo.minmove / syminfo.pricescale and syminfo.pointvalue == 1 and syminfo.mincontract == 1 ? 1 : 0)
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
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[0.01, 0.01]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
}

#[test]
fn runs_ticker_standard_subset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("ticker standard")
created = ticker.new("NASDAQ", "AAPL")
chart_created = ticker.new(syminfo.prefix, syminfo.ticker)
missing_created = ticker.new(na, "AAPL")
extended_created = ticker.new("NASDAQ", "AAPL", session.extended)
chart_session_created = ticker.new(syminfo.prefix, syminfo.ticker, syminfo.session)
missing_session_created = ticker.new("NASDAQ", "AAPL", na)
adjusted_created = ticker.new("NASDAQ", "AAPL", session.extended, adjustment.dividends)
missing_adjusted_created = ticker.new("NASDAQ", "AAPL", session.extended, na)
modified_identity = ticker.modify(created)
modified_extended = ticker.modify(created, session.extended)
modified_regular = ticker.modify(extended_created, session.regular)
modified_adjusted = ticker.modify(created, session.extended, adjustment.splits)
missing_modified = ticker.modify(na)
missing_modified_session = ticker.modify(created, na)
missing_modified_adjustment = ticker.modify(created, session.extended, na)
plain = ticker.standard("NASDAQ:AAPL")
current = ticker.standard(syminfo.tickerid)
extended_standard = ticker.standard(extended_created)
modified_standard = ticker.standard(modified_regular)
adjusted_standard = ticker.standard(modified_adjusted)
standard_from_modified = ticker.standard("{\"session\":\"extended\",\"symbol\":\"NASDAQ:AAPL\"}")
ha_created = ticker.heikinashi(created)
ha_adjusted = ticker.heikinashi(adjusted_created)
ha_standard = ticker.standard(ha_adjusted)
missing_ha = ticker.heikinashi(na)
missing = ticker.standard(na)
plot(created == "NASDAQ:AAPL" ? 1 : 0)
plot(chart_created == "NASDAQ:AAPL" ? 1 : 0)
plot(na(missing_created) ? 1 : 0)
plot(extended_created == "{\"session\":\"extended\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(chart_session_created == "{\"session\":\"regular\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(na(missing_session_created) ? 1 : 0)
plot(adjusted_created == "{\"session\":\"extended\",\"adjustment\":\"dividends\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(na(missing_adjusted_created) ? 1 : 0)
plot(modified_identity == "NASDAQ:AAPL" ? 1 : 0)
plot(modified_extended == "{\"session\":\"extended\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(modified_regular == "{\"session\":\"regular\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(modified_adjusted == "{\"session\":\"extended\",\"adjustment\":\"splits\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(na(missing_modified) ? 1 : 0)
plot(na(missing_modified_session) ? 1 : 0)
plot(na(missing_modified_adjustment) ? 1 : 0)
plot(plain == "NASDAQ:AAPL" ? 1 : 0)
plot(current == "NASDAQ:AAPL" ? 1 : 0)
plot(extended_standard == "NASDAQ:AAPL" ? 1 : 0)
plot(modified_standard == "NASDAQ:AAPL" ? 1 : 0)
plot(adjusted_standard == "NASDAQ:AAPL" ? 1 : 0)
plot(standard_from_modified == "NASDAQ:AAPL" ? 1 : 0)
plot(ha_created == "{\"chart\":\"heikinashi\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(ha_adjusted == "{\"chart\":\"heikinashi\",\"symbol\":\"NASDAQ:AAPL\"}" ? 1 : 0)
plot(ha_standard == "NASDAQ:AAPL" ? 1 : 0)
plot(na(missing_ha) ? 1 : 0)
plot(na(missing) ? 1 : 0)
plot(adjustment.none == "none" and adjustment.splits == "splits" and adjustment.dividends == "dividends" ? 1 : 0)
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
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[17].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[18].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[19].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[20].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[21].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[22].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[23].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[24].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[25].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[26].values, &[1.0, 1.0]);
}

#[test]
fn runs_chart_type_metadata() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("chart type metadata")
plot(chart.is_standard ? 1 : 0)
plot(chart.is_heikinashi ? 1 : 0)
plot(chart.is_kagi ? 1 : 0)
plot(chart.is_linebreak ? 1 : 0)
plot(chart.is_pnf ? 1 : 0)
plot(chart.is_range ? 1 : 0)
plot(chart.is_renko ? 1 : 0)
plot(chart.is_standard and not chart.is_heikinashi and not chart.is_kagi and not chart.is_linebreak and not chart.is_pnf and not chart.is_range and not chart.is_renko ? 1 : 0)
plot(color.r(chart.bg_color))
plot(color.g(chart.bg_color))
plot(color.b(chart.bg_color))
plot(color.t(chart.bg_color))
plot(color.r(chart.fg_color))
plot(color.g(chart.fg_color))
plot(color.b(chart.fg_color))
plot(color.t(chart.fg_color))
plot(chart.left_visible_bar_time)
plot(chart.right_visible_bar_time)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = [
        Bar {
            time: 10_000,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 100.0,
        },
        Bar {
            time: 20_000,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 100.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("result");

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    for plot in &result.plots[1..7] {
        assert_values_close(&plot.values, &[0.0, 0.0]);
    }
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[8].values, &[255.0, 255.0]);
    assert_values_close(&result.plots[9].values, &[255.0, 255.0]);
    assert_values_close(&result.plots[10].values, &[255.0, 255.0]);
    assert_values_close(&result.plots[11].values, &[0.0, 0.0]);
    for plot in &result.plots[12..16] {
        assert_values_close(&plot.values, &[0.0, 0.0]);
    }
    assert_values_close(&result.plots[16].values, &[10_000.0, 10_000.0]);
    assert_values_close(&result.plots[17].values, &[20_000.0, 20_000.0]);
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
