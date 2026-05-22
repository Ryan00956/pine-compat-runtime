use crate::{HistoryRetentionMode, PineValue, RuntimeProfile};

use super::drawings::LabelOutput;
use super::model::{
    ColorSeries, FillOutput, HLineOutput, PUBLIC_OUTPUT_SCHEMA_VERSION, PlotArrowSeries,
    PlotBarSeries, PlotCandleSeries, PlotCharSeries, PlotSeries, PlotShapeSeries, RuntimeResult,
};

pub fn public_runtime_result_json(result: &RuntimeResult) -> String {
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_OUTPUT_SCHEMA_VERSION);
    output.push_str("\"plots\":");
    output.push_str(&plots_json(&result.plots));
    output.push_str(",\"plotChars\":");
    output.push_str(&plot_chars_json(&result.plot_chars));
    output.push_str(",\"plotShapes\":");
    output.push_str(&plot_shapes_json(&result.plot_shapes));
    output.push_str(",\"plotArrows\":");
    output.push_str(&plot_arrows_json(&result.plot_arrows));
    output.push_str(",\"plotBars\":");
    output.push_str(&plot_bars_json(&result.plot_bars));
    output.push_str(",\"plotCandles\":");
    output.push_str(&plot_candles_json(&result.plot_candles));
    output.push_str(",\"bgColors\":");
    output.push_str(&colors_json(&result.bg_colors));
    output.push_str(",\"barColors\":");
    output.push_str(&colors_json(&result.bar_colors));
    output.push_str(",\"hlines\":");
    output.push_str(&hlines_json(&result.hlines));
    output.push_str(",\"fills\":");
    output.push_str(&fills_json(&result.fills));
    output.push_str(",\"labels\":");
    output.push_str(&labels_json(&result.labels));
    output.push_str(",\"diagnostics\":[]");
    output.push('}');
    output
}

pub fn public_runtime_profiled_result_json(
    result: &RuntimeResult,
    profile: &RuntimeProfile,
) -> String {
    let mut output = public_runtime_result_json(result);
    output.pop();
    output.push_str(",\"profile\":");
    output.push_str(&profile_json(profile));
    output.push('}');
    output
}

fn profile_json(profile: &RuntimeProfile) -> String {
    format!(
        concat!(
            "{{",
            "\"bars\":{},",
            "\"seriesBuffers\":{},",
            "\"seriesValues\":{},",
            "\"seriesCapacity\":{},",
            "\"maxSeriesDepth\":{},",
            "\"historyRetentionMode\":\"{}\",",
            "\"historyMaxConstantOffset\":{},",
            "\"historyMaxBarsBack\":{},",
            "\"historyHasDynamicOffsets\":{},",
            "\"symbolSlots\":{},",
            "\"symbolCapacity\":{},",
            "\"currentSeriesSlots\":{},",
            "\"currentSeriesCapacity\":{},",
            "\"varSlots\":{},",
            "\"varCapacity\":{},",
            "\"arraySlots\":{},",
            "\"arrayCapacity\":{},",
            "\"arrayValues\":{},",
            "\"arrayValueCapacity\":{},",
            "\"callStateSlots\":{},",
            "\"callStateCapacity\":{},",
            "\"valuewhenStateSlots\":{},",
            "\"valuewhenStateCapacity\":{},",
            "\"valuewhenStateValues\":{},",
            "\"valuewhenStateValueCapacity\":{},",
            "\"rollingWindowSlots\":{},",
            "\"rollingWindowCapacity\":{},",
            "\"rollingWindowValues\":{},",
            "\"rollingWindowValueCapacity\":{},",
            "\"rsiStateSlots\":{},",
            "\"rsiStateCapacity\":{},",
            "\"macdStateSlots\":{},",
            "\"macdStateCapacity\":{},",
            "\"plots\":{},",
            "\"plotValues\":{},",
            "\"plotCapacity\":{},",
            "\"plotChars\":{},",
            "\"plotCharValues\":{},",
            "\"plotCharCapacity\":{},",
            "\"plotShapes\":{},",
            "\"plotShapeValues\":{},",
            "\"plotShapeCapacity\":{},",
            "\"plotArrows\":{},",
            "\"plotArrowValues\":{},",
            "\"plotArrowCapacity\":{},",
            "\"plotBars\":{},",
            "\"plotBarValues\":{},",
            "\"plotBarCapacity\":{},",
            "\"plotCandles\":{},",
            "\"plotCandleValues\":{},",
            "\"plotCandleCapacity\":{},",
            "\"bgColors\":{},",
            "\"bgColorValues\":{},",
            "\"bgColorCapacity\":{},",
            "\"barColors\":{},",
            "\"barColorValues\":{},",
            "\"barColorCapacity\":{},",
            "\"hlines\":{},",
            "\"hlineCapacity\":{},",
            "\"fills\":{},",
            "\"fillCapacity\":{},",
            "\"labels\":{},",
            "\"labelSnapshots\":{},",
            "\"labelCapacity\":{},",
            "\"labelSnapshotCapacity\":{}",
            "}}"
        ),
        profile.bars,
        profile.series_buffers,
        profile.series_values,
        profile.series_capacity,
        profile.max_series_depth,
        history_retention_mode_json(profile.history_retention_mode),
        profile.history_max_constant_offset,
        option_u32_json(profile.history_max_bars_back),
        profile.history_has_dynamic_offsets,
        profile.symbol_slots,
        profile.symbol_capacity,
        profile.current_series_slots,
        profile.current_series_capacity,
        profile.var_slots,
        profile.var_capacity,
        profile.array_slots,
        profile.array_capacity,
        profile.array_values,
        profile.array_value_capacity,
        profile.call_state_slots,
        profile.call_state_capacity,
        profile.valuewhen_state_slots,
        profile.valuewhen_state_capacity,
        profile.valuewhen_state_values,
        profile.valuewhen_state_value_capacity,
        profile.rolling_window_slots,
        profile.rolling_window_capacity,
        profile.rolling_window_values,
        profile.rolling_window_value_capacity,
        profile.rsi_state_slots,
        profile.rsi_state_capacity,
        profile.macd_state_slots,
        profile.macd_state_capacity,
        profile.plots,
        profile.plot_values,
        profile.plot_capacity,
        profile.plot_chars,
        profile.plot_char_values,
        profile.plot_char_capacity,
        profile.plot_shapes,
        profile.plot_shape_values,
        profile.plot_shape_capacity,
        profile.plot_arrows,
        profile.plot_arrow_values,
        profile.plot_arrow_capacity,
        profile.plot_bars,
        profile.plot_bar_values,
        profile.plot_bar_capacity,
        profile.plot_candles,
        profile.plot_candle_values,
        profile.plot_candle_capacity,
        profile.bg_colors,
        profile.bg_color_values,
        profile.bg_color_capacity,
        profile.bar_colors,
        profile.bar_color_values,
        profile.bar_color_capacity,
        profile.hlines,
        profile.hline_capacity,
        profile.fills,
        profile.fill_capacity,
        profile.labels,
        profile.label_snapshots,
        profile.label_capacity,
        profile.label_snapshot_capacity
    )
}

fn history_retention_mode_json(mode: HistoryRetentionMode) -> &'static str {
    match mode {
        HistoryRetentionMode::StaticTrimmed => "staticTrimmed",
        HistoryRetentionMode::DynamicFull => "dynamicFull",
        HistoryRetentionMode::MaxBarsBack => "maxBarsBack",
    }
}

fn option_u32_json(value: Option<u32>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn plots_json(plots: &[PlotSeries]) -> String {
    let mut output = String::from("[");
    for (plot_index, plot) in plots.iter().enumerate() {
        if plot_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot.id));
        values_json_into(&mut output, &plot.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn colors_json(colors: &[ColorSeries]) -> String {
    let mut output = String::from("[");
    for (color_index, colors) in colors.iter().enumerate() {
        if color_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", colors.id));
        values_json_into(&mut output, &colors.values);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_chars_json(plot_chars: &[PlotCharSeries]) -> String {
    let mut output = String::from("[");
    for (plot_char_index, plot_char) in plot_chars.iter().enumerate() {
        if plot_char_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_char.id));
        values_json_into(&mut output, &plot_char.values);
        output.push_str("],\"chars\":[");
        values_json_into(&mut output, &plot_char.chars);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_char.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_shapes_json(plot_shapes: &[PlotShapeSeries]) -> String {
    let mut output = String::from("[");
    for (plot_shape_index, plot_shape) in plot_shapes.iter().enumerate() {
        if plot_shape_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_shape.id));
        values_json_into(&mut output, &plot_shape.values);
        output.push_str("],\"styles\":[");
        values_json_into(&mut output, &plot_shape.styles);
        output.push_str("],\"locations\":[");
        values_json_into(&mut output, &plot_shape.locations);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_shape.colors);
        output.push_str("],\"texts\":[");
        values_json_into(&mut output, &plot_shape.texts);
        output.push_str("],\"textColors\":[");
        values_json_into(&mut output, &plot_shape.text_colors);
        output.push_str("],\"sizes\":[");
        values_json_into(&mut output, &plot_shape.sizes);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_arrows_json(plot_arrows: &[PlotArrowSeries]) -> String {
    let mut output = String::from("[");
    for (plot_arrow_index, plot_arrow) in plot_arrows.iter().enumerate() {
        if plot_arrow_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"values\":[", plot_arrow.id));
        values_json_into(&mut output, &plot_arrow.values);
        output.push_str("],\"colorUps\":[");
        values_json_into(&mut output, &plot_arrow.color_ups);
        output.push_str("],\"colorDowns\":[");
        values_json_into(&mut output, &plot_arrow.color_downs);
        output.push_str("],\"minHeights\":[");
        values_json_into(&mut output, &plot_arrow.min_heights);
        output.push_str("],\"maxHeights\":[");
        values_json_into(&mut output, &plot_arrow.max_heights);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_bars_json(plot_bars: &[PlotBarSeries]) -> String {
    let mut output = String::from("[");
    for (plot_bar_index, plot_bar) in plot_bars.iter().enumerate() {
        if plot_bar_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_bar.id));
        values_json_into(&mut output, &plot_bar.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_bar.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_bar.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_bar.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_bar.colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn plot_candles_json(plot_candles: &[PlotCandleSeries]) -> String {
    let mut output = String::from("[");
    for (plot_candle_index, plot_candle) in plot_candles.iter().enumerate() {
        if plot_candle_index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"opens\":[", plot_candle.id));
        values_json_into(&mut output, &plot_candle.opens);
        output.push_str("],\"highs\":[");
        values_json_into(&mut output, &plot_candle.highs);
        output.push_str("],\"lows\":[");
        values_json_into(&mut output, &plot_candle.lows);
        output.push_str("],\"closes\":[");
        values_json_into(&mut output, &plot_candle.closes);
        output.push_str("],\"colors\":[");
        values_json_into(&mut output, &plot_candle.colors);
        output.push_str("],\"wickColors\":[");
        values_json_into(&mut output, &plot_candle.wick_colors);
        output.push_str("],\"borderColors\":[");
        values_json_into(&mut output, &plot_candle.border_colors);
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn values_json_into(output: &mut String, values: &[PineValue]) {
    for (value_index, value) in values.iter().enumerate() {
        if value_index > 0 {
            output.push(',');
        }
        output.push_str(&value_json(value));
    }
}

fn hlines_json(hlines: &[HLineOutput]) -> String {
    let mut output = String::from("[");
    for (index, hline) in hlines.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"price\":{}}}",
            hline.id,
            value_json(&hline.price)
        ));
    }
    output.push(']');
    output
}

fn fills_json(fills: &[FillOutput]) -> String {
    let mut output = String::from("[");
    for (index, fill) in fills.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"firstId\":{},\"secondId\":{}}}",
            fill.id, fill.first_id, fill.second_id
        ));
    }
    output.push(']');
    output
}

fn labels_json(labels: &[LabelOutput]) -> String {
    let mut output = String::from("[");
    for (index, label) in labels.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", label.id));
        for (snapshot_index, snapshot) in label.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"x\":");
                output.push_str(&value_json(&snapshot.x));
                output.push_str(",\"y\":");
                output.push_str(&value_json(&snapshot.y));
                output.push_str(",\"text\":");
                output.push_str(&value_json(&snapshot.text));
                output.push_str(",\"xloc\":");
                output.push_str(&value_json(&snapshot.xloc));
                output.push_str(",\"yloc\":");
                output.push_str(&value_json(&snapshot.yloc));
                output.push_str(",\"color\":");
                output.push_str(&value_json(&snapshot.color));
                output.push_str(",\"style\":");
                output.push_str(&value_json(&snapshot.style));
                output.push_str(",\"textColor\":");
                output.push_str(&value_json(&snapshot.text_color));
                output.push_str(",\"size\":");
                output.push_str(&value_json(&snapshot.size));
                output.push_str(",\"tooltip\":");
                output.push_str(&value_json(&snapshot.tooltip));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn value_json(value: &PineValue) -> String {
    match value {
        PineValue::Int(value) => value.to_string(),
        PineValue::Float(value) => value.to_string(),
        PineValue::Bool(value) => value.to_string(),
        PineValue::String(value) => format!("\"{}\"", json_escape(value)),
        PineValue::Color(value) => value.to_string(),
        PineValue::Plot(value) | PineValue::HLine(value) | PineValue::Label(value) => {
            value.to_string()
        }
        PineValue::Tuple(values) => {
            let mut output = String::from("[");
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&value_json(value));
            }
            output.push(']');
            output
        }
        PineValue::Array(_) | PineValue::Na | PineValue::Void => "null".to_owned(),
    }
}

fn json_escape(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}
