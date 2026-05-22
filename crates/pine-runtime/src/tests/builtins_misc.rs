use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_input_string_condition() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("input string")
mode = input.string("Close", "Mode")
plot(mode == "Close" ? close : open)
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
fn runs_additional_input_variants() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("more inputs")
threshold = input.price(2.5, "Price")
start = input.time(2, "Start")
symbol = input.symbol("AAPL", "Symbol")
timeframe = input.timeframe("D", "Timeframe")
session = input.session("0930-1600", "Session")
notes = input.text_area("Plan", "Notes")
enabled = time >= start and symbol == "AAPL" and timeframe == "D" and session == "0930-1600" and notes == "Plan"
plot(enabled ? math.max(close, threshold) : 0)
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
            time: 1,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        },
        Bar {
            time: 2,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 1.0,
        },
        Bar {
            time: 3,
            open: 3.0,
            high: 3.0,
            low: 3.0,
            close: 3.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[0.0, 2.5, 3.0]);
}

#[test]
fn runs_generic_input_variants() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("generic input")
length = input(2, "Length")
scale = input(1.5, "Scale")
enabled = input(true, "Enabled")
mode = input("SMA", "Mode")
shade = input(color.orange, "Shade")
plot(enabled and mode == "SMA" ? ta.sma(close, length) * scale : open, color=color.new(shade, 10))
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
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.25, 3.75]);
}

#[test]
fn runs_input_metadata_parameters() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("input metadata")
length = input.int(2, "Length", minval=1, maxval=20, step=1, options=[1, 2, 3], tooltip="Bars", inline="row", group="Settings", confirm=true, display=display.all)
scale = input.float(1.5, "Scale", minval=0.5, maxval=5.0, step=0.25, options=[1.0, 1.5], display=display.none)
enabled = input.bool(true, "Enabled", tooltip="Toggle", inline="row", group="Settings", confirm=false)
mode = input.string("SMA", "Mode", options=["SMA", "EMA"], tooltip="Mode")
shade = input.color(color.orange, "Shade", group="Style")
src = input.source(close, "Source", tooltip="Price", inline="src", group="Settings", confirm=true, display=display.all)
plot(enabled and mode == "SMA" ? math.max(src, length) * scale : close, color=shade)
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
    assert_values_close(&result.plots[0].values, &[3.0, 3.0, 4.5]);
}

#[test]
fn runs_color_new_and_named_colors() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("colors")
c = color.new(color.red, 50)
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
gradient = color.from_gradient(close, 1, 3, color.red, color.green)
missing_gradient = color.from_gradient(na, 1, 3, color.red, color.green)
hex = #ff990080
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
hex_channels = color.r(hex) + color.g(hex) + color.b(hex) + color.t(hex)
gradient_channels = color.r(gradient) + color.g(gradient) + color.b(gradient) + color.t(gradient)
bgcolor(custom)
plot(na(c) ? 0 : 1)
plot(opaque == color.new(color.blue, 0) ? 1 : 0)
plot(channels)
plot(hex_channels)
plot(gradient_channels)
plot(na(missing_gradient) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[458.0, 458.0]);
    assert_values_close(&result.plots[3].values, &[458.0, 458.0]);
    assert_values_close(&result.plots[4].values, &[255.0, 192.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_eq!(apply_transparency(0xFF0000, 50), 0xFF000080);
    assert_eq!(
        result.bg_colors[0].values,
        vec![PineValue::Color(0xFF990080), PineValue::Color(0xFF990080)]
    );
}

#[test]
fn runs_string_helpers() {
    let source = SourceFile::new(
        "test.pine",
        r##"indicator("strings")
mode = input.string("sma", "Mode")
upper = str.upper(mode)
lower = str.lower(upper)
length = str.length(upper)
missing = str.length(na)
matched = str.contains(upper, "M") and str.startswith(upper, "S") and str.endswith(upper, "A")
empty_match = str.contains(upper, "") and str.startswith(upper, "") and str.endswith(upper, "")
missing_match = str.contains(na, "S")
mid = str.pos(upper, "M")
missing_pos = str.pos(upper, "Z")
empty_pos = str.pos(upper, "")
na_pos = str.pos(upper, na)
slice = str.substring(upper, mid, mid + 1)
tail = str.substring(upper, mid)
wide = str.substring(upper, 1, 99)
na_begin = str.substring(upper, na, 1)
trimmed = str.trim(" \tSMA\n")
repeated = str.repeat("ab", 2, "-")
empty_repeat = str.repeat("ab", 0)
missing_repeat = str.repeat("ab", na)
replace_first = str.replace("hello", "l", "1")
replace_second = str.replace("hello", "l", "1", 1)
replace_missing = str.replace("hello", "z", "1", 0)
replace_all = str.replace_all("hello", "l", "1")
replace_boundary = str.replace("ab", "", ".", 1)
replace_all_boundaries = str.replace_all("ab", "", ".")
missing_replace = str.replace(na, "x", "y")
number = str.tonumber("1234.50")
signed_number = str.tonumber("-.5")
invalid_number = str.tonumber("$1,234.50")
exponent_number = str.tonumber("1e3")
missing_number = str.tonumber(na)
text_int = str.tostring(42)
text_float = str.tostring(1.25)
text_round0 = str.tostring(1.25, "#")
text_round1 = str.tostring(1.25, "#.#")
text_zeros = str.tostring(1.25, "#.0000")
text_percent = str.tostring(0.1234, format.percent)
text_price = str.tostring(1.234567891, format.price)
text_volume = str.tostring(1234.567, format.volume)
text_bool = str.tostring(true)
text_string = str.tostring("ok")
text_na = str.tostring(na)
values = array.new_float(3)
array.set(values, 0, 1.2)
array.set(values, 1, 2.6)
text_array = str.tostring(values, "#")
formatted = str.format("A={0}, B={1}, A2={0}", text_int, text_float)
formatted_missing = str.format("Missing {2}", text_int)
formatted_number = str.format("Rounded {0,number,#.00} Percent {1,number,percent}", 1.2, 0.0345)
formatted_array = str.format("Values {0}", values)
match_prefix = str.match("NASDAQ:AAPL", "^(?:BATS|NASDAQ|NYSE|AMEX):")
match_suffix = str.match("NASDAQ:AAPL", "AAPL$")
match_missing = str.match("NASDAQ:AAPL", "^NYSE:")
missing_match_regex = str.match(na, ".+")
split_words = str.split("A,B,,C", ",")
split_chars = str.split("xy", "")
split_missing = str.split(na, ",")
formatted_time_default = str.format_time(1609459200000)
formatted_time_date = str.format_time(1609459200000, "yyyy-MM-dd")
formatted_time_text = str.format_time(1609459200000, "HH:mm:ss 'on' MMM dd, yyyy", "UTC")
missing_format_time = str.format_time(na)
plot(upper == "SMA" and lower == "sma" ? length : 0)
plot(na(missing) ? 1 : 0)
plot(matched and empty_match ? 1 : 0)
plot(na(missing_match) ? 1 : 0)
plot(mid + empty_pos + na_pos)
plot(na(missing_pos) ? 1 : 0)
plot(slice == "M" and tail == "MA" and wide == "MA" and na_begin == "S" ? 1 : 0)
plot(trimmed == upper and repeated == "ab-ab" and empty_repeat == "" ? 1 : 0)
plot(na(missing_repeat) ? 1 : 0)
plot(replace_first == "he1lo" and replace_second == "hel1o" and replace_missing == "hello" ? 1 : 0)
plot(replace_all == "he11o" and replace_boundary == "a.b" and replace_all_boundaries == ".a.b." ? 1 : 0)
plot(na(missing_replace) ? 1 : 0)
plot(number == 1234.5 and signed_number == -0.5 ? 1 : 0)
plot(na(invalid_number) and na(exponent_number) and na(missing_number) ? 1 : 0)
plot(text_int == "42" and text_float == "1.25" and text_round0 == "1" and text_round1 == "1.3" ? 1 : 0)
plot(text_zeros == "1.2500" and text_percent == "12.34%" ? 1 : 0)
plot(text_price == "1.23456789" and text_volume == "1234.57" ? 1 : 0)
plot(text_bool == "true" and text_string == "ok" and text_na == "NaN" ? 1 : 0)
plot(text_array == "[1, 3, NaN]" ? 1 : 0)
plot(formatted == "A=42, B=1.25, A2=42" and formatted_missing == "Missing {2}" ? 1 : 0)
plot(formatted_number == "Rounded 1.20 Percent 3.45%" ? 1 : 0)
plot(formatted_array == "Values [1.2, 2.6, NaN]" ? 1 : 0)
plot(match_prefix == "NASDAQ:" and match_suffix == "AAPL" and match_missing == "" ? 1 : 0)
plot(na(missing_match_regex) ? 1 : 0)
plot(split_words.size() == 4 and split_words.get(0) == "A" and split_words.get(2) == "" and split_words.get(3) == "C" ? 1 : 0)
plot(split_chars.size() == 2 and split_chars.get(0) == "x" and split_chars.get(1) == "y" and na(split_missing) ? 1 : 0)
plot(formatted_time_default == "2021-01-01T00:00:00+0000" and formatted_time_date == "2021-01-01" ? 1 : 0)
plot(formatted_time_text == "00:00:00 on Jan 01, 2021" and na(missing_format_time) ? 1 : 0)
"##,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
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
    assert_values_close(&result.plots[27].values, &[1.0, 1.0]);
}

#[test]
fn runs_utc_time_component_variables() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("time components")
plot(year)
plot(month)
plot(weekofyear)
plot(dayofmonth)
plot(dayofweek)
plot(hour)
plot(minute)
plot(second)
ts = 1612235045000
made_ts = timestamp(2021, 2, 2, 3, 4, 5)
date_ts = timestamp(2021, 1, 1)
plot(year(ts))
plot(month(ts, "UTC"))
plot(weekofyear(ts))
plot(dayofmonth(ts))
plot(dayofweek(ts))
plot(hour(ts))
plot(minute(ts))
plot(second(ts))
plot(dayofweek == dayofweek.friday ? 1 : 0)
plot(dayofweek(ts) == dayofweek.tuesday ? 1 : 0)
plot(na(year(na)) and na(weekofyear(na)) and na(dayofweek(na)) ? 1 : 0)
plot(made_ts == ts and date_ts == 1609459200000 ? 1 : 0)
plot(na(timestamp(na, 1, 1)) ? 1 : 0)
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
            time: 1_609_459_200_000,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 100.0,
        },
        Bar {
            time: 1_612_235_045_000,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 100.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[2021.0, 2021.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 2.0]);
    assert_values_close(&result.plots[2].values, &[53.0, 5.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 2.0]);
    assert_values_close(&result.plots[4].values, &[6.0, 3.0]);
    assert_values_close(&result.plots[5].values, &[0.0, 3.0]);
    assert_values_close(&result.plots[6].values, &[0.0, 4.0]);
    assert_values_close(&result.plots[7].values, &[0.0, 5.0]);
    assert_values_close(&result.plots[8].values, &[2021.0, 2021.0]);
    assert_values_close(&result.plots[9].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[10].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[11].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[12].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[13].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[14].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[15].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[16].values, &[1.0, 0.0]);
    assert_values_close(&result.plots[17].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[18].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[19].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[20].values, &[1.0, 1.0]);
}

#[test]
fn runs_timeframe_helpers() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("timeframe helpers")
tf = input.timeframe("60", "TF")
plot(timeframe.period == "1" ? 1 : 0)
plot(timeframe.in_seconds())
plot(timeframe.in_seconds(""))
plot(timeframe.in_seconds("1S"))
plot(timeframe.in_seconds("45S"))
plot(timeframe.in_seconds(tf))
plot(timeframe.in_seconds("D"))
plot(timeframe.in_seconds("2W"))
plot(timeframe.in_seconds("3M"))
plot(na(timeframe.in_seconds(na)) ? 1 : 0)
plot(timeframe.from_seconds(60) == "1" ? 1 : 0)
plot(timeframe.from_seconds(timeframe.in_seconds("45S")) == "45S" ? 1 : 0)
plot(timeframe.from_seconds(timeframe.in_seconds("D")) == "D" ? 1 : 0)
plot(timeframe.from_seconds(timeframe.in_seconds("2W")) == "2W" ? 1 : 0)
plot(timeframe.from_seconds(timeframe.in_seconds("3M")) == "3M" ? 1 : 0)
plot(timeframe.change("1") ? 1 : 0)
plot(timeframe.isminutes and timeframe.isintraday and not timeframe.isseconds and not timeframe.isdaily and not timeframe.isweekly and not timeframe.ismonthly and not timeframe.isdwm ? 1 : 0)
plot(timeframe.multiplier)
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
    assert_values_close(&result.plots[1].values, &[60.0, 60.0]);
    assert_values_close(&result.plots[2].values, &[60.0, 60.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[45.0, 45.0]);
    assert_values_close(&result.plots[5].values, &[3600.0, 3600.0]);
    assert_values_close(&result.plots[6].values, &[86_400.0, 86_400.0]);
    assert_values_close(&result.plots[7].values, &[1_209_600.0, 1_209_600.0]);
    assert_values_close(&result.plots[8].values, &[7_776_000.0, 7_776_000.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 0.0]);
    assert_values_close(&result.plots[16].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[17].values, &[1.0, 1.0]);
}

#[test]
fn runs_timeframe_change() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("timeframe change")
plot(timeframe.change("1") ? 1 : 0)
plot(timeframe.change("D") ? 1 : 0)
plot(na(timeframe.change(na)) ? 1 : 0)
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
            time: 0,
            open: 1.0,
            high: 1.0,
            low: 1.0,
            close: 1.0,
            volume: 1.0,
        },
        Bar {
            time: 30_000,
            open: 2.0,
            high: 2.0,
            low: 2.0,
            close: 2.0,
            volume: 1.0,
        },
        Bar {
            time: 60_000,
            open: 3.0,
            high: 3.0,
            low: 3.0,
            close: 3.0,
            volume: 1.0,
        },
        Bar {
            time: 86_400_000,
            open: 4.0,
            high: 4.0,
            low: 4.0,
            close: 4.0,
            volume: 1.0,
        },
        Bar {
            time: 86_460_000,
            open: 5.0,
            high: 5.0,
            low: 5.0,
            close: 5.0,
            volume: 1.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("result");

    assert_values_close(&result.plots[0].values, &[1.0, 0.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 0.0, 0.0, 1.0, 0.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0]);
}

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
fn rejects_unsupported_calendar_function_timezone() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad calendar timezone")
plot(hour(time, "America/New_York"))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected calendar timezone error");

    assert!(
        error
            .message
            .contains("hour unsupported timezone `America/New_York`"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_unbalanced_str_format_placeholders() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad format")
plot(str.length(str.format("Value {0", close)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.format placeholder error");

    assert!(
        error.message.contains("str.format has unmatched `{`"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_str_match_regex() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad match")
plot(str.length(str.match("abc", "(")))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.match regex error");

    assert!(
        error.message.contains("str.match invalid regex"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_unsupported_str_format_time_timezone() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad time")
plot(str.length(str.format_time(1609459200000, "yyyy-MM-dd", "America/New_York")))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected str.format_time timezone error");

    assert!(
        error
            .message
            .contains("str.format_time unsupported timezone `America/New_York`"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_timestamp_date() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad timestamp")
plot(timestamp(2021, 2, 30))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected invalid timestamp error");

    assert!(
        error
            .message
            .contains("timestamp invalid UTC datetime: 2021-02-30"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_substring_range() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad substring")
plot(str.length(str.substring("SMA", 2, 1)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected substring range error");

    assert!(
        error
            .message
            .contains("str.substring end_pos 1 is less than begin_pos 2"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_invalid_string_repeat_counts() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad repeat")
plot(str.length(str.repeat("x", -1)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected negative repeat error");

    assert!(
        error
            .message
            .contains("str.repeat count cannot be negative: -1"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_string_repeat_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("huge repeat")
plot(str.length(str.repeat("x", 40961)))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected oversized repeat error");

    assert!(
        error
            .message
            .contains("str.repeat result cannot exceed 40960 characters"),
        "{}",
        error.message
    );
}

#[test]
fn runs_selected_math_functions() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
avg_value = math.avg(open, close, high, low)
floor_value = math.floor(close / 2)
ceil_value = math.ceil(close / 2 - 0.25)
trunc_value = math.trunc(close / 2 + 0.75)
const_value = math.floor(2) + math.ceil(1)
sqrt_value = math.sqrt(close)
cbrt_value = math.cbrt(close)
log_value = math.log(close)
log10_value = math.log10(close)
exp_value = math.exp(close)
acos_value = math.acos(close - 2)
asin_value = math.asin(close - 2)
atan_value = math.atan(close)
sign_value = math.sign(close - 2)
degrees_value = math.todegrees(close)
radians_value = math.toradians(close)
constants = math.pi + math.e + math.phi + math.rphi
sin_value = math.sin(close)
cos_value = math.cos(close)
tan_value = math.tan(close)
pow_value = math.pow(close, 2)
hypot_value = math.hypot(close, close + 1)
rounded_precision = math.round(close / 3, 2)
rounded_mintick = math.round_to_mintick(close + 0.006)
mintick = syminfo.mintick
seeded_random = math.random(10, 20, 7)
seeded_random_repeat = math.random(10, 20, 7)
default_random = math.random()
invalid_random = math.random(5, 5, 7)
plot(x)
plot(y)
plot(avg_value)
plot(floor_value + ceil_value)
plot(trunc_value)
plot(const_value)
plot(sqrt_value)
plot(cbrt_value)
plot(log_value)
plot(log10_value)
plot(exp_value)
plot(acos_value)
plot(asin_value)
plot(atan_value)
plot(sign_value)
plot(degrees_value)
plot(radians_value)
plot(constants)
plot(sin_value)
plot(cos_value)
plot(tan_value)
plot(pow_value)
plot(hypot_value)
plot(rounded_precision)
plot(rounded_mintick)
plot(mintick)
plot(seeded_random)
plot(seeded_random_repeat)
plot(default_random)
plot(invalid_random)
plot(math.sqrt(-1))
plot(math.log(0))
plot(math.log10(0))
plot(math.exp(1000))
plot(math.acos(2))
plot(math.asin(2))
plot(math.pow(-1, 0.5))
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

    assert_values_close(&result.plots[0].values, &[2.0, 1.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[2.0, 1.0, 2.0, 2.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 2.0, 3.0, 4.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 2.0, 3.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 2.0, 2.0]);
    assert_values_close(&result.plots[5].values, &[3.0, 3.0, 3.0, 3.0]);
    assert_values_close(
        &result.plots[6].values,
        &[1.0, 2.0_f64.sqrt(), 3.0_f64.sqrt(), 2.0],
    );
    assert_values_close(
        &result.plots[7].values,
        &[1.0, 2.0_f64.cbrt(), 3.0_f64.cbrt(), 4.0_f64.cbrt()],
    );
    assert_values_close(
        &result.plots[8].values,
        &[0.0, 2.0_f64.ln(), 3.0_f64.ln(), 4.0_f64.ln()],
    );
    assert_values_close(
        &result.plots[9].values,
        &[0.0, 2.0_f64.log10(), 3.0_f64.log10(), 4.0_f64.log10()],
    );
    assert_values_close(
        &result.plots[10].values,
        &[1.0_f64.exp(), 2.0_f64.exp(), 3.0_f64.exp(), 4.0_f64.exp()],
    );
    assert_values_close(
        &result.plots[11].values[..3],
        &[(-1.0_f64).acos(), 0.0_f64.acos(), 1.0_f64.acos()],
    );
    assert_eq!(result.plots[11].values[3], PineValue::Na);
    assert_values_close(
        &result.plots[12].values[..3],
        &[(-1.0_f64).asin(), 0.0_f64.asin(), 1.0_f64.asin()],
    );
    assert_eq!(result.plots[12].values[3], PineValue::Na);
    assert_values_close(
        &result.plots[13].values,
        &[
            1.0_f64.atan(),
            2.0_f64.atan(),
            3.0_f64.atan(),
            4.0_f64.atan(),
        ],
    );
    assert_values_close(&result.plots[14].values, &[-1.0, 0.0, 1.0, 1.0]);
    assert_values_close(
        &result.plots[15].values,
        &[
            1.0_f64.to_degrees(),
            2.0_f64.to_degrees(),
            3.0_f64.to_degrees(),
            4.0_f64.to_degrees(),
        ],
    );
    assert_values_close(
        &result.plots[16].values,
        &[
            1.0_f64.to_radians(),
            2.0_f64.to_radians(),
            3.0_f64.to_radians(),
            4.0_f64.to_radians(),
        ],
    );
    assert_values_close(
        &result.plots[17].values,
        &[std::f64::consts::PI
            + std::f64::consts::E
            + 1.618_033_988_749_895
            + 0.618_033_988_749_894_8; 4],
    );
    assert_values_close(
        &result.plots[18].values,
        &[1.0_f64.sin(), 2.0_f64.sin(), 3.0_f64.sin(), 4.0_f64.sin()],
    );
    assert_values_close(
        &result.plots[19].values,
        &[1.0_f64.cos(), 2.0_f64.cos(), 3.0_f64.cos(), 4.0_f64.cos()],
    );
    assert_values_close(
        &result.plots[20].values,
        &[1.0_f64.tan(), 2.0_f64.tan(), 3.0_f64.tan(), 4.0_f64.tan()],
    );
    assert_values_close(&result.plots[21].values, &[1.0, 4.0, 9.0, 16.0]);
    assert_values_close(
        &result.plots[22].values,
        &[5.0_f64.sqrt(), 13.0_f64.sqrt(), 5.0, 41.0_f64.sqrt()],
    );
    assert_values_close(&result.plots[23].values, &[0.33, 0.67, 1.0, 1.33]);
    assert_values_close(&result.plots[24].values, &[1.01, 2.01, 3.01, 4.01]);
    assert_values_close(&result.plots[25].values, &[0.01, 0.01, 0.01, 0.01]);
    for value in &result.plots[26].values {
        let value = value.as_f64().expect("seeded random is numeric");
        assert!((10.0..20.0).contains(&value), "random value {value}");
    }
    assert_eq!(result.plots[26].values, result.plots[27].values);
    for value in &result.plots[28].values {
        let value = value.as_f64().expect("default random is numeric");
        assert!((0.0..1.0).contains(&value), "random value {value}");
    }
    assert_eq!(result.plots[29].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[30].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[31].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[32].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[33].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[34].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[35].values, vec![PineValue::Na; 4]);
    assert_eq!(result.plots[36].values, vec![PineValue::Na; 4]);
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
fn runs_math_sum_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("math sum")
value = math.sum(close, 3)
with_na = math.sum(bar_index == 3 ? na : close, 3)
invalid = math.sum(close, 0)
plot(value)
plot(with_na)
plot(invalid)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[7.0, 14.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..3], &[7.0]);
    assert_eq!(result.plots[1].values[3], PineValue::Na);
    assert_eq!(result.plots[2].values, vec![PineValue::Na; 4]);
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

#[test]
fn rejects_unsupported_timeframe_in_seconds_timeframe() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad timeframe")
plot(timeframe.in_seconds("1H"))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let err = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected timeframe error");
    assert!(
        err.message
            .contains("timeframe.in_seconds unsupported timeframe `1H`"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn rejects_unsupported_timeframe_from_seconds_value() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad timeframe seconds")
plot(timeframe.from_seconds(46) == "" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let err = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected timeframe error");
    assert!(
        err.message
            .contains("timeframe.from_seconds unsupported seconds `46`"),
        "unexpected error: {err:?}"
    );
}

#[test]
fn rejects_unsupported_timeframe_change_timeframe() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bad timeframe change")
plot(timeframe.change("1H") ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let err = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected timeframe error");
    assert!(
        err.message
            .contains("timeframe.change unsupported timeframe `1H`"),
        "unexpected error: {err:?}"
    );
}
