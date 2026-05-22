
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

#[test]
fn runs_sma_plot_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("SMA")
ma = ta.sma(close, 3)
plot(ma)
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
    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Na,
            PineValue::Na,
            PineValue::Float(2.0),
            PineValue::Float(3.0),
        ]
    );
}

#[test]
fn preserves_var_state_across_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("var")
var x = 0
x := x + 1
plot(close + x)
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

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(2.0),
            PineValue::Float(4.0),
            PineValue::Float(6.0),
        ]
    );
}

#[test]
fn runs_ema_plot_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("EMA")
ma = ta.ema(close, 3)
plot(ma)
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

    assert_eq!(
        result.plots[0].values,
        vec![
            PineValue::Float(1.0),
            PineValue::Float(1.5),
            PineValue::Float(2.25),
            PineValue::Float(3.125),
        ]
    );
}

#[test]
fn runs_dema_tema_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("DEMA TEMA")
dema = ta.dema(close, 3)
tema = ta.tema(close, 3)
invalid = ta.dema(close, 0)
plot(dema)
plot(tema)
plot(invalid)
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

    assert_values_close(&result.plots[0].values, &[1.0, 1.75, 2.75, 3.8125]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.875, 2.9375, 4.0]);
    assert_eq!(result.plots[2].values, vec![PineValue::Na; 4]);
}

#[test]
fn runs_rma_plot_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("RMA")
ma = ta.rma(close, 3)
plot(ma)
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

    assert_values_close(
        &result.plots[0].values,
        &[
            1.0,
            1.3333333333333333,
            1.8888888888888888,
            2.5925925925925926,
        ],
    );
}

#[test]
fn runs_rsi_plot_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("RSI")
r = ta.rsi(close, 3)
plot(r)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(2.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[1..],
        &[100.0, 100.0, 66.66666666666666, 83.33333333333333],
    );
}

#[test]
fn collects_hline_and_fill_once() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("fill")
p = plot(close)
h = hline(2)
fill(p, h)
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
    assert_eq!(result.hlines.len(), 1);
    assert_eq!(result.fills.len(), 1);
    assert_eq!(result.hlines[0].price, PineValue::Int(2));
    assert_eq!(result.fills[0].first_id, result.plots[0].id);
    assert_eq!(result.fills[0].second_id, result.hlines[0].id);
}

#[test]
fn runs_output_metadata_parameters() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("output metadata")
p = plot(close, title="Close", color=color.green, linewidth=2, style=plot.style_line, trackprice=false, histbase=0, offset=1, join=false, editable=true, show_last=10, display=display.pane, format=format.price, precision=2, force_overlay=false)
h = hline(2, title="Two", color=color.gray, linestyle=hline.style_dotted, linewidth=1, editable=true, display=display.price_scale)
fill(p, h, color=color.new(color.green, 80), title="Fill", editable=false, show_last=5, fillgaps=true, display=display.status_line)
bgcolor(color.new(color.blue, 90), title="Background", offset=0, editable=false, show_last=3, display=display.data_window)
barcolor(close > open ? color.green : color.red, title="Bars", offset=0, editable=true, show_last=3, display=display.none)
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
    assert_eq!(result.hlines.len(), 1);
    assert_eq!(result.hlines[0].price, PineValue::Int(2));
    assert_eq!(result.fills.len(), 1);
    assert_eq!(result.fills[0].first_id, result.plots[0].id);
    assert_eq!(result.fills[0].second_id, result.hlines[0].id);
    assert_eq!(result.bg_colors.len(), 1);
    assert_eq!(result.bg_colors[0].values.len(), 3);
    assert_eq!(result.bar_colors.len(), 1);
    assert_eq!(result.bar_colors[0].values.len(), 3);
}

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
fn collects_bgcolor_and_barcolor_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("colors")
if close > 1
    bgcolor(color.green)
barcolor(close > 2 ? color.red : na)
plot(close)
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

    assert_eq!(result.bg_colors.len(), 1);
    assert_eq!(
        result.bg_colors[0].values,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
    assert_eq!(result.bar_colors.len(), 1);
    assert_eq!(
        result.bar_colors[0].values,
        vec![PineValue::Na, PineValue::Na, PineValue::Color(0xFF0000)]
    );
}

#[test]
fn collects_plotchar_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("plotchar")
if close > 1
    plotchar(close > 2, title="Marker", char="x", color=color.green, location=location.abovebar, offset=1, text="Up", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all)
plot(close)
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

    assert_eq!(result.plot_chars.len(), 1);
    assert_eq!(
        result.plot_chars[0].values,
        vec![PineValue::Na, PineValue::Bool(false), PineValue::Bool(true)]
    );
    assert_eq!(
        result.plot_chars[0].chars,
        vec![
            PineValue::Na,
            PineValue::String("x".to_owned()),
            PineValue::String("x".to_owned())
        ]
    );
    assert_eq!(
        result.plot_chars[0].colors,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
}

#[test]
fn collects_plotshape_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("plotshape")
if close > 1
    plotshape(close > 2, title="Buy", style=shape.triangleup, location=location.belowbar, color=color.green, offset=1, text="Buy", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all, force_overlay=false)
plot(close)
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

    assert_eq!(result.plot_shapes.len(), 1);
    assert_eq!(
        result.plot_shapes[0].values,
        vec![PineValue::Na, PineValue::Bool(false), PineValue::Bool(true)]
    );
    assert_eq!(
        result.plot_shapes[0].styles,
        vec![
            PineValue::Na,
            PineValue::String("shape.triangleup".to_owned()),
            PineValue::String("shape.triangleup".to_owned())
        ]
    );
    assert_eq!(
        result.plot_shapes[0].locations,
        vec![
            PineValue::Na,
            PineValue::String("location.belowbar".to_owned()),
            PineValue::String("location.belowbar".to_owned())
        ]
    );
    assert_eq!(
        result.plot_shapes[0].colors,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
    assert_eq!(
        result.plot_shapes[0].texts,
        vec![
            PineValue::Na,
            PineValue::String("Buy".to_owned()),
            PineValue::String("Buy".to_owned())
        ]
    );
    assert_eq!(
        result.plot_shapes[0].text_colors,
        vec![
            PineValue::Na,
            PineValue::Color(0xFFFFFF),
            PineValue::Color(0xFFFFFF)
        ]
    );
    assert_eq!(
        result.plot_shapes[0].sizes,
        vec![
            PineValue::Na,
            PineValue::String("size.small".to_owned()),
            PineValue::String("size.small".to_owned())
        ]
    );
}

#[test]
fn collects_plotarrow_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("plotarrow")
if close > 1
    plotarrow(close - 2, title="Momentum", colorup=color.green, colordown=color.red, offset=1, minheight=5, maxheight=20, editable=true, show_last=5, display=display.all, force_overlay=false)
plot(close)
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

    assert_eq!(result.plot_arrows.len(), 1);
    assert_eq!(
        result.plot_arrows[0].values,
        vec![PineValue::Na, PineValue::Float(0.0), PineValue::Float(1.0)]
    );
    assert_eq!(
        result.plot_arrows[0].color_ups,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
    assert_eq!(
        result.plot_arrows[0].color_downs,
        vec![
            PineValue::Na,
            PineValue::Color(0xFF0000),
            PineValue::Color(0xFF0000)
        ]
    );
    assert_eq!(
        result.plot_arrows[0].min_heights,
        vec![PineValue::Na, PineValue::Int(5), PineValue::Int(5)]
    );
    assert_eq!(
        result.plot_arrows[0].max_heights,
        vec![PineValue::Na, PineValue::Int(20), PineValue::Int(20)]
    );
}

#[test]
fn collects_plotbar_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("plotbar")
if close > 1
    plotbar(open, high, low, close, title="Bars", color=color.green, editable=true, show_last=5, display=display.all)
plot(close)
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
        bar_ohlc(2.0, 4.0, 1.0, 3.0),
        bar_ohlc(4.0, 6.0, 3.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plot_bars.len(), 1);
    assert_eq!(
        result.plot_bars[0].opens,
        vec![PineValue::Na, PineValue::Float(2.0), PineValue::Float(4.0)]
    );
    assert_eq!(
        result.plot_bars[0].highs,
        vec![PineValue::Na, PineValue::Float(4.0), PineValue::Float(6.0)]
    );
    assert_eq!(
        result.plot_bars[0].lows,
        vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(3.0)]
    );
    assert_eq!(
        result.plot_bars[0].closes,
        vec![PineValue::Na, PineValue::Float(3.0), PineValue::Float(5.0)]
    );
    assert_eq!(
        result.plot_bars[0].colors,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
}

#[test]
fn collects_plotcandle_series() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("plotcandle")
if close > 1
    plotcandle(open, high, low, close, title="Candles", color=color.green, wickcolor=color.white, editable=true, show_last=5, bordercolor=color.red, display=display.all)
plot(close)
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
        bar_ohlc(2.0, 4.0, 1.0, 3.0),
        bar_ohlc(4.0, 6.0, 3.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plot_candles.len(), 1);
    assert_eq!(
        result.plot_candles[0].opens,
        vec![PineValue::Na, PineValue::Float(2.0), PineValue::Float(4.0)]
    );
    assert_eq!(
        result.plot_candles[0].highs,
        vec![PineValue::Na, PineValue::Float(4.0), PineValue::Float(6.0)]
    );
    assert_eq!(
        result.plot_candles[0].lows,
        vec![PineValue::Na, PineValue::Float(1.0), PineValue::Float(3.0)]
    );
    assert_eq!(
        result.plot_candles[0].closes,
        vec![PineValue::Na, PineValue::Float(3.0), PineValue::Float(5.0)]
    );
    assert_eq!(
        result.plot_candles[0].colors,
        vec![
            PineValue::Na,
            PineValue::Color(0x008000),
            PineValue::Color(0x008000)
        ]
    );
    assert_eq!(
        result.plot_candles[0].wick_colors,
        vec![
            PineValue::Na,
            PineValue::Color(0xFFFFFF),
            PineValue::Color(0xFFFFFF)
        ]
    );
    assert_eq!(
        result.plot_candles[0].border_colors,
        vec![
            PineValue::Na,
            PineValue::Color(0xFF0000),
            PineValue::Color(0xFF0000)
        ]
    );
}

#[test]
fn runs_macd_tuple_assignment() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("MACD")
[macd, signal, hist] = ta.macd(close, 2, 3, 2)
plot(macd)
plot(signal)
plot(hist)
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(
        &result.plots[0].values,
        &[0.0, 0.16666666666666674, 0.30555555555555536],
    );
    assert_values_close(
        &result.plots[1].values,
        &[0.0, 0.11111111111111116, 0.24074074074074063],
    );
    assert_values_close(
        &result.plots[2].values,
        &[0.0, 0.05555555555555558, 0.06481481481481474],
    );
}

#[test]
fn runs_bollinger_bands_tuple_assignment() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("BB")
[basis, upper, lower] = ta.bb(close, 3, 2)
plot(basis)
plot(upper)
plot(lower)
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
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 3.0]);
    assert_values_close(
        &result.plots[1].values[2..],
        &[3.632993161855452, 4.6329931618554525],
    );
    assert_values_close(
        &result.plots[2].values[2..],
        &[0.36700683814454793, 1.367006838144548],
    );
}

#[test]
fn runs_bollinger_band_width_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("BB Width")
width = ta.bbw(close, 3, 2)
zero_basis = ta.bbw(close - close, 3, 2)
invalid = ta.bbw(close, 0, 2)
plot(width)
plot(na(zero_basis) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[1.632993161855452, 1.4966629547095767],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_keltner_channels_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("KC")
[middle, upper, lower] = ta.kc(close, 2, 2)
[middle_plain, upper_plain, lower_plain] = ta.kc(close, 2, 2, false)
[invalid_middle, invalid_upper, invalid_lower] = ta.kc(close, 0, 2)
plot(middle)
plot(upper)
plot(lower)
plot(upper_plain)
plot(lower_plain)
plot(na(invalid_upper) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(12.0, 15.0, 14.0, 12.0),
        bar_ohlc(9.0, 10.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[10.0, 11.333333333333332, 9.777777777777779],
    );
    assert_values_close(
        &result.plots[1].values,
        &[14.0, 19.333333333333332, 17.77777777777778],
    );
    assert_values_close(
        &result.plots[2].values,
        &[6.0, 3.333333333333332, 1.7777777777777786],
    );
    assert_values_close(&result.plots[3].values, &[14.0, 14.0, 13.333333333333334]);
    assert_values_close(
        &result.plots[4].values,
        &[6.0, 8.666666666666666, 6.222222222222223],
    );
    assert_values_close(&result.plots[5].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_keltner_channel_width_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("KCW")
width = ta.kcw(close, 2, 2)
plain_width = ta.kcw(close, 2, 2, false)
zero_basis = ta.kcw(close - close, 2, 2)
invalid = ta.kcw(close, 0, 2)
plot(width)
plot(plain_width)
plot(na(zero_basis) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(12.0, 15.0, 14.0, 12.0),
        bar_ohlc(9.0, 10.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[0.8, 1.411764705882353, 1.6363636363636362],
    );
    assert_values_close(
        &result.plots[1].values,
        &[0.8, 0.4705882352941177, 0.7272727272727272],
    );
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_pivots_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("pivots")
ph = ta.pivothigh(close, 1, 1)
pl = ta.pivotlow(close, 1, 1)
default_ph = ta.pivothigh(1, 1)
default_pl = ta.pivotlow(1, 1)
invalid = ta.pivothigh(close, -1, 1)
plot(ph)
plot(pl)
plot(default_ph)
plot(default_pl)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 5.0, 1.0),
        bar_ohlc(3.0, 12.0, 3.0, 3.0),
        bar_ohlc(2.0, 11.0, 4.0, 2.0),
        bar_ohlc(4.0, 14.0, 2.0, 4.0),
        bar_ohlc(1.0, 10.0, 4.0, 1.0),
        bar_ohlc(0.0, 9.0, 1.0, 0.0),
        bar_ohlc(2.0, 11.0, 3.0, 2.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..3], &[3.0]);
    assert_eq!(result.plots[0].values[3], PineValue::Na);
    assert_values_close(&result.plots[0].values[4..5], &[4.0]);
    assert_eq!(result.plots[0].values[5], PineValue::Na);
    assert_eq!(result.plots[0].values[6], PineValue::Na);

    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(&result.plots[1].values[3..4], &[2.0]);
    assert_eq!(result.plots[1].values[4], PineValue::Na);
    assert_eq!(result.plots[1].values[5], PineValue::Na);
    assert_values_close(&result.plots[1].values[6..], &[0.0]);

    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..3], &[12.0]);
    assert_eq!(result.plots[2].values[3], PineValue::Na);
    assert_values_close(&result.plots[2].values[4..5], &[14.0]);
    assert_eq!(result.plots[2].values[5], PineValue::Na);
    assert_eq!(result.plots[2].values[6], PineValue::Na);

    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..3], &[3.0]);
    assert_eq!(result.plots[3].values[3], PineValue::Na);
    assert_values_close(&result.plots[3].values[4..5], &[2.0]);
    assert_eq!(result.plots[3].values[5], PineValue::Na);
    assert_values_close(&result.plots[3].values[6..], &[1.0]);

    assert_values_close(
        &result.plots[4].values,
        &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    );
}

#[test]
fn runs_pivot_point_levels_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("pivot levels")
levels = ta.pivot_point_levels("Traditional", bar_index == 2)
plot(array.get(levels, 0))
plot(array.get(levels, 1))
plot(array.get(levels, 2))
developing = ta.pivot_point_levels("Traditional", bar_index == 2, true)
plot(array.get(developing, 0))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(10.0, 12.0, 8.0, 11.0, 10.0),
        bar_ohlcv(11.0, 13.0, 9.0, 12.0, 10.0),
        bar_ohlcv(12.0, 14.0, 10.0, 13.0, 10.0),
        bar_ohlcv(13.0, 15.0, 11.0, 14.0, 10.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 4);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[11.0, 11.0]);
    assert_values_close(&result.plots[1].values[2..], &[14.0, 14.0]);
    assert_values_close(&result.plots[2].values[2..], &[9.0, 9.0]);
    assert_values_close(
        &result.plots[3].values,
        &[10.333333333333334, 11.0, 12.333333333333334, 13.0],
    );
}

#[test]
fn calculates_pivot_point_level_formulas() {
    let period = PivotPointPeriod::new(10.0, 13.0, 8.0, 12.0);

    assert_values_close(
        &pivot_point_levels("Traditional", period, 12.0),
        &[11.0, 14.0, 9.0, 16.0, 6.0, 19.0, 4.0, 22.0, 2.0, 25.0, 0.0],
    );

    let fibonacci = pivot_point_levels("Fibonacci", period, 12.0);
    assert_values_close(
        &fibonacci[..7],
        &[11.0, 12.91, 9.09, 14.09, 7.91, 16.0, 6.0],
    );
    assert_eq!(fibonacci[7], PineValue::Na);
    assert_eq!(fibonacci[10], PineValue::Na);

    let woodie = pivot_point_levels("Woodie", period, 12.0);
    assert_values_close(
        &woodie[..9],
        &[11.25, 14.5, 9.5, 16.25, 6.25, 19.5, 4.5, 24.5, -0.5],
    );
    assert_eq!(woodie[9], PineValue::Na);
    assert_eq!(woodie[10], PineValue::Na);

    let classic = pivot_point_levels("Classic", period, 12.0);
    assert_values_close(
        &classic[..9],
        &[11.0, 14.0, 9.0, 16.0, 6.0, 21.0, 1.0, 26.0, -4.0],
    );
    assert_eq!(classic[9], PineValue::Na);
    assert_eq!(classic[10], PineValue::Na);

    let dm = pivot_point_levels("DM", period, 12.0);
    assert_values_close(&dm[..3], &[11.5, 15.0, 10.0]);
    assert_eq!(dm[3], PineValue::Na);
    assert_eq!(dm[10], PineValue::Na);

    assert_values_close(
        &pivot_point_levels("Camarilla", period, 12.0),
        &[
            11.0,
            12.458333333333334,
            11.541666666666666,
            12.916666666666666,
            11.083333333333334,
            13.375,
            10.625,
            14.75,
            9.25,
            19.5,
            4.5,
        ],
    );

    assert_eq!(
        pivot_point_levels("Unknown", period, 12.0),
        pivot_na_levels()
    );
}

#[test]
fn runs_cum_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("cum")
value = ta.cum(close)
index_sum = ta.cum(bar_index)
reset_after_na = ta.cum(bar_index == 2 ? na : close)
plot(value)
plot(index_sum)
plot(reset_after_na)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 3.0, 6.0, 10.0, 15.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 1.0, 3.0, 6.0, 10.0]);
    assert_values_close(&result.plots[2].values[..2], &[1.0, 3.0]);
    assert_eq!(result.plots[2].values[2], PineValue::Na);
    assert_values_close(&result.plots[2].values[3..], &[4.0, 9.0]);
}

#[test]
fn runs_obv_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("obv")
plot(ta.obv)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_volume(1.0, 10.0),
        bar_volume(3.0, 20.0),
        bar_volume(3.0, 30.0),
        bar_volume(2.0, 40.0),
        bar_volume(5.0, 50.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[20.0, 20.0, -20.0, 30.0]);
}

#[test]
fn runs_accdist_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("accdist")
plot(ta.accdist)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
        bar_ohlcv(10.0, 20.0, 10.0, 10.0, 50.0),
        bar_ohlcv(10.0, 10.0, 10.0, 10.0, 30.0),
        bar_ohlcv(20.0, 30.0, 10.0, 25.0, 20.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values[..2], &[40.0, -10.0]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(&result.plots[0].values[3..], &[10.0]);
}

#[test]
fn runs_iii_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("iii")
plot(ta.iii)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
        bar_ohlcv(12.0, 20.0, 10.0, 5.0, 2.0),
        bar_ohlcv(10.0, 10.0, 10.0, 10.0, 10.0),
        bar_ohlcv(10.0, 20.0, 10.0, 15.0, 0.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values[..2], &[0.004, -1.0]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_eq!(result.plots[0].values[3], PineValue::Na);
}

#[test]
fn runs_vwap_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("vwap")
plot(ta.vwap)
plot(ta.vwap(close))
plot(ta.vwap(close, bar_index == 1))
[basis, upper, lower] = ta.vwap(close, false, 2.0)
plot(basis)
plot(upper)
plot(lower)
plot(ta.vwap(bar_index == 2 ? na : close))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(9.0, 12.0, 6.0, 9.0, 10.0),
        bar_ohlcv(18.0, 24.0, 12.0, 18.0, 30.0),
        bar_ohlcv(25.0, 30.0, 15.0, 15.0, 0.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[9.0, 15.75, 15.75]);
    assert_values_close(&result.plots[1].values, &[9.0, 15.75, 15.75]);
    assert_values_close(&result.plots[2].values, &[9.0, 18.0, 18.0]);
    assert_values_close(&result.plots[3].values, &[9.0, 15.75, 15.75]);
    assert_values_close(
        &result.plots[4].values,
        &[9.0, 23.544228634059948, 23.544228634059948],
    );
    assert_values_close(
        &result.plots[5].values,
        &[9.0, 7.955771365940052, 7.955771365940052],
    );
    assert_values_close(&result.plots[6].values[..2], &[9.0, 15.75]);
    assert_eq!(result.plots[6].values[2], PineValue::Na);
}

#[test]
fn runs_nvi_pvi_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("volume index")
plot(ta.nvi)
plot(ta.pvi)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_volume(10.0, 100.0),
        bar_volume(12.0, 90.0),
        bar_volume(6.0, 120.0),
        bar_volume(0.0, 80.0),
        bar_volume(5.0, 60.0),
        bar_volume(10.0, 50.0),
        bar_volume(15.0, 70.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[1.0, 1.2, 1.2, 1.2, 1.2, 2.4, 2.4],
    );
    assert_values_close(
        &result.plots[1].values,
        &[1.0, 1.0, 0.5, 0.5, 0.5, 0.5, 0.75],
    );
}

#[test]
fn runs_pvt_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("pvt")
plot(ta.pvt)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_volume(10.0, 100.0),
        bar_volume(12.0, 50.0),
        bar_volume(6.0, 30.0),
        bar_volume(6.0, 20.0),
        bar_volume(9.0, 10.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[10.0, -5.0, -5.0, 0.0]);
}

#[test]
fn runs_wad_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("wad")
plot(ta.wad)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 10.0, 10.0, 10.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(10.0, 12.0, 8.0, 9.0),
        bar_ohlc(8.0, 10.0, 7.0, 9.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, -1.0, -1.0, 1.0]);
}

#[test]
fn runs_wvad_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("wvad")
plot(ta.wvad)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(10.0, 15.0, 5.0, 12.0, 100.0),
        bar_ohlcv(10.0, 10.0, 10.0, 10.0, 50.0),
        bar_ohlcv(20.0, 25.0, 15.0, 15.0, 40.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values[..1], &[20.0]);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[-20.0]);
}

#[test]
fn runs_true_range_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("TR")
tr = ta.tr()
plot(tr)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(9.0, 10.0, 8.0, 9.0),
        bar_ohlc(11.0, 12.0, 11.0, 11.0),
        bar_ohlc(7.0, 8.0, 6.0, 7.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[2.0, 3.0, 5.0]);
}

#[test]
fn true_range_can_return_na_on_first_bar() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("TR")
tr = ta.tr(false)
plot(tr)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(9.0, 10.0, 8.0, 9.0),
        bar_ohlc(11.0, 12.0, 11.0, 11.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.0]);
}

#[test]
fn runs_true_range_variable_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("TR variable")
plot(ta.tr)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.history.max_constant_offset, 1);

    let bars = vec![
        bar_ohlc(1.0, 2.0, 1.0, 1.5),
        bar_ohlc(2.0, 5.0, 2.0, 4.0),
        bar_ohlc(3.0, 4.0, 1.0, 2.0),
    ];
    let result = run_historical(&hir, &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.5, 3.0]);
}

#[test]
fn runs_atr_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("ATR")
atr = ta.atr(3)
plot(atr)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(9.0, 10.0, 8.0, 9.0),
        bar_ohlc(11.0, 12.0, 11.0, 11.0),
        bar_ohlc(7.0, 8.0, 6.0, 7.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[2.0, 2.3333333333333335, 3.2222222222222223],
    );
}

#[test]
fn runs_supertrend_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("Supertrend")
[line, direction] = ta.supertrend(2, 3)
[bad_line, bad_direction] = ta.supertrend(2, 0)
plot(line)
plot(direction)
plot(na(bad_line) and na(bad_direction) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 14.0, 16.0),
        bar_ohlc(16.0, 14.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[
            14.0,
            14.0,
            14.0,
            8.666666666666668,
            9.944444444444445,
            20.037037037037038,
        ],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, -1.0, -1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_dmi_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("DMI")
[plus, minus, adx] = ta.dmi(3, 2)
[bad_plus, bad_minus, bad_adx] = ta.dmi(3, 0)
plot(plus)
plot(minus)
plot(adx)
plot(na(bad_plus) and na(bad_minus) and na(bad_adx) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 14.0, 16.0),
        bar_ohlc(16.0, 14.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[
            0.0,
            16.666666666666664,
            27.777777777777775,
            51.38888888888888,
            44.88888888888889,
            18.397085610200364,
        ],
    );
    assert_values_close(
        &result.plots[1].values,
        &[0.0, 0.0, 0.0, 0.0, 0.0, 44.26229508196722],
    );
    assert_values_close(
        &result.plots[2].values,
        &[0.0, 50.0, 75.0, 87.5, 93.75, 67.51453488372093],
    );
    assert_values_close(&result.plots[3].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_change_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("change")
c1 = ta.change(close)
c2 = ta.change(close, 2)
index_change = ta.change(bar_index)
flag_change = ta.change(close > open)
plot(c1)
plot(c2)
plot(index_change)
plot(na(flag_change) ? 0 : flag_change ? 1 : -1)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(6.0), bar(10.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 3.0, 4.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[5.0, 7.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_values_close(&result.plots[2].values[1..], &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[0.0, -1.0, -1.0, -1.0]);
}

#[test]
fn runs_mom_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("mom")
value = ta.mom(close, 2)
index_value = ta.mom(bar_index, 2)
plot(value)
plot(index_value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(6.0), bar(10.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[5.0, 7.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[2.0, 2.0]);
}

#[test]
fn runs_roc_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("roc")
value = ta.roc(close, 2)
zero = ta.roc(open, 2)
index_value = ta.roc(bar_index, 2)
plot(value)
plot(zero)
plot(index_value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 1.0, 0.0, 10.0),
        bar_ohlc(1.0, 1.0, 1.0, 15.0),
        bar_ohlc(2.0, 2.0, 2.0, 20.0),
        bar_ohlc(3.0, 3.0, 3.0, 30.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[100.0, 100.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(&result.plots[1].values[3..], &[200.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_eq!(result.plots[2].values[2], PineValue::Na);
    assert_values_close(&result.plots[2].values[3..], &[200.0]);
}

#[test]
fn runs_rising_falling_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("trend")
up = ta.rising(close, 2)
down = ta.falling(close, 2)
index_up = ta.rising(bar_index, 2)
index_down = ta.falling(bar_index, 2)
plot(up ? 1 : 0)
plot(down ? 1 : 0)
plot(index_up ? 1 : 0)
plot(index_down ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar(1.0),
        bar(2.0),
        bar(3.0),
        bar(2.0),
        bar(1.0),
        bar(2.0),
        bar(4.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(
        &result.plots[0].values,
        &[0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0],
    );
    assert_values_close(
        &result.plots[1].values,
        &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    );
    assert_values_close(
        &result.plots[2].values,
        &[0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0],
    );
    assert_values_close(&result.plots[3].values, &[0.0; 7]);
}

#[test]
fn runs_barssince_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barssince")
value = ta.barssince(close > 2)
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(2.0), bar(4.0), bar(1.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[0.0, 1.0, 0.0, 1.0]);
}

#[test]
fn runs_valuewhen_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("valuewhen")
last_close = ta.valuewhen(close > 2, close, 0)
previous_index = ta.valuewhen(close > 2, bar_index, 1)
last_flag = ta.valuewhen(close > 2, close > open, 0)
plot(last_close)
plot(previous_index)
plot(na(last_flag) ? 0 : last_flag ? 1 : -1)
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
        bar_ohlc(2.0, 3.0, 2.0, 3.0),
        bar_ohlc(3.0, 3.0, 2.0, 2.0),
        bar_ohlc(5.0, 5.0, 4.0, 4.0),
        bar_ohlc(1.0, 1.0, 1.0, 1.0),
        bar_ohlc(4.0, 5.0, 4.0, 5.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.0, 3.0, 4.0, 4.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(&result.plots[1].values[3..], &[1.0, 1.0, 3.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 1.0, 1.0, -1.0, -1.0, 1.0]);
}

#[test]
fn runs_cross_functions_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("cross")
baseline = 2.0
crossed = ta.cross(close, baseline)
over = ta.crossover(close, baseline)
under = ta.crossunder(close, baseline)
plot(crossed ? 1 : 0)
plot(over ? 1 : 0)
plot(under ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(1.0), bar(2.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[0.0, 1.0, 1.0, 0.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 1.0, 0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[0.0, 0.0, 1.0, 0.0, 0.0]);
}

#[test]
fn runs_highest_lowest_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("extremes")
hi = ta.highest(close, 3)
lo = ta.lowest(close, 3)
plot(hi)
plot(lo)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[3.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
}

#[test]
fn runs_all_time_extremes_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("all-time extremes")
hi = ta.max(close)
lo = ta.min(open)
held = ta.max(bar_index == 2 ? na : low)
plot(hi)
plot(lo)
plot(held)
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
            open: 3.0,
            high: 5.0,
            low: 5.0,
            close: 1.0,
            volume: 100.0,
        },
        Bar {
            time: 2,
            open: 2.0,
            high: 4.0,
            low: 4.0,
            close: 3.0,
            volume: 100.0,
        },
        Bar {
            time: 3,
            open: 4.0,
            high: 6.0,
            low: 1.0,
            close: 2.0,
            volume: 100.0,
        },
        Bar {
            time: 4,
            open: 1.0,
            high: 7.0,
            low: 6.0,
            close: 5.0,
            volume: 100.0,
        },
        Bar {
            time: 5,
            open: 5.0,
            high: 6.0,
            low: 3.0,
            close: 4.0,
            volume: 100.0,
        },
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 3.0, 3.0, 5.0, 5.0]);
    assert_values_close(&result.plots[1].values, &[3.0, 2.0, 2.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[5.0, 5.0, 5.0, 6.0, 6.0]);
}

#[test]
fn runs_highestbars_lowestbars_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("extreme bars")
hi = ta.highestbars(close, 3)
lo = ta.lowestbars(close, 3)
plot(hi)
plot(lo)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0), bar(5.0), bar(4.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[1.0, 0.0, 0.0, 1.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[2.0, 1.0, 2.0, 0.0]);
}

#[test]
fn runs_single_argument_extremes_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("single argument extremes")
hi = ta.highest(2)
lo = ta.lowest(2)
hi_offset = ta.highestbars(2)
lo_offset = ta.lowestbars(length=2)
plot(hi)
plot(lo)
plot(hi_offset)
plot(lo_offset)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 5.0, 1.0, 1.0),
        bar_ohlc(1.0, 3.0, 0.0, 1.0),
        bar_ohlc(1.0, 4.0, 2.0, 1.0),
        bar_ohlc(1.0, 4.0, -1.0, 1.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[5.0, 4.0, 4.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_values_close(&result.plots[1].values[1..], &[0.0, 0.0, -1.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_values_close(&result.plots[2].values[1..], &[1.0, 0.0, 0.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_values_close(&result.plots[3].values[1..], &[0.0, 1.0, 0.0]);
}

#[test]
fn runs_stdev_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("stdev")
biased = ta.stdev(close, 3)
sample = ta.stdev(close, 3, false)
plot(biased)
plot(sample)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.816496580927726, 1.247219128924647],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 1.5275252316519468]);
}

#[test]
fn runs_variance_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("variance")
biased = ta.variance(close, 3)
sample = ta.variance(close, 3, false)
plot(biased)
plot(sample)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[0.6666666666666666, 1.5555555555555556],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 2.3333333333333335]);
}

#[test]
fn runs_correlation_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("correlation")
same = ta.correlation(close, close, 3)
inverse = ta.correlation(close, -close, 3)
flat = ta.correlation(close, open, 3)
simple = ta.correlation(close, 10, 3)
with_na = ta.correlation(close, bar_index == 3 ? na : high, 3)
plot(same)
plot(inverse)
plot(flat)
plot(simple)
plot(with_na)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(10.0, 1.0, 1.0, 1.0, 1.0),
        bar_ohlcv(10.0, 2.0, 2.0, 2.0, 1.0),
        bar_ohlcv(10.0, 3.0, 3.0, 3.0, 1.0),
        bar_ohlcv(10.0, 5.0, 5.0, 5.0, 1.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[1.0, 1.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[-1.0, -1.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_eq!(result.plots[2].values[2], PineValue::Na);
    assert_eq!(result.plots[2].values[3], PineValue::Na);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_eq!(result.plots[3].values[2], PineValue::Na);
    assert_eq!(result.plots[3].values[3], PineValue::Na);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..3], &[1.0]);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
}

#[test]
fn runs_covariance_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("covariance")
same = ta.covariance(close, close, 3)
inverse = ta.covariance(close, -close, 3)
flat = ta.covariance(close, open, 3)
simple = ta.covariance(close, 10, 3)
with_na = ta.covariance(close, bar_index == 3 ? na : high, 3)
invalid = ta.covariance(close, high, 0)
plot(same)
plot(inverse)
plot(flat)
plot(simple)
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

    let bars = vec![
        bar_ohlcv(10.0, 1.0, 1.0, 1.0, 1.0),
        bar_ohlcv(10.0, 2.0, 2.0, 2.0, 1.0),
        bar_ohlcv(10.0, 3.0, 3.0, 3.0, 1.0),
        bar_ohlcv(10.0, 5.0, 5.0, 5.0, 1.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0 / 3.0, 14.0 / 9.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[-2.0 / 3.0, -14.0 / 9.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[0.0, 0.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..], &[0.0, 0.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..3], &[2.0 / 3.0]);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_eq!(result.plots[5].values[2], PineValue::Na);
    assert_eq!(result.plots[5].values[3], PineValue::Na);
}

#[test]
fn runs_median_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("median")
odd = ta.median(close, 3)
even = ta.median(close, 4)
simple = ta.median(3, 3)
with_na = ta.median(bar_index == 3 ? na : close, 3)
invalid = ta.median(close, 0)
plot(odd)
plot(even)
plot(simple)
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

    let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(&result.plots[1].values[3..], &[3.5]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[3.0, 3.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..3], &[2.0]);
    assert_eq!(result.plots[3].values[3], PineValue::Na);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_eq!(result.plots[4].values[2], PineValue::Na);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
}

#[test]
fn runs_mode_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("mode")
repeated = ta.mode(close, 3)
unique = ta.mode(close + bar_index, 3)
tie = ta.mode(close, 4)
simple = ta.mode(3, 3)
with_na = ta.mode(bar_index == 3 ? na : close, 3)
invalid = ta.mode(close, 0)
plot(repeated)
plot(unique)
plot(tie)
plot(simple)
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

    let bars = vec![bar(1.0), bar(1.0), bar(2.0), bar(2.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[1.0, 2.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_eq!(result.plots[2].values[2], PineValue::Na);
    assert_values_close(&result.plots[2].values[3..], &[1.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..], &[3.0, 3.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..3], &[1.0]);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_eq!(result.plots[5].values[2], PineValue::Na);
    assert_eq!(result.plots[5].values[3], PineValue::Na);
}

#[test]
fn runs_percentile_nearest_rank_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("percentile")
middle = ta.percentile_nearest_rank(close, 3, 50)
lowest = ta.percentile_nearest_rank(close, 3, 0)
highest = ta.percentile_nearest_rank(close, 3, 100)
simple = ta.percentile_nearest_rank(3, 3, 50)
with_na = ta.percentile_nearest_rank(bar_index == 3 ? na : close, 3, 50)
invalid = ta.percentile_nearest_rank(close, 3, 150)
plot(middle)
plot(lowest)
plot(highest)
plot(simple)
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

    let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.0, 2.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[5.0, 8.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..], &[3.0, 3.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..3], &[2.0]);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_eq!(result.plots[5].values[2], PineValue::Na);
    assert_eq!(result.plots[5].values[3], PineValue::Na);
}

#[test]
fn runs_percentile_linear_interpolation_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("linear percentile")
middle = ta.percentile_linear_interpolation(close, 3, 50)
quarter = ta.percentile_linear_interpolation(close, 3, 25)
lowest = ta.percentile_linear_interpolation(close, 3, 0)
highest = ta.percentile_linear_interpolation(close, 3, 100)
simple = ta.percentile_linear_interpolation(3, 3, 50)
with_na = ta.percentile_linear_interpolation(bar_index == 3 ? na : close, 3, 50)
invalid = ta.percentile_linear_interpolation(close, 3, -1)
plot(middle)
plot(quarter)
plot(lowest)
plot(highest)
plot(simple)
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

    let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 5.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[1.5, 3.5]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[1.0, 2.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..], &[5.0, 8.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..], &[3.0, 3.0]);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_values_close(&result.plots[5].values[2..3], &[2.0]);
    assert_eq!(result.plots[5].values[3], PineValue::Na);
    assert_eq!(result.plots[6].values[0], PineValue::Na);
    assert_eq!(result.plots[6].values[1], PineValue::Na);
    assert_eq!(result.plots[6].values[2], PineValue::Na);
    assert_eq!(result.plots[6].values[3], PineValue::Na);
}

#[test]
fn runs_percentrank_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("percentrank")
rank = ta.percentrank(close, 3)
low_rank = ta.percentrank(bar_index == 3 ? 1 : close, 3)
simple = ta.percentrank(3, 3)
with_na = ta.percentrank(bar_index == 3 ? na : close, 3)
invalid = ta.percentrank(close, 0)
plot(rank)
plot(low_rank)
plot(simple)
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

    let bars = vec![bar(1.0), bar(5.0), bar(2.0), bar(8.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[200.0 / 3.0, 100.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(&result.plots[1].values[2..], &[200.0 / 3.0, 100.0 / 3.0]);
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(&result.plots[2].values[2..], &[100.0, 100.0]);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_values_close(&result.plots[3].values[2..3], &[200.0 / 3.0]);
    assert_eq!(result.plots[3].values[3], PineValue::Na);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_eq!(result.plots[4].values[2], PineValue::Na);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
}

#[test]
fn runs_range_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("range")
value = ta.range(close, 3)
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(3.0), bar(2.0), bar(5.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[2.0, 3.0]);
}

#[test]
fn runs_dev_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("dev")
value = ta.dev(close, 3)
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[1.1111111111111112, 1.7777777777777777],
    );
}

#[test]
fn runs_vwma_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("vwma")
value = ta.vwma(close, 3)
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
        bar_volume(1.0, 10.0),
        bar_volume(3.0, 20.0),
        bar_volume(5.0, 30.0),
        bar_volume(7.0, 40.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[3.6666666666666665, 5.444444444444445],
    );
}

#[test]
fn runs_mfi_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("mfi")
value = ta.mfi(close, 3)
flat = ta.mfi(close * 0 + 1, 2)
invalid = ta.mfi(close, 0)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_volume(10.0, 100.0),
        bar_volume(11.0, 200.0),
        bar_volume(12.0, 300.0),
        bar_volume(10.0, 400.0),
        bar_volume(13.0, 500.0),
        bar_volume(12.0, 600.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[
            100.0,
            59.183673469387756,
            71.63120567375887,
            36.72316384180791,
        ],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_tsi_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("tsi")
value = ta.tsi(close, 2, 3)
flat = ta.tsi(close * 0 + 1, 2, 3)
invalid = ta.tsi(close, 0, 3)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar(10.0),
        bar(11.0),
        bar(12.0),
        bar(10.0),
        bar(13.0),
        bar(12.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[1..],
        &[
            1.0,
            1.0,
            4.163336342344337e-17,
            0.42857142857142866,
            0.2085561497326204,
        ],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_cmo_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("cmo")
value = ta.cmo(close, 3)
flat = ta.cmo(close * 0 + 1, 2)
invalid = ta.cmo(close, 0)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar(10.0),
        bar(11.0),
        bar(12.0),
        bar(10.0),
        bar(13.0),
        bar(12.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[3..],
        &[0.0, 33.333333333333336, 0.0],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_cci_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("cci")
value = ta.cci(close, 3)
flat = ta.cci(close * 0 + 1, 2)
invalid = ta.cci(close, 0)
plot(value)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(2.0), bar(1.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[100.0, -50.0, -100.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_cog_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("cog")
value = ta.cog(close, 3)
zero = ta.cog(close * 0, 2)
invalid = ta.cog(close, 0)
plot(value)
plot(na(zero) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
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

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[-1.6666666666666667, -1.7777777777777777],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_wma_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("wma")
value = ta.wma(close, 3)
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[2.8333333333333335, 5.166666666666667],
    );
}

#[test]
fn runs_hma_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("hma")
value = ta.hma(close, 4)
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(7.0), bar(11.0), bar(16.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_eq!(result.plots[0].values[3], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[4..],
        &[10.38888888888889, 15.38888888888889],
    );
}

#[test]
fn runs_swma_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("swma")
value = ta.swma(close)
with_na = ta.swma(bar_index == 4 ? na : close)
plot(value)
plot(with_na)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0), bar(16.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(&result.plots[0].values[3..], &[3.5, 7.0]);
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(&result.plots[1].values[3..4], &[3.5]);
    assert_eq!(result.plots[1].values[4], PineValue::Na);
}

#[test]
fn runs_alma_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("alma")
value = ta.alma(close, 4, 0.85, 6)
floored = ta.alma(close, 4, 0.85, 6, true)
with_na = ta.alma(bar_index == 4 ? na : close, 4, 0.85, 6)
invalid = ta.alma(close, 4, 0.85, 0)
plot(value)
plot(floored)
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

    let bars = vec![bar(1.0), bar(2.0), bar(4.0), bar(8.0), bar(16.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[3..],
        &[5.935295490253145, 11.87059098050629],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_eq!(result.plots[1].values[2], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[3..],
        &[4.370978545474149, 8.741957090948299],
    );
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_eq!(result.plots[2].values[2], PineValue::Na);
    assert_values_close(&result.plots[2].values[3..4], &[5.935295490253145]);
    assert_eq!(result.plots[2].values[4], PineValue::Na);
    assert_eq!(result.plots[3].values[0], PineValue::Na);
    assert_eq!(result.plots[3].values[1], PineValue::Na);
    assert_eq!(result.plots[3].values[2], PineValue::Na);
    assert_eq!(result.plots[3].values[3], PineValue::Na);
    assert_eq!(result.plots[3].values[4], PineValue::Na);
}

#[test]
fn runs_linreg_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("linreg")
current = ta.linreg(close, 3, 0)
previous = ta.linreg(close, 3, 1)
projected = ta.linreg(close, 3, -1)
single = ta.linreg(close, 1, 0)
with_na = ta.linreg(bar_index == 3 ? na : close, 3, 0)
invalid = ta.linreg(close, 0, 0)
plot(current)
plot(previous)
plot(projected)
plot(single)
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
    assert_values_close(
        &result.plots[0].values[2..],
        &[3.8333333333333335, 7.666666666666667],
    );
    assert_eq!(result.plots[1].values[0], PineValue::Na);
    assert_eq!(result.plots[1].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[1].values[2..],
        &[2.3333333333333335, 4.666666666666667],
    );
    assert_eq!(result.plots[2].values[0], PineValue::Na);
    assert_eq!(result.plots[2].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[2].values[2..],
        &[5.333333333333334, 10.666666666666668],
    );
    assert_values_close(&result.plots[3].values, &[1.0, 2.0, 4.0, 8.0]);
    assert_eq!(result.plots[4].values[0], PineValue::Na);
    assert_eq!(result.plots[4].values[1], PineValue::Na);
    assert_values_close(&result.plots[4].values[2..3], &[3.8333333333333335]);
    assert_eq!(result.plots[4].values[3], PineValue::Na);
    assert_eq!(result.plots[5].values[0], PineValue::Na);
    assert_eq!(result.plots[5].values[1], PineValue::Na);
    assert_eq!(result.plots[5].values[2], PineValue::Na);
    assert_eq!(result.plots[5].values[3], PineValue::Na);
}

#[test]
fn runs_stoch_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("stoch")
k = ta.stoch(close, high, low, 3)
flat = ta.stoch(close, 1 + close * 0, 1 + close * 0, 2)
invalid = ta.stoch(close, high, low, 0)
plot(k)
plot(na(flat) ? 1 : 0)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 14.0, 16.0),
        bar_ohlc(16.0, 14.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[
            75.0,
            83.33333333333333,
            83.33333333333333,
            11.11111111111111,
        ],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);
}

#[test]
fn runs_wpr_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("wpr")
value = ta.wpr(3)
invalid = ta.wpr(0)
plot(value)
plot(na(invalid) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 14.0, 16.0),
        bar_ohlc(16.0, 14.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[2..],
        &[
            -25.0,
            -16.666666666666668,
            -16.666666666666668,
            -88.88888888888889,
        ],
    );
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0, 1.0, 1.0, 1.0]);

    let source = SourceFile::new(
        "test.pine",
        r#"indicator("flat wpr")
plot(na(ta.wpr(2)) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 1.0, 1.0, 1.0), bar_ohlc(1.0, 1.0, 1.0, 1.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
}

#[test]
fn runs_ao_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("ao")
value = ta.ao()
plot(value)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars: Vec<_> = (1..=40).map(|value| bar(value as f64)).collect();
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    for value in &result.plots[0].values[..33] {
        assert_eq!(*value, PineValue::Na);
    }
    assert_values_close(
        &result.plots[0].values[33..],
        &[14.5, 14.5, 14.5, 14.5, 14.5, 14.5, 14.5],
    );
}

#[test]
fn runs_bop_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bop")
value = ta.bop()
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
        bar_ohlc(10.0, 12.0, 8.0, 11.0),
        bar_ohlc(10.0, 13.0, 9.0, 9.0),
        bar_ohlc(10.0, 10.0, 10.0, 10.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values[..2], &[0.25, -0.25]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
}

#[test]
fn runs_sar_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("sar")
sar = ta.sar(0.02, 0.02, 0.2)
plot(sar)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(10.0, 11.0, 9.0, 10.0),
        bar_ohlc(10.0, 12.0, 10.0, 11.0),
        bar_ohlc(11.0, 13.0, 11.0, 12.0),
        bar_ohlc(12.0, 16.0, 12.0, 15.0),
        bar_ohlc(15.0, 17.0, 14.0, 16.0),
        bar_ohlc(16.0, 14.0, 8.0, 9.0),
        bar_ohlc(9.0, 10.0, 6.0, 7.0),
        bar_ohlc(7.0, 8.0, 4.0, 5.0),
        bar_ohlc(5.0, 7.0, 3.0, 6.0),
        bar_ohlc(6.0, 12.0, 5.0, 11.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(
        &result.plots[0].values[1..],
        &[
            9.0, 9.0, 9.16, 9.5704, 17.0, 17.0, 16.56, 15.8064, 14.781888,
        ],
    );
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
fn realtime_rollback_restores_math_random_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime random")
plot(math.random(0, 1, 7))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    let forming_value = forming.plots[0].values[1].clone();

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_eq!(rolled_back.plots[0].values[1], forming_value);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_eq!(confirmed.plots[0].values[1], forming_value);
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
fn profiles_runtime_storage() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("profile")
ma = ta.sma(close, 2)
plot(ma)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.profile.bars, 3);
    assert_eq!(profiled.profile.series_buffers, 0);
    assert_eq!(profiled.profile.series_values, 0);
    assert!(profiled.profile.series_capacity >= profiled.profile.series_values);
    assert_eq!(profiled.profile.max_series_depth, 0);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::StaticTrimmed
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 0);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(!profiled.profile.history_has_dynamic_offsets);
    assert_eq!(profiled.profile.rolling_window_slots, 1);
    assert_eq!(profiled.profile.rolling_window_values, 2);
    assert!(
        profiled.profile.rolling_window_value_capacity >= profiled.profile.rolling_window_values
    );
    assert_eq!(profiled.profile.plots, 1);
    assert_eq!(profiled.profile.plot_values, 3);
    assert!(profiled.profile.plot_capacity >= profiled.profile.plot_values);
    assert_eq!(profiled.profile.plot_shapes, 0);
    assert_eq!(profiled.profile.plot_arrows, 0);
    assert_eq!(profiled.profile.plot_bars, 0);
    assert_eq!(profiled.profile.plot_candles, 0);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[1..], &[1.5, 2.5]);
}

#[test]
fn trims_constant_history_to_required_depth() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("static history")
plot(close[2])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_eq!(profiled.result.plots[0].values[1], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[2..], &[1.0, 2.0]);
    assert_eq!(profiled.profile.max_series_depth, 2);
    assert_eq!(profiled.profile.series_values, 2);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::StaticTrimmed
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 2);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(!profiled.profile.history_has_dynamic_offsets);
}

#[test]
fn keeps_full_history_when_dynamic_offsets_exist() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("dynamic history retention")
length = input.int(1, "Length")
plot(close[length])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Na);
    assert_values_close(&profiled.result.plots[0].values[1..], &[1.0, 2.0, 3.0]);
    assert_eq!(profiled.profile.max_series_depth, 4);
    assert!(profiled.profile.series_values >= 4);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::DynamicFull
    );
    assert_eq!(profiled.profile.history_max_constant_offset, 0);
    assert_eq!(profiled.profile.history_max_bars_back, None);
    assert!(profiled.profile.history_has_dynamic_offsets);
}

#[test]
fn max_bars_back_bounds_dynamic_history_retention() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("dynamic history retention", max_bars_back=2)
offset = bar_index == 0 ? 0 : 3
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_eq!(profiled.result.plots[0].values[0], PineValue::Float(1.0));
    assert_eq!(profiled.result.plots[0].values[1..], vec![PineValue::Na; 3]);
    assert_eq!(profiled.profile.max_series_depth, 2);
    assert_eq!(
        profiled.profile.history_retention_mode,
        HistoryRetentionMode::MaxBarsBack
    );
    assert_eq!(profiled.profile.history_max_bars_back, Some(2));
    assert!(profiled.profile.history_has_dynamic_offsets);
}

#[test]
fn append_bar_matches_full_historical_run() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("incremental")
ma = ta.sma(close, 3)
e = ta.ema(close, 2)
plot(ma)
plot(e)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];

    let full = run_historical(&hir, &bars).expect("full result");
    let mut runtime = HistoricalRuntime::new(&hir);
    for (index, bar) in bars.iter().copied().enumerate() {
        runtime.append_bar(bar).expect("append result");
        assert_eq!(runtime.profile().bars, index + 1);
    }
    let incremental = runtime.result();

    assert_eq!(incremental, full);
}

#[test]
fn bar_update_model_marks_committing_updates() {
    let bar = bar(1.0);

    assert!(BarUpdate::historical(bar).commits_series());
    assert!(BarUpdate::confirmed(bar).commits_series());
    assert!(!BarUpdate::forming(bar).commits_series());
}

#[test]
fn runs_barstate_isfirst_over_historical_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate")
plot(barstate.isfirst ? 1 : 0)
plot(barstate.islast ? 1 : 0)
plot(barstate.isnew ? 1 : 0)
plot(barstate.isconfirmed ? 1 : 0)
plot(barstate.ishistory ? 1 : 0)
plot(barstate.isrealtime ? 1 : 0)
plot(session.ismarket ? 1 : 0)
plot(session.ispremarket ? 1 : 0)
plot(session.ispostmarket ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0), bar(3.0)])
        .expect("runtime result");

    assert_values_close(&result.plots[0].values, &[1.0, 0.0, 0.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 0.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[8].values, &[0.0, 0.0, 0.0]);
}

#[test]
fn append_bar_treats_current_open_ended_historical_bar_as_last() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate append")
plot(barstate.islast ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let hir = analysis.hir.expect("HIR");
    let mut runtime = HistoricalRuntime::new(&hir);
    runtime.append_bar(bar(1.0)).expect("first append");
    runtime.append_bar(bar(2.0)).expect("second append");
    let result = runtime.result();

    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
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
fn barstate_realtime_flags_track_update_kind() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("barstate realtime")
plot(barstate.isconfirmed ? close : 0)
plot(barstate.ishistory ? close : 0)
plot(barstate.isrealtime ? close : 0)
plot(barstate.islast ? close : 0)
plot(barstate.isnew ? close : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    let confirmed = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");
    assert_values_close(&confirmed.plots[0].values, &[1.0]);
    assert_values_close(&confirmed.plots[1].values, &[1.0]);
    assert_values_close(&confirmed.plots[2].values, &[0.0]);
    assert_values_close(&confirmed.plots[3].values, &[1.0]);
    assert_values_close(&confirmed.plots[4].values, &[1.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[2].values, &[0.0, 2.0]);
    assert_values_close(&forming.plots[3].values, &[1.0, 2.0]);
    assert_values_close(&forming.plots[4].values, &[1.0, 2.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(4.0)))
        .expect("second forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&forming.plots[2].values, &[0.0, 4.0]);
    assert_values_close(&forming.plots[3].values, &[1.0, 4.0]);
    assert_values_close(&forming.plots[4].values, &[1.0, 0.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(3.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 3.0]);
    assert_values_close(&confirmed.plots[1].values, &[1.0, 0.0]);
    assert_values_close(&confirmed.plots[2].values, &[0.0, 3.0]);
    assert_values_close(&confirmed.plots[3].values, &[1.0, 3.0]);
    assert_values_close(&confirmed.plots[4].values, &[1.0, 0.0]);
}

#[test]
fn realtime_forming_updates_roll_back_previous_forming_output() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    let first = runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");
    assert_values_close(&first.plots[0].values, &[1.0]);

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);
    assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_values_close(&rolled_back.plots[0].values, &[1.0, 3.0]);
    assert_values_close(&runtime.confirmed_result().plots[0].values, &[1.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 4.0]);
    assert_eq!(runtime.profile().bars, 2);
    assert_eq!(runtime.confirmed_profile().bars, 2);
}

#[test]
fn realtime_rollback_restores_var_state() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("realtime var")
var x = 0
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
    let hir = analysis.hir.expect("HIR");
    let mut runtime = RealtimeRuntime::new(&hir);

    runtime
        .update(BarUpdate::historical(bar(1.0)))
        .expect("historical update");

    let forming = runtime
        .update(BarUpdate::forming(bar(2.0)))
        .expect("forming update");
    assert_values_close(&forming.plots[0].values, &[1.0, 2.0]);

    let rolled_back = runtime
        .update(BarUpdate::forming(bar(3.0)))
        .expect("second forming update");
    assert_values_close(&rolled_back.plots[0].values, &[1.0, 2.0]);

    let confirmed = runtime
        .update(BarUpdate::confirmed(bar(4.0)))
        .expect("confirmed update");
    assert_values_close(&confirmed.plots[0].values, &[1.0, 2.0]);
}

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
fn advances_conditional_sma_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional sma")
ma = close
if close > open
    ma := ta.sma(close, 2)
plot(ma)
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
    assert_values_close(&result.plots[0].values[1..], &[2.0, 4.0, 7.0]);
}

#[test]
fn advances_conditional_ema_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional ema")
e = close
if close > open
    e := ta.ema(close, 2)
plot(e)
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
    assert_values_close(
        &result.plots[0].values,
        &[2.0, 2.0, 4.666666666666667, 6.888888888888889],
    );
}

#[test]
fn advances_conditional_dema_tema_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional dema tema")
d = close
t = close
if close > open
    d := ta.dema(close, 2)
    t := ta.tema(close, 2)
plot(d)
plot(t)
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

    assert_values_close(
        &result.plots[0].values,
        &[2.0, 2.0, 5.555555555555555, 7.925925925925926],
    );
    assert_values_close(
        &result.plots[1].values,
        &[2.0, 2.0, 5.851851851851852, 8.074074074074074],
    );
}

#[test]
fn advances_conditional_vwap_anchor_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional anchored vwap")
score = close
if close > open
    score := ta.vwap(close, bar_index == 2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(0.0, 10.0, 10.0, 10.0, 1.0),
        bar_ohlcv(30.0, 20.0, 20.0, 20.0, 100.0),
        bar_ohlcv(0.0, 30.0, 30.0, 30.0, 1.0),
        bar_ohlcv(0.0, 40.0, 40.0, 40.0, 1.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[10.0, 20.0, 30.0, 35.0]);
}

#[test]
fn pads_conditional_plot_with_na_when_branch_is_skipped() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional plot")
if close > open
    plot(close)
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
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(
        result.plots[0].values,
        vec![PineValue::Float(2.0), PineValue::Na, PineValue::Float(6.0)]
    );
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
fn advances_conditional_kc_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional kc")
score = close
if close > open
    [middle, upper, lower] = ta.kc(close, 2, 2)
    score := upper
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 11.0, 9.0, 10.0),
        bar_ohlc(13.0, 15.0, 14.0, 12.0),
        bar_ohlc(0.0, 10.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[14.0, 12.0, 16.0]);
}

#[test]
fn advances_conditional_kcw_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional kcw")
score = close
if close > open
    score := ta.kcw(close, 2, 2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 11.0, 9.0, 10.0),
        bar_ohlc(13.0, 15.0, 14.0, 12.0),
        bar_ohlc(0.0, 10.0, 8.0, 9.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[0.8, 12.0, 1.4285714285714286]);
}

#[test]
fn advances_conditional_pivot_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional pivot")
score = close
if close > open
    score := ta.pivothigh(close, 1, 1)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 1.0, 1.0, 1.0),
        bar_ohlc(5.0, 2.0, 2.0, 2.0),
        bar_ohlc(0.0, 3.0, 3.0, 3.0),
        bar_ohlc(0.0, 2.0, 2.0, 2.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..2], &[2.0]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(&result.plots[0].values[3..], &[3.0]);
}

#[test]
fn advances_conditional_rsi_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional rsi")
r = close
if close > open
    r := ta.rsi(close, 2)
plot(r)
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
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
}

#[test]
fn advances_conditional_atr_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional atr")
a = close
if close > open
    a := ta.atr(2)
plot(a)
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
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 2.5]);
}

#[test]
fn advances_conditional_dmi_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional dmi")
score = close
if close > open
    [plus, minus, adx] = ta.dmi(3, 2)
    score := plus + minus + adx
plot(score)
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
    assert_values_close(
        &result.plots[0].values,
        &[0.0, 2.0, 100.0, 132.14285714285714],
    );
}

#[test]
fn advances_conditional_stoch_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional stoch")
score = close
if close > open
    score := ta.stoch(close, high, low, 2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 0.0, 5.0),
        bar_ohlc(3.0, 100.0, 100.0, 2.0),
        bar_ohlc(4.0, 20.0, 10.0, 15.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 75.0]);
}

#[test]
fn advances_conditional_wpr_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional wpr")
score = close
if close > open
    score := ta.wpr(2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 0.0, 5.0),
        bar_ohlc(3.0, 100.0, 100.0, 2.0),
        bar_ohlc(4.0, 20.0, 10.0, 15.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, -25.0]);
}

#[test]
fn advances_conditional_ao_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional ao")
score = close
if close > open
    score := ta.ao()
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars: Vec<_> = (1..=35)
        .map(|value| {
            if value == 11 {
                bar_ohlc(2.0, 2.0, 2.0, 1.0)
            } else {
                let value = value as f64;
                bar_ohlc(0.0, value, value, value)
            }
        })
        .collect();
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    for value in &result.plots[0].values[..10] {
        assert_eq!(*value, PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[10..11], &[1.0]);
    for value in &result.plots[0].values[11..34] {
        assert_eq!(*value, PineValue::Na);
    }
    assert_values_close(&result.plots[0].values[34..], &[14.794117647058822]);
}

#[test]
fn advances_conditional_sar_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional sar")
score = close
if close > open
    score := ta.sar(0.02, 0.02, 0.2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 1.0, 5.0),
        bar_ohlc(3.0, 4.0, 1.0, 2.0),
        bar_ohlc(4.0, 20.0, 10.0, 15.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 1.0]);
}

#[test]
fn advances_conditional_mfi_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional mfi")
score = close
if close > open
    score := ta.mfi(close, 2)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlcv(1.0, 10.0, 1.0, 5.0, 10.0),
        bar_ohlcv(3.0, 4.0, 1.0, 2.0, 10.0),
        bar_ohlcv(4.0, 20.0, 10.0, 15.0, 10.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
}

#[test]
fn advances_conditional_tsi_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional tsi")
score = close
if close > open
    score := ta.tsi(close, 2, 3)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 1.0, 5.0),
        bar_ohlc(3.0, 4.0, 1.0, 2.0),
        bar_ohlc(4.0, 20.0, 10.0, 15.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 1.0]);
}

#[test]
fn advances_conditional_cmo_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional cmo")
score = close
if close > open
    score := ta.cmo(close, 1)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(1.0, 10.0, 1.0, 5.0),
        bar_ohlc(3.0, 4.0, 1.0, 2.0),
        bar_ohlc(4.0, 20.0, 10.0, 15.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[2.0, 100.0]);
}

#[test]
fn advances_conditional_cci_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional cci")
score = close
if close > open
    score := ta.cci(close, 3)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 1.0, 1.0, 1.0),
        bar_ohlc(3.0, 2.0, 2.0, 2.0),
        bar_ohlc(0.0, 3.0, 3.0, 3.0),
        bar_ohlc(0.0, 4.0, 4.0, 4.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..2], &[2.0]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(&result.plots[0].values[3..], &[80.0]);
}

#[test]
fn advances_conditional_cog_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional cog")
score = close
if close > open
    score := ta.cog(close, 3)
plot(score)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![
        bar_ohlc(0.0, 1.0, 1.0, 1.0),
        bar_ohlc(3.0, 2.0, 2.0, 2.0),
        bar_ohlc(0.0, 3.0, 3.0, 3.0),
        bar_ohlc(0.0, 4.0, 4.0, 4.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..2], &[2.0]);
    assert_eq!(result.plots[0].values[2], PineValue::Na);
    assert_values_close(&result.plots[0].values[3..], &[-1.625]);
}

#[test]
fn advances_conditional_macd_only_when_branch_executes() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional macd")
if close > open
    [macd, signal, hist] = ta.macd(close, 2, 3, 2)
    plot(macd)
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
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(
        &[
            result.plots[0].values[0].clone(),
            result.plots[0].values[2].clone(),
            result.plots[0].values[3].clone(),
        ],
        &[0.0, 0.666666666666667, 0.8888888888888893],
    );
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
fn runs_function_with_ta_call() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf sma")
smooth(src, len) => ta.sma(src, len)
plot(smooth(close, 2))
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
    assert_values_close(&result.plots[0].values[1..], &[1.5, 2.5, 3.5]);
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
fn runs_block_body_function_with_ta_call() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("udf block ta")
smooth2(src, len) =>
    ma = ta.sma(src, len)
    ma * 2
plot(smooth2(close, 2))
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
fn runs_float_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array ops")
values = array.new_float(2, close)
array.push(values, high)
array.set(values, 0, low)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_float_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array methods")
values = array.new_float(2, close)
values.push(high)
values.set(0, low)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[4.0, 6.0, 8.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_int_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array ops")
values = array.new_int(2, bar_index)
array.push(values, 10)
array.set(values, 0, 3)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first + last + array.size(values))
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_int_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array methods")
values = array.new_int(2, bar_index)
values.push(10)
values.set(0, 3)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first + last + values.size())
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[15.0, 15.0, 15.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_bool_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array ops")
values = array.new_bool(2, close > open)
array.push(values, true)
array.set(values, 0, false)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot((first or last) ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_bool_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array methods")
values = array.new_bool(2, close > open)
values.push(true)
values.set(0, false)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot((first or last) ? values.size() : 0)
plot(na(missing) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_string_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array ops")
values = array.new_string(2, "seed")
array.push(values, "tail")
array.set(values, 0, "head")
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
text = str.tostring(values)
plot(first == "head" and last == "tail" ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(text == "[head, seed]" ? 1 : 0)
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_string_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array methods")
values = array.new_string(2, "seed")
values.push("tail")
values.set(0, "head")
first = values.get(0)
last = values.pop()
missing = values.get(10)
text = str.format("Values {0}", values)
plot(first == "head" and last == "tail" ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(text == "Values [head, seed]" ? 1 : 0)
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_color_array_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array ops")
values = array.new_color(2, color.red)
array.push(values, color.green)
array.set(values, 0, color.blue)
first = array.get(values, 0)
last = array.pop(values)
missing = array.get(values, 10)
plot(first == color.blue and last == color.green ? array.size(values) : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
}

#[test]
fn runs_color_array_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array methods")
values = array.new_color(2, color.red)
values.push(color.green)
values.set(0, color.blue)
first = values.get(0)
last = values.pop()
missing = values.get(10)
plot(first == color.blue and last == color.green ? values.size() : 0)
plot(na(missing) ? 1 : 0)
plot(color.b(first) + color.g(last))
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[383.0, 383.0, 383.0]);
}

#[test]
fn runs_array_helper_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array helpers")
values = array.new_int()
array.unshift(values, 2)
array.unshift(values, 1)
first = array.first(values)
last = array.last(values)
shifted = array.shift(values)
empty = array.new_string()
plot(first + last + shifted + array.size(values))
plot(na(array.first(empty)) and na(array.last(empty)) and na(array.shift(empty)) ? 1 : 0)
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
    assert_values_close(&result.plots[0].values, &[5.0, 5.0, 5.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_helper_method_calls() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array helper methods")
values = array.new_string()
values.unshift("tail")
values.unshift("head")
first = values.first()
last = values.last()
shifted = values.shift()
colors = array.new_color()
colors.unshift(color.green)
colors.unshift(color.red)
color_first = colors.first()
color_last = colors.last()
color_shifted = colors.shift()
plot(first == "head" and last == "tail" and shifted == "head" ? values.size() : 0)
plot(color_first == color.red and color_last == color.green and color_shifted == color.red ? colors.size() : 0)
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
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_array_insert_remove_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array insert remove")
ints = array.new_int()
ints.push(1)
ints.push(3)
array.insert(ints, 1, 2)
removed = ints.remove(0)
plot(removed)
plot(ints.get(0) * 10 + ints.get(1))

words = array.new_string()
words.push("a")
words.push("c")
words.insert(1, "b")
word_removed = array.remove(words, 2)
plot(word_removed == "c" and words.join("|") == "a|b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.insert(1, color.green)
color_removed = colors.remove(0)
plot(color_removed == color.red and colors.get(0) == color.green ? 1 : 0)

flags = array.new_bool()
flags.insert(0, true)
plot(flags.remove(0) ? flags.size() : 99)

plot(na(array.remove(flags, 0)) ? 1 : 0)
array.insert(flags, 3, false)
plot(flags.size())

negative = array.from(10, 20, 30)
plot(negative.get(-1) + negative.get(-3))
negative.set(-2, 99)
plot(negative.get(1))
negative.insert(-1, 25)
plot(negative.get(2) * 100 + negative.get(-1))
negative_head = negative.remove(-4)
negative_tail = negative.remove(-1)
plot(negative_head + negative_tail + negative.size())
plot(na(negative.get(-3)) and na(negative.remove(-3)) ? 1 : 0)
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

    assert_eq!(result.plots.len(), 12);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[7].values, &[40.0, 40.0]);
    assert_values_close(&result.plots[8].values, &[99.0, 99.0]);
    assert_values_close(&result.plots[9].values, &[2530.0, 2530.0]);
    assert_values_close(&result.plots[10].values, &[42.0, 42.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_fill_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array fill")
ints = array.new_int(4, 1)
array.fill(ints, 9, 1, 3)
plot(ints.get(0) * 1000 + ints.get(1) * 100 + ints.get(2) * 10 + ints.get(3))
ints.fill(2)
plot(ints.get(0) + ints.get(3))

floats = array.new_float(3, close)
floats.fill(high, 0, 2)
plot(floats.get(0) + floats.get(1) + floats.get(2))

words = array.new_string(3, "a")
words.fill("b", 1, 3)
plot(words.join("|") == "a|b|b" ? 1 : 0)

colors = array.new_color(2, color.red)
colors.fill(color.green)
plot(colors.get(0) == color.green and colors.get(1) == color.green ? 1 : 0)

flags = array.new_bool(2, false)
array.fill(flags, true, 0, 1)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

array.fill(flags, false, -1, 1)
array.fill(flags, false, 0, 3)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 7);
    assert_values_close(&result.plots[0].values, &[1991.0, 1991.0]);
    assert_values_close(&result.plots[1].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[2].values, &[10.0, 15.0]);
    assert_values_close(&result.plots[3].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_from_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array from")
ints = array.from(1, 2, 3)
plot(ints.size())
plot(ints.sum())
ints.push(4)
plot(ints.last())

floats = array.from(1, close, na)
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)

flags = array.from(true, false)
plot(flags.get(0) and not flags.get(1) ? 1 : 0)

words = array.from("a", "b")
plot(words.join("|") == "a|b" ? 1 : 0)

colors = array.from(color.red, color.green)
plot(colors.get(0) == color.red and colors.get(1) == color.green ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 8);
    assert_values_close(&result.plots[0].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[6.0, 6.0]);
    assert_values_close(&result.plots[2].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[3].values, &[3.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_reference_and_copy_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array references")
source = array.new_int()
alias = source
copy = array.copy(source)
method_copy = source.copy()
array.push(alias, 1)
array.push(copy, 2)
method_copy.push(3)
plot(array.size(source))
plot(array.get(source, 0))
plot(array.size(copy))
plot(array.get(copy, 0))
plot(method_copy.size())
plot(method_copy.get(0))
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

    assert_eq!(result.plots.len(), 6);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[4].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[5].values, &[3.0, 3.0, 3.0]);
}

#[test]
fn runs_array_search_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array search")
numbers = array.new_int()
array.push(numbers, 2)
array.push(numbers, 3)
array.push(numbers, 2)
plot(array.includes(numbers, 2) ? 1 : 0)
plot(array.indexof(numbers, 2))
plot(array.lastindexof(numbers, 2))
plot(numbers.indexof(9))
array.sort(numbers)
plot(array.binary_search(numbers, 2))
plot(numbers.binary_search(9))
plot(array.binary_search_leftmost(numbers, 4))
plot(array.binary_search_rightmost(numbers, 4))
plot(numbers.binary_search_leftmost(2))
plot(numbers.binary_search_rightmost(2))

truth_flags = array.from(true, true)
plot(array.every(truth_flags) and truth_flags.some() ? 1 : 0)
truth_flags.push(false)
plot(array.every(truth_flags) ? 99 : (array.some(truth_flags) ? 1 : 0))
truth_numbers = array.from(1, -2, 3)
plot(truth_numbers.every() and array.some(truth_numbers) ? 1 : 0)
truth_numbers.push(0)
plot(array.every(truth_numbers) ? 99 : 1)
truth_floats = array.new_float()
truth_floats.push(na)
truth_floats.push(0)
truth_floats.push(close)
plot(array.every(truth_floats) ? 99 : (truth_floats.some() ? 1 : 0))
empty_truth = array.new_bool()
plot(array.every(empty_truth) and not empty_truth.some() ? 1 : 0)
na_truth = array.new_int(2)
plot(array.every(na_truth) ? 99 : (array.some(na_truth) ? 98 : 1))

words = array.new_string()
words.push("a")
words.push("b")
words.push("a")
plot(words.includes("b") ? words.lastindexof("a") : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.includes(color.green) ? colors.indexof(color.green) : 0)
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

    assert_eq!(result.plots.len(), 19);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[3].values, &[-1.0, -1.0, -1.0]);
    assert_values_close(&result.plots[4].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[5].values, &[-1.0, -1.0, -1.0]);
    assert_values_close(&result.plots[6].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[7].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[8].values, &[0.0, 0.0, 0.0]);
    assert_values_close(&result.plots[9].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[14].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[15].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[16].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[17].values, &[2.0, 2.0, 2.0]);
    assert_values_close(&result.plots[18].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn runs_numeric_array_statistics() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array statistics")
ints = array.new_int()
array.push(ints, 2)
array.push(ints, 5)
array.push(ints, 1)
plot(array.min(ints))
plot(array.max(ints))
plot(array.sum(ints))
plot(array.avg(ints))
plot(array.range(ints))
plot(array.median(ints))
plot(array.percentile_nearest_rank(ints, 50))
plot(ints.percentile_linear_interpolation(75))
plot(array.percentrank(ints, 1))
plot(array.variance(ints, false))
mode_ints = array.from(1, 3, 3, 2, 2)
plot(mode_ints.mode())

floats = array.new_float()
floats.push(close)
floats.push(high)
floats.push(na)
plot(floats.min())
plot(floats.max())
plot(floats.sum())
plot(floats.avg())
plot(floats.range())
plot(floats.median())
plot(floats.percentile_nearest_rank(50))
plot(array.percentile_linear_interpolation(floats, 50))
plot(floats.percentrank(1))
plot(array.variance(floats))
plot(floats.stdev(false))

signs = array.from(-2, 0, 3)
absolutes = signs.abs()
plot(absolutes.get(0) + absolutes.get(1) + absolutes.get(2))
plot(signs.get(0))
float_signs = array.new_float()
float_signs.push(-close)
float_signs.push(na)
float_abs = array.abs(float_signs)
plot(float_abs.get(0))
plot(na(float_abs.get(1)) ? 1 : 0)

standard_values = array.from(2, 4, 4, 4, 5, 5, 7, 9)
standardized = standard_values.standardize()
plot(standardized.get(0))
plot(standardized.get(7))
plot(standard_values.get(0))
standard_with_na = array.from(close, na, high)
standardized_with_na = array.standardize(standard_with_na)
plot(standardized_with_na.size())
plot(na(standardized_with_na.get(1)) ? 1 : 0)

covariance_x = array.from(1, 2, 3)
covariance_y = array.from(1, 5, 7)
plot(array.covariance(covariance_x, covariance_y))
plot(covariance_x.covariance(covariance_y, false))
covariance_with_na_x = array.from(close, na, high)
covariance_with_na_y = array.from(open, close, na)
plot(array.covariance(covariance_with_na_x, covariance_with_na_y))
plot(na(covariance_with_na_x.covariance(covariance_with_na_y, false)) ? 1 : 0)
mismatched_covariance = array.from(1, 2)
plot(na(array.covariance(covariance_x, mismatched_covariance)) ? 1 : 0)

empty = array.new_float()
only_na = array.new_int(2)
empty_standardized = array.standardize(empty)
only_na_standardized = only_na.standardize()
plot(na(array.min(empty)) and na(array.max(only_na)) and na(array.sum(empty)) and na(array.avg(only_na)) and na(array.range(empty)) and na(array.mode(ints)) and na(array.percentile_nearest_rank(empty, 50)) and na(array.percentile_linear_interpolation(ints, 150)) and na(array.percentrank(empty, 0)) and empty_standardized.size() == 0 and only_na_standardized.size() == 0 and na(array.covariance(empty, empty)) and na(array.variance(empty)) and na(only_na.stdev()) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 37);
    assert_values_close(&result.plots[0].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[1].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[2].values, &[8.0, 8.0]);
    assert_values_close(&result.plots[3].values, &[8.0 / 3.0, 8.0 / 3.0]);
    assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[5].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[6].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[7].values, &[3.5, 3.5]);
    assert_values_close(&result.plots[8].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[9].values, &[13.0 / 3.0, 13.0 / 3.0]);
    assert_values_close(&result.plots[10].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[11].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[12].values, &[4.0, 6.0]);
    assert_values_close(&result.plots[13].values, &[6.0, 9.0]);
    assert_values_close(&result.plots[14].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[15].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[16].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[17].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[18].values, &[3.0, 4.5]);
    assert_values_close(&result.plots[19].values, &[100.0, 100.0]);
    assert_values_close(&result.plots[20].values, &[1.0, 2.25]);
    assert_values_close(&result.plots[21].values, &[2.0_f64.sqrt(), 4.5_f64.sqrt()]);
    assert_values_close(&result.plots[22].values, &[5.0, 5.0]);
    assert_values_close(&result.plots[23].values, &[-2.0, -2.0]);
    assert_values_close(&result.plots[24].values, &[2.0, 3.0]);
    assert_values_close(&result.plots[25].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[26].values, &[-1.5, -1.5]);
    assert_values_close(&result.plots[27].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[28].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[29].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[30].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[31].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[32].values, &[3.0, 3.0]);
    assert_values_close(&result.plots[33].values, &[0.0, 0.0]);
    assert_values_close(&result.plots[34].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[35].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[36].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_ordering_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array ordering")
ints = array.new_int()
array.push(ints, 3)
array.push(ints, 1)
array.push(ints, 2)
array.sort(ints)
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
desc_ints = array.from(1, 3, 2)
desc_ints.sort(order.descending)
plot(desc_ints.get(0) * 100 + desc_ints.get(1) * 10 + desc_ints.get(2))
desc_float_special = array.new_float()
desc_float_special.push(na)
desc_float_special.push(close)
desc_float_special.push(high)
desc_float_special.sort(order.descending)
plot(na(desc_float_special.get(0)) and desc_float_special.get(1) == high and desc_float_special.get(2) == close ? 1 : 0)
ints.reverse()
plot(ints.get(0) * 100 + ints.get(1) * 10 + ints.get(2))
unsorted_ints = array.from(30, 10, 20)
sorted_int_indices = unsorted_ints.sort_indices()
plot(sorted_int_indices.get(0) * 100 + sorted_int_indices.get(1) * 10 + sorted_int_indices.get(2))
desc_sorted_int_indices = unsorted_ints.sort_indices(order.descending)
plot(desc_sorted_int_indices.get(0) * 100 + desc_sorted_int_indices.get(1) * 10 + desc_sorted_int_indices.get(2))
plot(unsorted_ints.get(0) * 100 + unsorted_ints.get(1) * 10 + unsorted_ints.get(2))

floats = array.new_float()
floats.push(na)
floats.push(high)
floats.push(close)
floats.sort()
plot(floats.get(0) + floats.get(1))
plot(na(floats.get(2)) ? 1 : 0)
float_indices_source = array.new_float()
float_indices_source.push(na)
float_indices_source.push(high)
float_indices_source.push(close)
float_indices = array.sort_indices(float_indices_source)
plot(float_indices.get(0) * 100 + float_indices.get(1) * 10 + float_indices.get(2))

words = array.new_string()
words.push("b")
words.push("a")
words.push("c")
words.push("")
array.sort(words)
plot(words.get(0) == "a" and words.get(1) == "b" and words.get(2) == "c" and words.get(3) == "" ? 1 : 0)
words.sort(order.descending)
plot(words.get(0) == "" and words.get(1) == "c" and words.get(2) == "b" and words.get(3) == "a" ? 1 : 0)
word_indices = words.sort_indices(order.ascending)
plot(word_indices.get(0) == 3 and word_indices.get(1) == 2 and word_indices.get(2) == 1 and word_indices.get(3) == 0 ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors.reverse()
plot(colors.get(0) == color.green and colors.get(1) == color.red ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 14);
    assert_values_close(&result.plots[0].values, &[123.0, 123.0]);
    assert_values_close(&result.plots[1].values, &[321.0, 321.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[3].values, &[321.0, 321.0]);
    assert_values_close(&result.plots[4].values, &[120.0, 120.0]);
    assert_values_close(&result.plots[5].values, &[21.0, 21.0]);
    assert_values_close(&result.plots[6].values, &[3120.0, 3120.0]);
    assert_values_close(&result.plots[7].values, &[6.0, 9.0]);
    assert_values_close(&result.plots[8].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[9].values, &[210.0, 210.0]);
    assert_values_close(&result.plots[10].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[11].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[12].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[13].values, &[1.0, 1.0]);
}

#[test]
fn runs_array_join_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array join")
ints = array.new_int()
ints.push(1)
ints.push(2)
plot(array.join(ints, "|") == "1|2" ? 1 : 0)

floats = array.new_float()
floats.push(1.25)
floats.push(2.5)
plot(floats.join() == "1.25,2.5" ? 1 : 0)

flags = array.new_bool()
flags.push(false)
flags.push(true)
plot(array.join(flags, "/") == "false/true" ? 1 : 0)

words = array.new_string()
words.push("a")
words.push("b")
plot(words.join("-") == "a-b" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
plot(colors.join("|") == "16711680|32768" ? 1 : 0)

empty = array.new_string()
plot(array.join(empty, "|") == "" ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar_ohlc(1.0, 4.0, 0.0, 2.0), bar_ohlc(2.0, 6.0, 1.0, 3.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 6);
    for plot in &result.plots {
        assert_values_close(&plot.values, &[1.0, 1.0]);
    }
}

#[test]
fn rejects_oversized_array_join_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array join limit")
values = array.new_string(410)
array.set(values, 0, str.repeat("x", 100))
plot(str.length(array.join(values, str.repeat("y", 100))))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.join limit error");

    assert!(
        error
            .message
            .contains("array.join result cannot exceed 40960 characters"),
        "{}",
        error.message
    );
}

#[test]
fn runs_array_slice_concat_operations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array slice concat")
ints = array.new_int()
ints.push(1)
ints.push(2)
ints.push(3)
part = array.slice(ints, 1, 3)
part.set(0, 20)
plot(part.size())
plot(part.get(0) + part.get(1))
plot(ints.get(1))

more = array.new_int()
more.push(4)
returned = array.concat(ints, more)
plot(array.size(ints))
plot(array.size(returned))
plot(returned.get(3))

words = array.new_string()
words.push("a")
words.push("b")
words.push("c")
tail = words.slice(1, 3)
extra = array.new_string()
extra.push("d")
words.concat(extra)
plot(tail.join("|") == "b|c" and words.join("|") == "a|b|c|d" ? 1 : 0)

colors = array.new_color()
colors.push(color.red)
colors.push(color.green)
colors_tail = colors.slice(1, 2)
colors.concat(colors_tail)
plot(colors.get(2) == color.green ? 1 : 0)
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

    assert_eq!(result.plots.len(), 8);
    assert_values_close(&result.plots[0].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[1].values, &[23.0, 23.0]);
    assert_values_close(&result.plots[2].values, &[2.0, 2.0]);
    assert_values_close(&result.plots[3].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[4].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[5].values, &[4.0, 4.0]);
    assert_values_close(&result.plots[6].values, &[1.0, 1.0]);
    assert_values_close(&result.plots[7].values, &[1.0, 1.0]);
}

#[test]
fn handles_invalid_array_slice_bounds() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array slice bounds")
values = array.new_int()
values.push(1)
plot(na(array.slice(values, -1, 1)) ? 1 : 0)
plot(na(values.slice(1, 3)) ? 1 : 0)
plot(na(array.slice(values, 1, 0)) ? 1 : 0)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0]);
    assert_values_close(&result.plots[1].values, &[1.0]);
    assert_values_close(&result.plots[2].values, &[1.0]);
}

#[test]
fn rejects_oversized_array_concat_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array concat limit")
left = array.new_int(100000, 1)
right = array.new_int(1, 2)
array.concat(left, right)
plot(array.size(left))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.concat limit error");

    assert!(
        error
            .message
            .contains("array.concat cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_array_insert_result() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array insert limit")
values = array.new_int(100000, 1)
array.insert(values, 0, 2)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array.insert limit error");

    assert!(
        error
            .message
            .contains("array.insert cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn var_float_array_persists_across_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("var array")
var values = array.new_float()
fresh = array.new_float()
array.push(values, close)
array.push(fresh, close)
plot(array.size(values))
plot(array.size(fresh))
plot(array.get(values, 0))
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

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0, 2.0, 3.0]);
    assert_values_close(&result.plots[1].values, &[1.0, 1.0, 1.0]);
    assert_values_close(&result.plots[2].values, &[1.0, 1.0, 1.0]);
}

#[test]
fn handles_float_array_edge_cases() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array edges")
values = array.new_float()
missing = array.get(values, 0)
popped = array.pop(values)
array.set(values, 10, close)
plot(na(missing) ? 1 : 0)
plot(na(popped) ? 1 : 0)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let result = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_eq!(result.plots.len(), 3);
    assert_values_close(&result.plots[0].values, &[1.0]);
    assert_values_close(&result.plots[1].values, &[1.0]);
    assert_values_close(&result.plots[2].values, &[0.0]);
}

#[test]
fn rejects_negative_float_array_size() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array negative size")
values = array.new_float(-1)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected negative array size error");

    assert!(
        error
            .message
            .contains("array.new_float size cannot be negative"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_oversized_float_array_creation() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array oversized")
values = array.new_float(100001)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected oversized array error");

    assert!(
        error
            .message
            .contains("array.new_float size cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_float_array_push_past_limit() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array push limit")
values = array.new_float(100000)
array.push(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array push limit error");

    assert!(
        error
            .message
            .contains("array.push cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn rejects_float_array_unshift_past_limit() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array unshift limit")
values = array.new_float(100000)
array.unshift(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected array unshift limit error");

    assert!(
        error
            .message
            .contains("array.unshift cannot exceed 100000 elements"),
        "{}",
        error.message
    );
}

#[test]
fn profiles_float_array_storage() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array profile")
var values = array.new_float()
array.push(values, close)
plot(array.size(values))
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let profiled = run_historical_profiled(&analysis.hir.expect("HIR"), &[bar(1.0), bar(2.0)])
        .expect("profiled runtime result");

    assert_eq!(profiled.profile.array_slots, 1);
    assert_eq!(profiled.profile.array_values, 2);
    assert!(profiled.profile.array_value_capacity >= 2);
}

#[test]
fn runs_readonly_float_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("array udf")
first(values) => array.get(values, 0)
var values = array.new_float()
array.push(values, close)
plot(first(values) + array.size(values))
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
fn runs_readonly_int_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("int array udf")
first(values) => array.get(values, 0)
var values = array.new_int()
array.push(values, bar_index)
plot(first(values) + array.size(values))
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
fn runs_readonly_bool_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("bool array udf")
first(values) => array.get(values, 0)
var values = array.new_bool()
array.push(values, bar_index == 0)
plot(first(values) ? array.size(values) : 0)
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
fn runs_readonly_string_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("string array udf")
first(values) => array.get(values, 0)
var values = array.new_string()
array.push(values, "seed")
plot(first(values) == "seed" ? array.size(values) : 0)
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
fn runs_readonly_color_array_udf_parameter() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("color array udf")
first(values) => array.get(values, 0)
var values = array.new_color()
array.push(values, color.red)
plot(first(values) == color.red ? array.size(values) : 0)
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
fn stores_expression_history_before_reading_previous_bars() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("expression history")
plot((close + open)[1])
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
        bar_ohlc(3.0, 4.0, 3.0, 4.0),
        bar_ohlc(5.0, 6.0, 5.0, 6.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.plots.len(), 1);
    assert_eq!(result.plots[0].values[0], PineValue::Na);
    assert_values_close(&result.plots[0].values[1..], &[3.0, 7.0]);
}

#[test]
fn runs_input_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("input history")
length = input.int(2, "Length")
plot(close[length])
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
    assert_eq!(result.plots[0].values[1], PineValue::Na);
    assert_values_close(&result.plots[0].values[2..], &[1.0, 2.0]);
}

#[test]
fn runs_simple_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("simple history")
var values = array.new_int()
array.push(values, 1)
offset = math.min(array.size(values), 1)
plot(close[offset])
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
    assert_values_close(&result.plots[0].values[1..], &[1.0, 2.0]);
}

#[test]
fn runs_series_history_offset() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("series history")
offset = bar_index == 0 ? 0 : 1
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0), bar(2.0), bar(3.0), bar(4.0)];
    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(profiled.result.plots.len(), 1);
    assert_values_close(&profiled.result.plots[0].values, &[1.0, 1.0, 2.0, 3.0]);
    assert_eq!(profiled.profile.max_series_depth, 4);
}

#[test]
fn series_history_offset_out_of_range_returns_na() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("series history out of range")
plot(close[bar_index + 1])
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
    assert_eq!(result.plots[0].values, vec![PineValue::Na; 3]);
}

#[test]
fn rejects_negative_dynamic_history_offset_at_runtime() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("negative dynamic history")
values = array.new_int()
offset = array.indexof(values, 1)
plot(close[offset])
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("runtime should reject negative dynamic history offset");
    assert!(error.message.contains("non-negative"), "{}", error.message);
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

fn bar(close: f64) -> Bar {
    bar_ohlc(close, close, close, close)
}

fn bar_volume(close: f64, volume: f64) -> Bar {
    Bar {
        time: 0,
        open: close,
        high: close,
        low: close,
        close,
        volume,
    }
}

fn bar_ohlc(open: f64, high: f64, low: f64, close: f64) -> Bar {
    bar_ohlcv(open, high, low, close, 1.0)
}

fn bar_ohlcv(open: f64, high: f64, low: f64, close: f64, volume: f64) -> Bar {
    Bar {
        time: 0,
        open,
        high,
        low,
        close,
        volume,
    }
}

fn assert_values_close(actual: &[PineValue], expected: &[f64]) {
    assert_eq!(actual.len(), expected.len());
    for (actual, expected) in actual.iter().zip(expected) {
        let actual = actual
            .as_f64()
            .unwrap_or_else(|| panic!("expected numeric value, got {actual:?}"));
        assert!(
            (actual - expected).abs() < 1e-10,
            "expected {expected}, got {actual}"
        );
    }
}
