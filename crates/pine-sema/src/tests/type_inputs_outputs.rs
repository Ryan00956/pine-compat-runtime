use super::*;

#[test]
fn accepts_input_string_in_conditions() {
    let analysis =
        analyze("mode = input.string(\"SMA\", \"Mode\")\nplot(mode == \"SMA\" ? close : open)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "input.string")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_additional_input_variants() {
    let analysis = analyze(
        "threshold = input.price(2.5, \"Price\")\nstart = input.time(0, \"Start\")\nsymbol = input.symbol(\"AAPL\", \"Symbol\")\ntimeframe = input.timeframe(\"D\", \"Timeframe\")\nsession = input.session(\"0930-1600\", \"Session\")\nnotes = input.text_area(\"Plan\", \"Notes\")\nplot(time >= start and symbol == \"AAPL\" and timeframe == \"D\" and session == \"0930-1600\" and notes == \"Plan\" ? math.max(close, threshold) : open)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for name in [
        "input.price",
        "input.time",
        "input.symbol",
        "input.timeframe",
        "input.session",
        "input.text_area",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == name),
            "{name} should be reported as supported"
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_generic_input_variants() {
    let analysis = analyze(
        "length = input(2, \"Length\")\nscale = input(1.5, \"Scale\")\nenabled = input(true, \"Enabled\")\nmode = input(\"SMA\", \"Mode\")\nshade = input(color.orange, \"Shade\")\nplot(enabled and mode == \"SMA\" ? ta.sma(close, length) * scale : open, color=color.new(shade, 10))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "input")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_common_input_metadata_parameters() {
    let analysis = analyze(
        r#"length = input.int(2, "Length", minval=1, maxval=20, step=1, options=[1, 2, 3], tooltip="Bars", inline="row", group="Settings", confirm=true, display=display.all)
scale = input.float(1.5, "Scale", minval=0.5, maxval=5.0, step=0.25, options=[1.0, 1.5], display=display.none)
enabled = input.bool(true, "Enabled", tooltip="Toggle", inline="row", group="Settings", confirm=false)
mode = input.string("SMA", "Mode", options=["SMA", "EMA"], tooltip="Mode")
shade = input.color(color.orange, "Shade", group="Style")
src = input.source(close, "Source", tooltip="Price", inline="src", group="Settings", confirm=true, display=display.all)
plot(enabled and mode == "SMA" ? math.max(src, length) * scale : close, color=shade)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_bgcolor_and_barcolor() {
    let analysis =
        analyze("bgcolor(close > open ? color.green : na)\nbarcolor(color.red)\nplot(close)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "bgcolor")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barcolor")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_common_output_metadata_parameters() {
    let analysis = analyze(
        r#"p = plot(close, title="Close", color=color.green, linewidth=2, style=plot.style_line, trackprice=false, histbase=0, offset=1, join=false, editable=true, show_last=10, display=display.pane, format=format.price, precision=2, force_overlay=false)
h = hline(2, title="Two", color=color.gray, linestyle=hline.style_dotted, linewidth=1, editable=true, display=display.price_scale)
fill(p, h, color=color.new(color.green, 80), title="Fill", editable=false, show_last=5, fillgaps=true, display=display.status_line)
bgcolor(color.new(color.blue, 90), title="Background", offset=0, editable=false, show_last=3, display=display.data_window)
barcolor(close > open ? color.green : color.red, title="Bars", offset=0, editable=true, show_last=3, display=display.none)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    for name in [
        "display.pane",
        "display.price_scale",
        "display.status_line",
        "display.data_window",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|feature| feature.feature == name),
            "{name} should be reported as supported"
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_plotchar() {
    let analysis = analyze(
        "plotchar(close > open, title=\"Marker\", char=\"x\", color=color.green, location=location.abovebar, offset=1, text=\"Up\", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plotchar")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_plotshape() {
    let analysis = analyze(
        "plotshape(close > open, title=\"Buy\", style=shape.triangleup, location=location.belowbar, color=color.green, offset=1, text=\"Buy\", textcolor=color.white, editable=true, size=size.small, show_last=5, display=display.all, force_overlay=false)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plotshape")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "shape.triangleup")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_plotarrow() {
    let analysis = analyze(
        "plotarrow(close - open, title=\"Momentum\", colorup=color.green, colordown=color.red, offset=1, minheight=5, maxheight=20, editable=true, show_last=5, display=display.all, force_overlay=false)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plotarrow")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_plotbar() {
    let analysis = analyze(
        "plotbar(open, high, low, close, title=\"Bars\", color=color.green, editable=true, show_last=5, display=display.all)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plotbar")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_plotcandle() {
    let analysis = analyze(
        "plotcandle(open, high, low, close, title=\"Candles\", color=color.green, wickcolor=color.white, editable=true, show_last=5, bordercolor=color.red, display=display.all)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "plotcandle")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_minimal_label_new() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\nother = label.new(x=1, y=close, text=\"Close\", xloc=xloc.bar_index, yloc=yloc.price, color=color.green, style=label.style_label_up, textcolor=color.white, size=size.small, tooltip=\"Tip\")\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "label.new")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unsupported_label_new_modes() {
    let analysis = analyze(
        "label.new(bar_index, high, \"High\", xloc=xloc.bar_time, yloc=yloc.abovebar, style=\"label.style_unknown\", size=\"size.massive\")\nplot(close)\n",
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("xloc.bar_index")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("yloc.price")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("label.style_label_down")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("size.normal")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_label_mutation_methods() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\nlabel.set_x(id, bar_index)\nlabel.set_y(id, low)\nlabel.set_xy(id, bar_index, close)\nlabel.set_text(id, \"Close\")\nlabel.set_color(id, color.green)\nlabel.set_textcolor(id, color.white)\nlabel.set_style(id, label.style_label_up)\nlabel.set_size(id, size.small)\nlabel.set_tooltip(id, \"Tip\")\nlabel.set_text(na, \"noop\")\nlabel.delete(na)\nlabel.delete(id)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "label.set_text")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_label_side_effects_inside_functions() {
    let analysis = analyze(
        "change(price) =>\n    id = label.new(bar_index, price, \"High\")\n    label.delete(id)\n    price\nplot(change(close))\n",
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "function_side_effect"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_unimplemented_label_methods() {
    let analysis = analyze("label.get_text(na)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "label.get_text"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_minimal_line_new() {
    let analysis = analyze(
        "id = line.new(bar_index - 1, low, bar_index, high)\nother = line.new(x1=0, y1=open, x2=bar_index, y2=close)\nline.set_x1(id, bar_index)\nline.set_y1(id, low)\nline.set_xy1(id, bar_index, open)\nline.set_x2(id, bar_index)\nline.set_y2(id, high)\nline.set_xy2(id, bar_index, close)\nline.set_color(id, color.green)\nline.set_width(id, 2)\nline.set_style(id, line.style_dashed)\nline.set_extend(id, extend.right)\nline.delete(na)\nline.delete(id)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.new")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unimplemented_line_methods() {
    let analysis = analyze("line.get_price(na, bar_index)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "line.get_price"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_input_history_offset() {
    let analysis = analyze("len = input.int(1, \"Length\")\nx = close[len]\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_indicator_max_bars_back() {
    let analysis = analyze("indicator(\"Demo\", max_bars_back=10)\nplot(close[bar_index])\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.max_bars_back, Some(10));
}

#[test]
fn rejects_negative_indicator_max_bars_back() {
    let analysis = analyze("indicator(\"Demo\", max_bars_back=-1)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_VALUE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_non_const_indicator_max_bars_back() {
    let analysis = analyze("indicator(\"Demo\", max_bars_back=bar_index)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn simple_int_params_accept_input_int_expressions() {
    let analysis = analyze("length = input.int(2, \"Length\") + 1\nplot(ta.sma(close, length))\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn simple_int_params_reject_series_int() {
    let analysis = analyze("plot(ta.sma(close, bar_index))\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}
