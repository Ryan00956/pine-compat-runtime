use super::*;

#[test]
fn accepts_fixnan() {
    let analysis = analyze(
        "source = close > open ? close : na\nplot(fixnan(source) + (fixnan(color.green == color.red ? color.green : na) == color.green ? 1 : 0))\n",
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
            .any(|feature| feature.feature == "fixnan")
    );
}

#[test]
fn accepts_type_casts() {
    let analysis = analyze(
        "length = int(2.9)\nscale = float(length)\nflag = bool(close - open)\nlabel = string(close)\nshade = color(color.red)\nmissing = color(na)\nplot(flag ? ta.sma(close, length) + scale + str.length(label) + (shade == color.red and na(missing) ? 1 : 0) : float(na))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in ["int", "float", "bool", "string", "color"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} should be reported as supported"
        );
    }
}

#[test]
fn accepts_ta_window_statistics() {
    let analysis = analyze(
        "plot(ta.bbw(close, 3, 2) + ta.stdev(close, 3, false) + ta.variance(close, 3, true) + ta.range(close, 3) + ta.dev(close, 3) + ta.vwma(close, 3) + ta.wma(close, 3) + ta.hma(close, 4) + ta.swma(close) + ta.alma(close, 4, 0.85, 6, true) + ta.linreg(close, 3, 0) + ta.correlation(close, 1, 3) + ta.covariance(1, high, 3) + ta.median(1, 3) + ta.mode(1, 3) + ta.percentile_nearest_rank(1, 3, 50) + ta.percentile_linear_interpolation(1, 3, 50) + ta.percentrank(1, 3))\n",
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
            .any(|feature| feature.feature == "ta.bbw")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.stdev")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.variance")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.range")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.dev")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.correlation")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.covariance")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.median")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.mode")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.percentile_nearest_rank")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.percentile_linear_interpolation")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.percentrank")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.vwma")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.wma")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.hma")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.swma")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.alma")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.linreg")
    );
}

#[test]
fn accepts_ta_numeric_series_sources() {
    let analysis = analyze(
        "[macd, signal, hist] = ta.macd(bar_index, 2, 3, 2)\n[basis, upper, lower] = ta.bb(bar_index, 3, 2)\nplot(ta.sma(bar_index, 2) + ta.ema(bar_index, 2) + ta.rma(bar_index, 2) + ta.rsi(bar_index, 2) + ta.bbw(bar_index, 3, 2) + ta.stdev(bar_index, 3) + ta.variance(bar_index, 3) + ta.range(bar_index, 3) + ta.dev(bar_index, 3) + ta.vwma(bar_index, 3) + ta.wma(bar_index, 3) + ta.hma(bar_index, 4) + ta.swma(bar_index) + ta.alma(bar_index, 4, 0.85, 6) + ta.linreg(bar_index, 3, 0) + ta.median(bar_index, 3) + ta.mode(bar_index, 3) + ta.percentrank(bar_index, 3) + ta.mom(bar_index, 2) + ta.roc(bar_index, 2) + macd + signal + hist + basis + upper + lower + (ta.rising(bar_index, 2) ? 1 : 0) + (ta.falling(bar_index, 2) ? 1 : 0))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
}

#[test]
fn accepts_ta_momentum_history_calls() {
    let analysis = analyze(
        "flag_changed = ta.change(close > open)\nplot(ta.change(bar_index) + ta.mom(close, 2) + ta.roc(open, 2) + (flag_changed ? 1 : 0))\n",
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
            .any(|feature| feature.feature == "ta.change")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.mom")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.roc")
    );
    let hir = analysis.hir.expect("HIR");
    assert_eq!(hir.history.max_constant_offset, 2);
    assert!(!hir.history.has_dynamic_offsets);
    assert!(
        hir.series_history
            .iter()
            .any(|requirement| requirement.max_constant_offset == 2)
    );
}

#[test]
fn accepts_ta_trend_window_calls() {
    let analysis = analyze("plot(ta.rising(close, 2) ? 1 : ta.falling(open, 2) ? -1 : 0)\n");

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
            .any(|feature| feature.feature == "ta.rising")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.falling")
    );
}

#[test]
fn accepts_ta_extreme_bar_offsets() {
    let analysis = analyze(
        "plot(ta.highest(3) + ta.lowest(3) + ta.highestbars(close, 3) + ta.lowestbars(open, 3) + ta.highestbars(3) + ta.lowestbars(length=3))\n",
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
            .any(|feature| feature.feature == "ta.highest")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.lowest")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.highestbars")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.lowestbars")
    );
}

#[test]
fn accepts_ta_barssince() {
    let analysis = analyze("plot(ta.barssince(close > open))\n");

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
            .any(|feature| feature.feature == "ta.barssince")
    );
}

#[test]
fn accepts_ta_valuewhen() {
    let analysis = analyze(
        "price = ta.valuewhen(close > open, close, 0)\nflag = ta.valuewhen(close > open, close > high, 1)\nshade = ta.valuewhen(close > open, color.red, 0)\nplot(price + (flag ? 1 : 0) + (shade == color.red ? 1 : 0))\n",
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
            .any(|feature| feature.feature == "ta.valuewhen")
    );
}

#[test]
fn accepts_ta_cum() {
    let analysis = analyze("plot(ta.cum(close) + ta.cum(bar_index))\n");

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
            .any(|feature| feature.feature == "ta.cum")
    );
}

#[test]
fn accepts_ta_all_time_extremes() {
    let analysis = analyze("plot(ta.max(close) + ta.min(open) + ta.max(bar_index))\n");

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
            .any(|feature| feature.feature == "ta.max")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.min")
    );
}

#[test]
fn accepts_ta_dema_tema() {
    let analysis = analyze("plot(ta.dema(close, 3) + ta.tema(bar_index, 3))\n");

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
            .any(|feature| feature.feature == "ta.dema")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.tema")
    );
}

#[test]
fn accepts_ta_volume_flow_variables() {
    let analysis = analyze(
        "plot(ta.accdist + ta.iii + ta.nvi + ta.obv + ta.pvi + ta.pvt + ta.vwap + ta.vwap(close) + ta.vwap(close, bar_index == 1) + ta.wad + ta.wvad)\n",
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
            .any(|feature| feature.feature == "ta.accdist")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.iii")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.nvi")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.obv")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.pvi")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.pvt")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.vwap")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.wad")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.wvad")
    );
}

#[test]
fn accepts_ta_vwap_bands_tuple_overload() {
    let analysis = analyze(
        "[basis, upper, lower] = ta.vwap(close, bar_index == 1, 2.0)\nplot(basis + upper + lower)\n",
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
            .any(|feature| feature.feature == "ta.vwap")
    );
}

#[test]
fn accepts_ta_pivot_point_levels() {
    let analysis = analyze(
        "levels = ta.pivot_point_levels(\"Traditional\", bar_index == 2, true)\nplot(array.get(levels, 0))\n",
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
            .any(|feature| feature.feature == "ta.pivot_point_levels")
    );
}

#[test]
fn accepts_ta_supertrend() {
    let analysis = analyze("[line, direction] = ta.supertrend(2.0, 3)\nplot(line + direction)\n");

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
            .any(|feature| feature.feature == "ta.supertrend")
    );
}

#[test]
fn accepts_ta_dmi() {
    let analysis = analyze("[plus, minus, adx] = ta.dmi(3, 2)\nplot(plus + minus + adx)\n");

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
            .any(|feature| feature.feature == "ta.dmi")
    );
}

#[test]
fn accepts_ta_stoch() {
    let analysis = analyze("plot(ta.stoch(close, high, low, 3))\n");

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
            .any(|feature| feature.feature == "ta.stoch")
    );
}

#[test]
fn accepts_ta_sar() {
    let analysis = analyze("plot(ta.sar(0.02, 0.02, 0.2))\n");

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
            .any(|feature| feature.feature == "ta.sar")
    );
}

#[test]
fn accepts_ta_wpr() {
    let analysis = analyze("plot(ta.wpr(3))\n");

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
            .any(|feature| feature.feature == "ta.wpr")
    );
}

#[test]
fn accepts_ta_ao() {
    let analysis = analyze("plot(ta.ao())\n");

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
            .any(|feature| feature.feature == "ta.ao")
    );
}

#[test]
fn accepts_ta_bop() {
    let analysis = analyze("plot(ta.bop())\n");

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
            .any(|feature| feature.feature == "ta.bop")
    );
}

#[test]
fn accepts_ta_cci() {
    let analysis = analyze("plot(ta.cci(close, 3))\n");

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
            .any(|feature| feature.feature == "ta.cci")
    );
}

#[test]
fn accepts_ta_cog() {
    let analysis = analyze("plot(ta.cog(close, 3))\n");

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
            .any(|feature| feature.feature == "ta.cog")
    );
}

#[test]
fn accepts_ta_kc_and_kcw() {
    let analysis = analyze(
        "[middle, upper, lower] = ta.kc(close, 3, 2, false)\nplot(middle + upper + lower + ta.kcw(close, 3, 2))\n",
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
            .any(|feature| feature.feature == "ta.kc")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.kcw")
    );
}

#[test]
fn accepts_ta_pivots() {
    let analysis = analyze(
        "plot(ta.pivothigh(close, 2, 1) + ta.pivotlow(low, 2, 1) + ta.pivothigh(2, 1) + ta.pivotlow(leftbars=2, rightbars=1))\n",
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
            .any(|feature| feature.feature == "ta.pivothigh")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "ta.pivotlow")
    );
}

#[test]
fn accepts_ta_mfi() {
    let analysis = analyze("plot(ta.mfi(hlc3, 3))\n");

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
            .any(|feature| feature.feature == "ta.mfi")
    );
}

#[test]
fn accepts_ta_tsi() {
    let analysis = analyze("plot(ta.tsi(close, 2, 3))\n");

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
            .any(|feature| feature.feature == "ta.tsi")
    );
}

#[test]
fn accepts_ta_cmo() {
    let analysis = analyze("plot(ta.cmo(close, 3))\n");

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
            .any(|feature| feature.feature == "ta.cmo")
    );
}

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
fn rejects_unknown_history_offset() {
    let analysis = analyze("x = close[len]\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "dynamic_history_offset"
    );
}

#[test]
fn rejects_non_int_history_offset() {
    let analysis = analyze("x = close[close]\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "dynamic_history_offset"
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
fn accepts_series_history_offset() {
    let analysis = analyze("x = close[bar_index]\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("HIR");
    assert!(hir.history.has_dynamic_offsets);
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
fn rejects_negative_history_offset() {
    let analysis = analyze("x = close[-1]\n");

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "negative_history_offset"
    );
}

#[test]
fn accepts_constant_history_offset() {
    let analysis = analyze("x = close[1]\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
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

#[test]
fn rejects_wrong_builtin_argument_type() {
    let analysis = analyze("plot(ta.sma(close, close))\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_TYPE")
    );
}

#[test]
fn rejects_missing_builtin_argument() {
    let analysis = analyze("plot()\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARITY")
    );
}

#[test]
fn rejects_unknown_named_argument() {
    let analysis = analyze("indicator(\"Demo\", bogus=true)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CALL_ARG_NAME")
    );
}

#[test]
fn accepts_named_colors_and_color_new() {
    let analysis = analyze(
        r#"indicator("colors")
base = input.color(color.orange, "Base")
shade = color.new(base, 50)
opaque = color.new(color.blue)
custom = color.rgb(255, 153, 0, 50)
gradient = color.from_gradient(close, 1, 3, color.red, color.green)
hex = #ff990080
channels = color.r(custom) + color.g(custom) + color.b(custom) + color.t(custom)
hex_channels = color.r(hex) + color.g(hex) + color.b(hex) + color.t(hex)
plot(close, color=gradient)
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
            .any(|feature| feature.feature == "color.new")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.rgb")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.r")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.g")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.b")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.t")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.from_gradient")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "hex color literal")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "color.orange")
    );
}

#[test]
fn rejects_unknown_named_color() {
    let analysis = analyze("plot(close, color=color.not_registered)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_UNKNOWN_COLOR")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_string_helpers() {
    let analysis = analyze(
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

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "str.upper",
        "str.lower",
        "str.length",
        "str.contains",
        "str.startswith",
        "str.endswith",
        "str.pos",
        "str.substring",
        "str.trim",
        "str.repeat",
        "str.replace",
        "str.replace_all",
        "str.tonumber",
        "str.tostring",
        "str.format",
        "str.match",
        "str.split",
        "str.format_time",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} not reported as supported"
        );
    }
}

#[test]
fn accepts_time_helpers() {
    let analysis = analyze(
        r#"indicator("time helpers")
ts = timestamp(2021, 2, 2, 3, 4, 5)
plot(year(ts) + month(ts, "UTC") + weekofyear(ts) + dayofmonth(ts) + dayofweek(ts) + hour(ts) + minute(ts) + second(ts) + (dayofweek == dayofweek.friday ? 1 : 0))
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "timestamp",
        "year",
        "month",
        "weekofyear",
        "dayofmonth",
        "dayofweek",
        "dayofweek.friday",
        "hour",
        "minute",
        "second",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} not reported as supported"
        );
    }
}

#[test]
fn accepts_timeframe_helpers() {
    let analysis = analyze(
        r#"indicator("timeframe helpers")
tf = input.timeframe("60", "TF")
seconds = timeframe.in_seconds() + timeframe.in_seconds(tf) + timeframe.in_seconds("D")
roundtrip = timeframe.from_seconds(timeframe.in_seconds(tf)) == tf
tf_change = timeframe.change("D")
is_one_minute = timeframe.isminutes and timeframe.isintraday and not timeframe.isseconds and not timeframe.isdaily and not timeframe.isweekly and not timeframe.ismonthly and not timeframe.isdwm and timeframe.multiplier == 1
plot(timeframe.period == "1" and is_one_minute and roundtrip and tf_change ? seconds : 0)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "timeframe.in_seconds",
        "timeframe.from_seconds",
        "timeframe.change",
        "timeframe.period",
        "timeframe.isseconds",
        "timeframe.isminutes",
        "timeframe.isintraday",
        "timeframe.isdaily",
        "timeframe.isweekly",
        "timeframe.ismonthly",
        "timeframe.isdwm",
        "timeframe.multiplier",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} not reported as supported"
        );
    }
}

#[test]
fn accepts_barstate_isfirst() {
    let analysis = analyze(
        "plot((barstate.isfirst or barstate.islast or barstate.isnew or barstate.isconfirmed or barstate.ishistory or barstate.isrealtime or session.ismarket or session.ispremarket or session.ispostmarket) ? 1 : 0)\n",
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
            .any(|feature| feature.feature == "barstate.isfirst")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barstate.islast")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barstate.isnew")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barstate.isconfirmed")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barstate.ishistory")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "barstate.isrealtime")
    );
    for feature in [
        "session.ismarket",
        "session.ispremarket",
        "session.ispostmarket",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} not reported as supported"
        );
    }
}

#[test]
fn accepts_global_price_and_derived_series() {
    let analysis = analyze(
        "plot(open + high + low + close + volume + time + time_close + hl2 + hlc3 + hlcc4 + ohlc4 + bar_index)\n",
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
fn accepts_selected_math_functions() {
    let analysis = analyze(
        r#"indicator("math")
x = math.max(math.abs(close - 3), math.round(close / 2), 1)
y = math.min(x, 3.5)
avg_value = math.avg(open, close, high, low)
rounded_precision = math.round(close / 3, 2)
z = math.floor(close / 2) + math.ceil(close / 2)
w = math.trunc(close / 2) + math.sqrt(close) + math.cbrt(close) + math.log(close) + math.pow(close, 2) + math.hypot(close, high)
random_value = math.random(10, 20, 7)
scale = math.log10(close) + math.exp(close)
trig = math.sin(close) + math.cos(close) + math.tan(close)
inverse_trig = math.acos(close - 2) + math.asin(close - 2) + math.atan(close)
angle_helpers = math.sign(close - 2) + math.todegrees(close) + math.toradians(close)
constants = math.pi + math.e + math.phi + math.rphi
rounded_mintick = math.round_to_mintick(close + 0.006)
mintick = syminfo.mintick
sum_value = math.sum(close, 3)
plot(y)
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
            .any(|feature| feature.feature == "math.max")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.min")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.avg")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.round")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.random")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.floor")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.ceil")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.trunc")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.sqrt")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.cbrt")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.log")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.log10")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.exp")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.acos")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.asin")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.atan")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.sign")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.todegrees")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.toradians")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.pi")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.e")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.phi")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.rphi")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.pow")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.hypot")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.sin")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.cos")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.tan")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.sum")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "math.round_to_mintick")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "syminfo.mintick")
    );
}

#[test]
fn accepts_syminfo_metadata() {
    let analysis = analyze(
        r#"indicator("syminfo")
identity = syminfo.tickerid == "NASDAQ:AAPL" and syminfo.ticker == "AAPL" and syminfo.prefix == "NASDAQ"
details = syminfo.description == "Apple Inc." and syminfo.type == "stock" and syminfo.currency == "USD" and syminfo.basecurrency == "USD"
session = syminfo.session == "regular" and syminfo.timezone == "Etc/UTC" and syminfo.root == "AAPL" and syminfo.volumetype == "base"
scale = syminfo.mintick + syminfo.pointvalue + syminfo.minmove + syminfo.pricescale
plot(identity and details and session ? scale : 0)
"#,
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "syminfo.tickerid",
        "syminfo.ticker",
        "syminfo.prefix",
        "syminfo.description",
        "syminfo.type",
        "syminfo.currency",
        "syminfo.basecurrency",
        "syminfo.session",
        "syminfo.timezone",
        "syminfo.root",
        "syminfo.volumetype",
        "syminfo.mintick",
        "syminfo.pointvalue",
        "syminfo.minmove",
        "syminfo.pricescale",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} not reported as supported"
        );
    }
}

#[test]
fn accepts_if_tuple_declaration_shadowing_outer_symbols() {
    let analysis =
        analyze("x = close\ny = close\nif close > open\n    [x, y] = [high, low]\nplot(x)\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_if_branch_assignment_type_mismatch() {
    let analysis = analyze("x = close\nif close > open\n    x := true\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_ASSIGN_TYPE")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn rejects_non_bool_while_condition() {
    let analysis = analyze("while close\n    plot(close)\n");

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_CONDITION_TYPE"),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_float_array_operations() {
    let analysis = analyze(
        "values = array.new_float(2, close)\narray.push(values, high)\narray.set(values, 0, low)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first + last + array.size(values))\n",
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
            .any(|feature| feature.feature == "array.new_float")
    );
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|feature| feature.feature == "array.size")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_float_array_method_calls() {
    let analysis = analyze(
        "values = array.new_float(2, close)\nvalues.push(high)\nvalues.set(0, low)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first + last + values.size())\n",
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
            .any(|feature| feature.feature == "array.push")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_int_array_operations() {
    let analysis = analyze(
        "values = array.new_int(2, bar_index)\narray.push(values, 10)\narray.set(values, 0, 3)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first + last + array.size(values))\n",
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
            .any(|feature| feature.feature == "array.new_int")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_int_array_method_calls() {
    let analysis = analyze(
        "values = array.new_int(2, bar_index)\nvalues.push(10)\nvalues.set(0, 3)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first + last + values.size())\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_bool_array_operations() {
    let analysis = analyze(
        "values = array.new_bool(2, close > open)\narray.push(values, true)\narray.set(values, 0, false)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot((first or last) ? 1 : array.size(values))\n",
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
            .any(|feature| feature.feature == "array.new_bool")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_bool_array_method_calls() {
    let analysis = analyze(
        "values = array.new_bool(2, close > open)\nvalues.push(true)\nvalues.set(0, false)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot((first or last) ? 1 : values.size())\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_string_array_operations() {
    let analysis = analyze(
        "values = array.new_string(2, \"seed\")\narray.push(values, \"tail\")\narray.set(values, 0, \"head\")\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first == \"head\" and last == \"tail\" ? array.size(values) : 0)\n",
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
            .any(|feature| feature.feature == "array.new_string")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_string_array_method_calls() {
    let analysis = analyze(
        "values = array.new_string(2, \"seed\")\nvalues.push(\"tail\")\nvalues.set(0, \"head\")\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first == \"head\" and last == \"tail\" ? values.size() : 0)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_color_array_operations() {
    let analysis = analyze(
        "values = array.new_color(2, color.red)\narray.push(values, color.green)\narray.set(values, 0, color.blue)\nfirst = array.get(values, 0)\nlast = array.pop(values)\narray.clear(values)\nplot(first == color.blue and last == color.green ? array.size(values) : 0)\n",
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
            .any(|feature| feature.feature == "array.new_color")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_color_array_method_calls() {
    let analysis = analyze(
        "values = array.new_color(2, color.red)\nvalues.push(color.green)\nvalues.set(0, color.blue)\nfirst = values.get(0)\nlast = values.pop()\nvalues.clear()\nplot(first == color.blue and last == color.green ? values.size() : 0)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_helper_operations() {
    let analysis = analyze(
        "values = array.new_int()\narray.unshift(values, 2)\narray.unshift(values, 1)\nfirst = array.first(values)\nlast = array.last(values)\nshifted = array.shift(values)\nplot(first + last + shifted + array.size(values))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in ["array.unshift", "array.first", "array.last", "array.shift"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_insert_remove_operations() {
    let analysis = analyze(
        "values = array.new_int()\nvalues.push(1)\narray.insert(values, 1, 2)\nvalues.insert(-1, 3)\nremoved = values.remove(-2)\nplot(removed + values.get(-1))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in ["array.insert", "array.remove"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_fill_operations() {
    let analysis = analyze(
        "values = array.new_string(3, \"a\")\narray.fill(values, \"b\", 1, 3)\nints = array.new_int(2, 1)\nints.fill(2)\nplot(values.get(1) == \"b\" and ints.get(0) == 2 ? 1 : 0)\n",
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
            .any(|supported| supported.feature == "array.fill"),
        "{:?}",
        analysis.compatibility.supported
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_from_operations() {
    let analysis = analyze(
        "ints = array.from(1, 2, 3)\nfloats = array.from(1, close, na)\nflags = array.from(true, false)\nwords = array.from(\"a\", \"b\")\ncolors = array.from(color.red, color.green)\nplot(ints.sum() + floats.avg() + (flags.get(0) ? 1 : 0) + (words.join(\"|\") == \"a|b\" ? 1 : 0) + (colors.get(0) == color.red ? 1 : 0))\n",
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
            .any(|supported| supported.feature == "array.from"),
        "{:?}",
        analysis.compatibility.supported
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_helper_method_calls() {
    let analysis = analyze(
        "values = array.new_string()\nvalues.unshift(\"tail\")\nvalues.unshift(\"head\")\nfirst = values.first()\nlast = values.last()\nshifted = values.shift()\nplot(first == \"head\" and last == \"tail\" and shifted == \"head\" ? values.size() : 0)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_copy_operations() {
    let analysis = analyze(
        "source = array.new_int()\nalias = source\ncopy = array.copy(source)\nmethod_copy = source.copy()\narray.push(alias, 1)\narray.push(copy, 2)\nmethod_copy.push(3)\nplot(array.size(source) + array.size(copy) + method_copy.size())\n",
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
            .any(|feature| feature.feature == "array.copy"),
        "{:?}",
        analysis.compatibility.supported
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_search_operations() {
    let analysis = analyze(
        "values = array.new_string()\narray.push(values, \"a\")\narray.push(values, \"b\")\narray.push(values, \"a\")\nhas_a = array.includes(values, \"a\")\nfirst = array.indexof(values, \"a\")\nlast = array.lastindexof(values, \"a\")\nmissing = values.indexof(\"z\")\nnums = array.from(1, 2, 2, 4)\nfound = array.binary_search(nums, 2)\nleft = nums.binary_search_leftmost(3)\nright = nums.binary_search_rightmost(3)\nflags = array.from(true, false)\nplot(has_a and values.includes(\"b\") and flags.some() and not flags.every() ? first + last + missing + found + left + right : 0)\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "array.includes",
        "array.every",
        "array.some",
        "array.indexof",
        "array.lastindexof",
        "array.binary_search",
        "array.binary_search_leftmost",
        "array.binary_search_rightmost",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_numeric_array_statistics() {
    let analysis = analyze(
        "ints = array.new_int()\narray.push(ints, 1)\narray.push(ints, 3)\narray.push(ints, 3)\nabs_ints = ints.abs()\nstandard_ints = ints.standardize()\nfloats = array.new_float()\nfloats.push(close)\nfloats.push(high)\nplot(array.min(ints) + array.max(ints) + array.sum(ints) + ints.range() + ints.median() + array.mode(ints) + ints.percentile_nearest_rank(50) + array.percentile_linear_interpolation(ints, 75) + array.percentrank(ints, 1) + ints.covariance(standard_ints) + ints.variance(false) + array.avg(floats) + floats.max() + array.range(floats) + array.stdev(floats) + array.sum(abs_ints) + standard_ints.get(0))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in [
        "array.min",
        "array.max",
        "array.sum",
        "array.avg",
        "array.range",
        "array.median",
        "array.mode",
        "array.percentile_nearest_rank",
        "array.percentile_linear_interpolation",
        "array.percentrank",
        "array.covariance",
        "array.standardize",
        "array.variance",
        "array.stdev",
        "array.abs",
    ] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_ordering_operations() {
    let analysis = analyze(
        "values = array.new_int()\narray.push(values, 3)\narray.push(values, 1)\nindices = values.sort_indices(order.descending)\narray.sort(values, order.descending)\nvalues.reverse()\nwords = array.from(\"b\", \"a\")\nword_indices = words.sort_indices(order.ascending)\nwords.sort(order.ascending)\nplot(values.get(0) + values.get(1) + indices.get(0) + word_indices.get(0))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in ["array.sort", "array.sort_indices", "array.reverse"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_join_operations() {
    let analysis = analyze(
        "values = array.new_string()\nvalues.push(\"a\")\nvalues.push(\"b\")\ntext = array.join(values, \"|\")\nints = array.new_int()\nints.push(1)\nints.push(2)\nplot(text == \"a|b\" and ints.join() == \"1,2\" ? 1 : 0)\n",
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
            .any(|supported| supported.feature == "array.join"),
        "{:?}",
        analysis.compatibility.supported
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_array_slice_concat_operations() {
    let analysis = analyze(
        "values = array.new_int()\nvalues.push(1)\nvalues.push(2)\nvalues.push(3)\npart = array.slice(values, 1, 3)\nmore = array.new_int()\nmore.push(4)\nreturned = values.concat(more)\nplot(part.size() + array.size(returned) + values.get(3))\n",
    );

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    for feature in ["array.slice", "array.concat"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{feature} missing from supported features: {:?}",
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_float_value_for_int_array_mutation() {
    let analysis = analyze("values = array.new_int()\narray.push(values, close)\nplot(close)\n");

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
fn rejects_numeric_value_for_bool_array_mutation() {
    let analysis = analyze("values = array.new_bool()\narray.push(values, close)\nplot(close)\n");

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
fn rejects_numeric_value_for_bool_array_unshift() {
    let analysis =
        analyze("values = array.new_bool()\narray.unshift(values, close)\nplot(close)\n");

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
fn rejects_numeric_value_for_bool_array_insert() {
    let analysis =
        analyze("values = array.new_bool()\narray.insert(values, 0, close)\nplot(close)\n");

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
fn rejects_numeric_value_for_bool_array_fill() {
    let analysis = analyze("values = array.new_bool(2)\narray.fill(values, close)\nplot(close)\n");

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
fn rejects_mixed_array_from_element_kinds() {
    let analysis = analyze("values = array.from(1, \"two\")\nplot(array.size(values))\n");

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
fn rejects_untyped_na_array_from() {
    let analysis = analyze("values = array.from(na, na)\nplot(array.size(values))\n");

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
fn rejects_numeric_value_for_bool_array_search() {
    let analysis = analyze("values = array.new_bool()\nplot(array.indexof(values, close))\n");

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
fn rejects_bool_array_binary_search() {
    let analysis = analyze("values = array.new_bool()\nplot(array.binary_search(values, 1))\n");

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
fn rejects_float_value_for_int_array_binary_search() {
    let analysis = analyze("values = array.new_int()\nplot(values.binary_search(close))\n");

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
fn rejects_string_array_truth_helpers() {
    let analysis = analyze("values = array.new_string()\nplot(array.every(values))\n");

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
fn rejects_bool_array_statistics() {
    let analysis = analyze("values = array.new_bool()\nplot(array.stdev(values))\n");

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
fn rejects_bool_array_sort() {
    let analysis = analyze("values = array.new_bool()\nvalues.push(true)\narray.sort(values)\n");

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
fn rejects_numeric_array_sort_order() {
    let analysis = analyze("values = array.new_int()\narray.sort(values, close)\n");

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
fn rejects_bool_array_sort_indices() {
    let analysis = analyze("values = array.new_bool()\nvalues.push(true)\nvalues.sort_indices()\n");

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
fn rejects_numeric_array_sort_indices_order() {
    let analysis = analyze("values = array.new_int()\narray.sort_indices(values, close)\n");

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
fn rejects_numeric_separator_for_array_join() {
    let analysis = analyze("values = array.new_string()\nplot(array.join(values, close))\n");

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
fn rejects_mismatched_array_concat_kind() {
    let analysis = analyze(
        "ints = array.new_int()\nfloats = array.new_float()\nplot(array.size(array.concat(ints, floats)))\n",
    );

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
fn rejects_series_array_slice_index() {
    let analysis =
        analyze("values = array.new_string()\nplot(array.size(values.slice(0, bar_index)))\n");

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
fn rejects_numeric_value_for_string_array_mutation() {
    let analysis = analyze("values = array.new_string()\narray.push(values, close)\nplot(close)\n");

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
fn rejects_numeric_value_for_color_array_mutation() {
    let analysis = analyze("values = array.new_color()\narray.push(values, close)\nplot(close)\n");

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
fn accepts_array_method_call_on_namespace_like_variable_name() {
    let analysis =
        analyze("strategy = array.new_float()\nstrategy.push(close)\nplot(strategy.size())\n");

    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn rejects_unknown_float_array_method() {
    let analysis = analyze("values = array.new_float()\nvalues.unsupported(close)\nplot(close)\n");

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
fn rejects_unsupported_array_function() {
    let analysis = analyze("values = array.new_line(0)\nplot(close)\n");

    assert!(
        analysis
            .compatibility
            .unsupported
            .iter()
            .any(|feature| feature.feature == "array.new_line"),
        "{:?}",
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}
