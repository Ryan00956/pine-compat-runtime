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
fn accepts_indicator_scale_metadata_parameter() {
    let analysis = analyze(
        r#"indicator("Scale metadata", overlay=true, scale=scale.right)
plot(scale.left == "scale.left" and scale.right == "scale.right" and scale.none == "scale.none" ? close : open)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    for name in ["scale.left", "scale.right", "scale.none"] {
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
fn accepts_indicator_format_precision_metadata_parameters() {
    let analysis = analyze(
        r#"indicator("Format metadata", overlay=true, format=format.percent, precision=2, scale=scale.right)
plot(format.inherit == "format.inherit" and format.price == "format.price" and format.percent == "format.percent" and format.volume == "format.volume" ? close : open)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    for name in [
        "format.inherit",
        "format.price",
        "format.percent",
        "format.volume",
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
fn rejects_unknown_indicator_scale_metadata_value() {
    let analysis = analyze("indicator(\"Scale metadata\", scale=\"custom\")\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_VALUE"),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn rejects_unknown_indicator_format_metadata_value() {
    let analysis = analyze("indicator(\"Format metadata\", format=\"custom\")\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_VALUE"),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn rejects_out_of_range_indicator_precision_metadata_value() {
    let analysis = analyze("indicator(\"Format metadata\", precision=17)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_VALUE"),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn accepts_alertcondition_const_string_subset() {
    let analysis = analyze(
        r#"alertcondition(close > open, "Up", "Close is above open")
alertcondition(close > open, "OHLCV", "O={{open}} H={{high}} L={{low}} C={{close}} V={{volume}}")
"#,
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
            .any(|feature| feature.feature == "alertcondition")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_alert_const_string_subset() {
    let analysis = analyze(
        r#"alert("Reached")
alert("Every call", alert.freq_all)
alert("Once", freq=alert.freq_once_per_bar)
alert("Close", freq=alert.freq_once_per_bar_close)
"#,
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
            .any(|feature| feature.feature == "alert")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_alert_dynamic_message_subset() {
    let analysis = analyze(
        r#"message = input.string("Reached", "Message")
alert(message)
alert(str.tostring(close))
"#,
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
            .any(|feature| feature.feature == "alert")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_alert_unsupported_frequency() {
    let analysis = analyze(
        r#"alert("Reached", freq="once")
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "alert_frequency")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_alert_placeholders() {
    let analysis = analyze(
        r#"alert("{{close}}")
alertcondition(true, "{{close}}", "Title placeholder")
alertcondition(true, "Title", "{{timenow}}")
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "alert_placeholders")
    );
    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("alert placeholder `{{close}}`"))
    );
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .message
            .contains("alert placeholder `{{timenow}}`")
    }));
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_alert_side_effects_inside_functions() {
    let analysis = analyze(
        r#"f() =>
    alert("Fn")
    close
plot(f())
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "function_side_effect")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_alertcondition_dynamic_messages() {
    let analysis = analyze(
        r#"title = input.string("Up", "Title")
alertcondition(close > open, title, "Message")
"#,
    );

    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_CALL_ARG_TYPE"
            && diagnostic.message.contains("argument `title`")
            && diagnostic.message.contains("Input String")
    }));
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_alertcondition_side_effects_inside_functions() {
    let analysis = analyze(
        r#"f() =>
    alertcondition(true, "Fn", "Message")
    close
plot(f())
"#,
    );

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "function_side_effect")
    );
    assert!(analysis.hir.is_none());
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
        "id = label.new(bar_index, high, \"High\")\nother = label.new(x=1, y=close, text=\"Close\", xloc=xloc.bar_index, yloc=yloc.price, color=color.green, style=label.style_label_up, textcolor=color.white, size=size.small, textalign=text.align_right, tooltip=\"Tip\", text_font_family=font.family_monospace, text_formatting=text.format_bold)\nplot(close)\n",
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
fn accepts_label_new_int_size() {
    let analysis = analyze("label.new(bar_index, high, \"High\", size=12)\nplot(close)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_label_new_non_int_size() {
    let analysis = analyze("label.new(bar_index, high, \"High\", size=12.5)\nplot(close)\n");

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
fn accepts_label_mutation_methods() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\ncopy = label.copy(id)\nlabel.set_x(id, bar_index)\nlabel.set_xloc(id, time, xloc.bar_time)\nlabel.set_y(id, low)\nlabel.set_xy(id, bar_index, close)\nlabel.set_yloc(id, yloc.abovebar)\nlabel.set_text(id, \"Close\")\nlabel.set_color(id, color.green)\nlabel.set_textcolor(id, color.white)\nlabel.set_style(id, label.style_label_up)\nlabel.set_size(id, size.small)\nlabel.set_tooltip(id, \"Tip\")\nlabel.set_textalign(id, text.align_left)\nlabel.set_text_font_family(id, font.family_monospace)\nlabel.set_text_formatting(id, text.format_bold + text.format_italic)\nlabel.set_text_formatting(na, text.format_italic)\nlabel.set_text(na, \"noop\")\nlabel.delete(na)\nlabel.delete(id)\nplot(label.get_x(copy))\nplot(close)\n",
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
            .any(|feature| feature.feature == "label.copy")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "label.set_textalign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "label.set_text_font_family")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "label.set_text_formatting")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_label_text_formatting() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\", text_formatting=text.format_bold + 4)\nlabel.set_text_formatting(id, text.format_italic + 4)\nplot(close)\n",
    );

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
fn rejects_unsupported_label_set_xloc_values() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\nlabel.set_xloc(id, bar_index, \"xloc.middle\")\nplot(close)\n",
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("xloc.bar_time")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_unsupported_label_set_yloc_values() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\nlabel.set_yloc(id, \"yloc.middle\")\nplot(close)\n",
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("yloc.abovebar")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_label_getter_methods() {
    let analysis = analyze(
        "id = label.new(bar_index, high, \"High\")\nplot(label.get_x(id))\nplot(label.get_y(id))\nplot(str.length(label.get_text(id)))\nplot(label.get_x(na))\n",
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
            .any(|feature| feature.feature == "label.get_text")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_label_side_effects_inside_functions() {
    let analysis = analyze(
        "change(price) =>\n    id = label.new(bar_index, price, \"High\")\n    copy = label.copy(id)\n    label.set_xloc(copy, time, xloc.bar_time)\n    label.set_yloc(copy, yloc.abovebar)\n    label.delete(id)\n    price\nplot(change(close))\n",
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
    let analysis = analyze("label.set_point(na, na)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "label.set_point"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_drawing_object_method_syntax() {
    let analysis = analyze(
        "label_id = label.new(bar_index, high, \"start\")\nline_id = line.new(bar_index, low, bar_index + 1, high)\nbox_id = box.new(bar_index, high, bar_index + 1, low)\ntable_id = table.new(position.top_right, 1, 1)\nlabel_id.set_text(\"method\")\nlabel_id.set_xy(bar_index, close)\nline_id.set_xy1(bar_index, low)\nline_id.set_color(color.green)\nbox_id.set_lefttop(bar_index, high)\nbox_id.set_xloc(bar_index - 1, bar_index + 1, xloc.bar_index)\ntable_id.cell(0, 0, \"A\")\ntable_id.set_bgcolor(color.green)\nplot(str.length(label_id.get_text()))\nplot(line_id.get_x1())\nplot(box_id.get_right())\nplot(close)\n",
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
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.set_xy1")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_xloc")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unknown_drawing_object_method_syntax() {
    let analysis = analyze("id = label.new(bar_index, high, \"start\")\nid.set_point(na)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UNKNOWN_METHOD"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_drawing_object_method_side_effects_inside_functions() {
    let analysis = analyze(
        "change(price) =>\n    id = label.new(bar_index, price, \"start\")\n    id.set_text(\"method\")\n    price\nplot(change(close))\n",
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
fn accepts_minimal_line_new() {
    let analysis = analyze(
        "id = line.new(bar_index - 1, low, bar_index, high)\nother = line.new(x1=0, y1=open, x2=bar_index, y2=close)\nstyled = line.new(x1=bar_index, y1=low, x2=bar_index + 1, y2=high, xloc=xloc.bar_index, extend=extend.right, color=color.green, style=line.style_dashed, width=2, force_overlay=false)\ncopy = line.copy(id)\nline.set_x1(id, bar_index)\nline.set_y1(id, low)\nline.set_xy1(id, bar_index, open)\nline.set_x2(id, bar_index)\nline.set_y2(id, high)\nline.set_xy2(id, bar_index, close)\nline.set_xloc(id, bar_index - 2, bar_index + 2, xloc.bar_index)\nline.set_color(id, color.green)\nline.set_width(id, 2)\nline.set_style(id, line.style_dashed)\nline.set_extend(id, extend.right)\nplot(line.get_price(copy, bar_index))\nplot(line.get_x1(copy))\nplot(line.get_y1(copy))\nplot(line.get_x2(copy))\nplot(line.get_y2(copy))\nline.delete(na)\nline.delete(id)\nplot(close)\n",
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
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.copy")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.set_xloc")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.get_price")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.get_x1")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.get_y1")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.get_x2")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "line.get_y2")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unimplemented_line_methods() {
    let analysis = analyze("line.set_first_point(na, na)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "line.set_first_point"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_unsupported_line_set_xloc_values() {
    let analysis = analyze(
        "id = line.new(bar_index, low, bar_index + 1, high)\nline.set_xloc(id, time, time + 60000, xloc.bar_time)\nplot(close)\n",
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("xloc.bar_index")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_invalid_line_new_options() {
    let analysis = analyze(
        "id = line.new(x1=bar_index, y1=low, x2=bar_index, y2=high, xloc=xloc.bar_time, style=\"line.style_unknown\")\nplot(close)\n",
    );

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
fn rejects_line_side_effects_inside_functions() {
    let analysis = analyze(
        "change(price) =>\n    id = line.new(bar_index - 1, price, bar_index, price)\n    copy = line.copy(id)\n    line.set_xy1(copy, bar_index, low)\n    line.delete(id)\n    price\nplot(change(close))\n",
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
fn accepts_minimal_box_new() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index, low)\nother = box.new(left=0, top=open, right=bar_index, bottom=close)\nstyled = box.new(left=bar_index, top=high, right=bar_index + 1, bottom=low, border_color=color.white, border_width=2, border_style=line.style_dashed, extend=extend.right, xloc=xloc.bar_index, bgcolor=color.green, text=\"box text\", text_size=size.small, text_color=color.white, text_halign=text.align_left, text_valign=text.align_top, text_wrap=text.wrap_auto, text_font_family=font.family_monospace, force_overlay=false, text_formatting=text.format_bold + text.format_italic)\ncopy = box.copy(id)\nbox.set_left(id, bar_index)\nbox.set_top(id, high)\nbox.set_right(id, bar_index)\nbox.set_bottom(id, low)\nbox.set_lefttop(id, bar_index, close)\nbox.set_rightbottom(id, bar_index, open)\nbox.set_bgcolor(id, color.green)\nbox.set_border_color(id, color.white)\nbox.set_border_width(id, 2)\nbox.set_border_style(id, line.style_dashed)\nbox.set_extend(id, extend.right)\nbox.set_xloc(id, bar_index - 2, bar_index + 2, xloc.bar_index)\nbox.set_text(id, \"box text\")\nbox.set_text_color(id, color.white)\nbox.set_text_size(id, size.small)\nbox.set_text_halign(id, text.align_left)\nbox.set_text_valign(id, text.align_top)\nbox.set_text_wrap(id, text.wrap_auto)\nbox.set_text_font_family(id, font.family_monospace)\nbox.set_text_formatting(id, text.format_bold + text.format_italic)\nbox.set_text_formatting(na, text.format_italic)\nbox.delete(na)\nbox.delete(id)\nplot(box.get_top(copy))\nplot(box.get_bottom(copy))\nplot(box.get_left(copy))\nplot(box.get_right(copy))\nplot(close)\n",
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
            .any(|feature| feature.feature == "box.new")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.copy")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.get_top")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.get_bottom")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.get_left")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.get_right")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_extend")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_xloc")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_color")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_size")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_halign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_valign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_wrap")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_font_family")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "box.set_text_formatting")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unimplemented_box_methods() {
    let analysis = analyze("box.set_top_left_point(na, na)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "box.set_top_left_point"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_invalid_box_text_formatting() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index, low)\nbox.set_text_formatting(id, text.format_bold + 4)\nplot(close)\n",
    );

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
fn accepts_box_new_int_text_size() {
    let analysis =
        analyze("id = box.new(bar_index, high, bar_index, low, text_size=19)\nplot(close)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_box_new_text_size() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index, low, text_size=\"size.bad\")\nplot(close)\n",
    );

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
fn accepts_box_set_int_text_size() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index, low)\nbox.set_text_size(id, 19)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_box_set_text_size() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index, low)\nbox.set_text_size(id, \"size.bad\")\nplot(close)\n",
    );

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
fn rejects_unsupported_box_set_xloc_values() {
    let analysis = analyze(
        "id = box.new(bar_index, high, bar_index + 1, low)\nbox.set_xloc(id, time, time + 60000, xloc.bar_time)\nplot(close)\n",
    );

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("xloc.bar_index")),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_invalid_box_new_options() {
    let analysis = analyze(
        "id = box.new(left=bar_index, top=high, right=bar_index, bottom=low, xloc=xloc.bar_time, text_formatting=text.format_bold + 4)\nplot(close)\n",
    );

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
fn rejects_box_side_effects_inside_functions() {
    let analysis = analyze(
        "change(price) =>\n    id = box.new(bar_index, price, bar_index, low)\n    copy = box.copy(id)\n    box.set_lefttop(copy, bar_index, high)\n    box.delete(id)\n    price\nplot(change(close))\n",
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
fn accepts_minimal_table_new_and_cell() {
    let analysis = analyze(
        "id = table.new(position.top_right, 2, 2, bgcolor=color.gray, frame_color=color.black, frame_width=2, border_color=color.white, border_width=1)\ntable.cell(id, 0, 0, \"A\")\ntable.cell(id, column=1, row=0, text=\"B\", bgcolor=color.green, text_color=color.white, tooltip=\"initial\", text_font_family=font.family_monospace, text_formatting=text.format_bold)\ntable.cell_set_text(id, 1, 0, \"B2\")\ntable.cell_set_bgcolor(id, 1, 0, color.red)\ntable.cell_set_text_color(id, 1, 0, color.blue)\ntable.cell_set_width(id, 1, 0, 25)\ntable.cell_set_height(id, 1, 0, 40)\ntable.cell_set_text_size(id, 1, 0, size.small)\ntable.cell_set_text_halign(id, 1, 0, text.align_left)\ntable.cell_set_text_valign(id, 1, 0, text.align_top)\ntable.cell_set_text_wrap(id, 1, 0, text.wrap_auto)\ntable.cell_set_tooltip(id, 1, 0, \"updated\")\ntable.cell_set_text_font_family(id, 1, 0, font.family_default)\ntable.cell_set_text_formatting(id, 1, 0, text.format_bold + text.format_italic)\ntable.merge_cells(id, 0, 0, 1, 0)\ntable.set_position(id, position.bottom_right)\ntable.set_bgcolor(id, color.yellow)\ntable.set_frame_color(id, color.black)\ntable.set_frame_width(id, 3)\ntable.set_border_color(id, color.white)\ntable.set_border_width(id, 4)\ntable.clear(id, 0, 0, 1, 1)\ntable.set_position(na, position.top_left)\ntable.set_bgcolor(na, color.red)\ntable.set_frame_color(na, color.blue)\ntable.set_frame_width(na, 2)\ntable.set_border_color(na, color.green)\ntable.set_border_width(na, 5)\ntable.cell_set_text(na, 0, 1, \"noop\")\ntable.cell_set_bgcolor(na, 0, 1, color.red)\ntable.cell_set_text_color(na, 0, 1, color.blue)\ntable.cell_set_width(na, 0, 1, 25)\ntable.cell_set_height(na, 0, 1, 40)\ntable.cell_set_text_size(na, 0, 1, size.small)\ntable.cell_set_text_halign(na, 0, 1, text.align_left)\ntable.cell_set_text_valign(na, 0, 1, text.align_top)\ntable.cell_set_text_wrap(na, 0, 1, text.wrap_none)\ntable.cell_set_tooltip(na, 0, 1, \"noop\")\ntable.cell_set_text_font_family(na, 0, 1, font.family_monospace)\ntable.cell_set_text_formatting(na, 0, 1, text.format_italic)\ntable.cell(na, 0, 1, \"noop\")\ntable.merge_cells(na, 0, 0, 0, 0)\ntable.clear(na, 0, 0, 0, 0)\ntable.delete(na)\ntable.delete(id)\nplot(close)\n",
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
            .any(|feature| feature.feature == "table.new")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_position")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_bgcolor")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_frame_color")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_frame_width")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_border_color")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.set_border_width")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_bgcolor")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_color")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_width")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_height")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_size")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_halign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_valign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_wrap")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.merge_cells")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_tooltip")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_font_family")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "table.cell_set_text_formatting")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_table_text_wrap() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\")\ntable.cell_set_text_wrap(id, 0, 0, \"text.wrap_bad\")\nplot(close)\n",
    );

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
fn accepts_table_cell_int_text_size() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\", text_size=19)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_table_cell_text_size() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\", text_size=\"size.bad\")\nplot(close)\n",
    );

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
fn accepts_table_cell_set_int_text_size() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\")\ntable.cell_set_text_size(id, 0, 0, 19)\nplot(close)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_invalid_table_cell_set_text_size() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\")\ntable.cell_set_text_size(id, 0, 0, \"size.bad\")\nplot(close)\n",
    );

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
fn rejects_invalid_table_text_formatting() {
    let analysis = analyze(
        "id = table.new(position.top_right, 1, 1)\ntable.cell(id, 0, 0, \"A\", text_formatting=text.format_bold + 4)\nplot(close)\n",
    );

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
fn accepts_indicator_metadata_positional_order() {
    let analysis = analyze(
        "indicator(\"Demo\", \"D\", true, format.price, 2, scale.right, 10)\nplot(close[bar_index])\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("hir");
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
