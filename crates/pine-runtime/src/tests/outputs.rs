use pine_sema::analyze_source;
use pine_syntax::SourceFile;

use super::*;

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
fn collects_label_new_snapshots() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("labels")
label.new(bar_index, high, "bar")
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

    assert_eq!(result.labels.len(), 3);
    for (index, label) in result.labels.iter().enumerate() {
        assert_eq!(label.id, index as u32 + 1);
        assert_eq!(label.snapshots.len(), 1);
        let snapshot = &label.snapshots[0];
        assert_eq!(snapshot.bar_index, index);
        assert!(snapshot.exists);
        assert_eq!(snapshot.x, PineValue::Int(index as i64));
        assert_eq!(snapshot.y, PineValue::Float(index as f64 + 1.0));
        assert_eq!(snapshot.text, PineValue::String("bar".to_owned()));
        assert_eq!(
            snapshot.xloc,
            PineValue::String("xloc.bar_index".to_owned())
        );
        assert_eq!(snapshot.yloc, PineValue::String("yloc.price".to_owned()));
        assert_eq!(snapshot.color, PineValue::Na);
        assert_eq!(
            snapshot.style,
            PineValue::String("label.style_label_down".to_owned())
        );
        assert_eq!(snapshot.text_color, PineValue::Na);
        assert_eq!(snapshot.size, PineValue::String("size.normal".to_owned()));
        assert_eq!(snapshot.tooltip, PineValue::String(String::new()));
    }
}

#[test]
fn collects_label_new_options() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label options")
label.new(x=bar_index, y=high, text="bar", xloc=xloc.bar_index, yloc=yloc.price, color=color.green, style=label.style_label_up, textcolor=color.white, size=size.small, tooltip="Tip")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let snapshot = &result.labels[0].snapshots[0];

    assert_eq!(
        snapshot.xloc,
        PineValue::String("xloc.bar_index".to_owned())
    );
    assert_eq!(snapshot.yloc, PineValue::String("yloc.price".to_owned()));
    assert_eq!(snapshot.color, PineValue::Color(0x008000));
    assert_eq!(
        snapshot.style,
        PineValue::String("label.style_label_up".to_owned())
    );
    assert_eq!(snapshot.text_color, PineValue::Color(0xFFFFFF));
    assert_eq!(snapshot.size, PineValue::String("size.small".to_owned()));
    assert_eq!(snapshot.tooltip, PineValue::String("Tip".to_owned()));
}

#[test]
fn collects_label_mutation_snapshots() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label mutation")
id = label.new(bar_index, high, "start")
label.set_x(id, 1)
label.set_y(id, close + 1)
label.set_text(id, "changed")
label.set_color(id, color.green)
label.set_textcolor(id, color.white)
label.set_style(id, label.style_label_up)
label.set_size(id, size.small)
label.set_tooltip(id, "Tip")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let bars = vec![bar(1.0)];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");
    let label = &result.labels[0];

    assert_eq!(label.snapshots.len(), 9);
    assert_eq!(label.snapshots[0].x, PineValue::Int(0));
    assert_eq!(label.snapshots[1].x, PineValue::Int(1));
    assert_eq!(label.snapshots[2].y, PineValue::Float(2.0));
    assert_eq!(
        label.snapshots[3].text,
        PineValue::String("changed".to_owned())
    );
    assert_eq!(label.snapshots[4].color, PineValue::Color(0x008000));
    assert_eq!(label.snapshots[5].text_color, PineValue::Color(0xFFFFFF));
    assert_eq!(
        label.snapshots[6].style,
        PineValue::String("label.style_label_up".to_owned())
    );
    assert_eq!(
        label.snapshots[7].size,
        PineValue::String("size.small".to_owned())
    );
    assert_eq!(
        label.snapshots[8].tooltip,
        PineValue::String("Tip".to_owned())
    );
}

#[test]
fn skips_noop_label_mutations() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label noops")
var id = label.new(bar_index, high, "start")
label.set_text(id, "start")
label.set_text(na, "ignored")
if bar_index == 1
    label.set_text(id, "start")
plot(close)
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

    assert_eq!(result.labels[0].snapshots.len(), 1);
}

#[test]
fn collects_label_delete_snapshots() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label delete")
var id = label.new(bar_index, high, "start")
if bar_index == 1
    label.delete(id)
if bar_index == 2
    label.set_text(id, "ignored")
    label.delete(id)
label.delete(na)
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
    let label = &result.labels[0];

    assert_eq!(label.snapshots.len(), 2);
    assert!(label.snapshots[0].exists);
    assert_eq!(label.snapshots[0].bar_index, 0);
    assert!(!label.snapshots[1].exists);
    assert_eq!(label.snapshots[1].bar_index, 1);
}

#[test]
fn rejects_label_creation_past_limit() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label limit")
for i = 0 to 500
    label.new(i, close, "x")
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let error = run_historical(&analysis.hir.expect("HIR"), &[bar(1.0)])
        .expect_err("expected label limit error");

    assert!(error.message.contains("label count cannot exceed"));
}

#[test]
fn profiles_label_storage() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label profile")
id = label.new(bar_index, high, "start")
label.set_text(id, "changed")
label.delete(id)
plot(close)
"#,
    );
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );

    let profiled =
        run_historical_profiled(&analysis.hir.expect("HIR"), &[bar(1.0)]).expect("runtime result");

    assert_eq!(profiled.profile.labels, 1);
    assert_eq!(profiled.profile.label_snapshots, 3);
    assert!(profiled.profile.label_capacity >= 1);
    assert!(profiled.profile.label_snapshot_capacity >= 3);
}

#[test]
fn collects_conditional_and_stored_label_ids() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("conditional labels")
if close > 1
    created = label.new(bar_index, close, "stored")
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

    assert_eq!(result.labels.len(), 2);
    assert_eq!(result.labels[0].id, 1);
    assert_eq!(result.labels[0].snapshots[0].bar_index, 1);
    assert_eq!(result.labels[0].snapshots[0].x, PineValue::Int(1));
    assert_eq!(result.labels[1].id, 2);
    assert_eq!(result.labels[1].snapshots[0].bar_index, 2);
    assert_eq!(result.labels[1].snapshots[0].x, PineValue::Int(2));
}

#[test]
fn collects_label_side_effects_in_control_flow() {
    let source = SourceFile::new(
        "test.pine",
        r#"indicator("label control flow")
var label_id = label.new(bar_index, high, "start")
if bar_index == 1
    label.set_text(label_id, "if")
direction = close > open ? 1 : -1
switch direction
    1 => label.set_color(label_id, color.green)
    => label.set_color(label_id, color.red)
for i = 0 to 0
    if bar_index == 2
        label.set_tooltip(label_id, "for")
j = 0
while j < 1
    if bar_index == 3
        label.set_size(label_id, size.small)
    j := j + 1
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
        bar_ohlc(1.0, 1.0, 1.0, 1.0),
        bar_ohlc(1.0, 2.0, 1.0, 2.0),
        bar_ohlc(3.0, 2.0, 2.0, 2.0),
        bar_ohlc(3.0, 4.0, 3.0, 4.0),
    ];
    let result = run_historical(&analysis.hir.expect("HIR"), &bars).expect("runtime result");

    assert_eq!(result.labels.len(), 1);
    let snapshots = &result.labels[0].snapshots;
    assert_eq!(snapshots.len(), 8);
    assert_eq!(snapshots[0].bar_index, 0);
    assert_eq!(snapshots[1].color, PineValue::Color(0xFF0000));
    assert_eq!(snapshots[2].text, PineValue::String("if".to_owned()));
    assert_eq!(snapshots[3].color, PineValue::Color(0x008000));
    assert_eq!(snapshots[4].color, PineValue::Color(0xFF0000));
    assert_eq!(snapshots[5].tooltip, PineValue::String("for".to_owned()));
    assert_eq!(snapshots[6].color, PineValue::Color(0x008000));
    assert_eq!(
        snapshots[7].size,
        PineValue::String("size.small".to_owned())
    );
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
