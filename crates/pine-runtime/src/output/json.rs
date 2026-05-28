use crate::{HistoryRetentionMode, PineValue, RuntimeProfile};

use super::alerts::AlertEvent;
use super::drawings::{BoxOutput, LabelOutput, LineOutput, TableOutput};
use super::model::{
    ColorSeries, FillOutput, HLineOutput, PUBLIC_RUNTIME_SCHEMA_VERSION, PlotArrowSeries,
    PlotBarSeries, PlotCandleSeries, PlotCharSeries, PlotSeries, PlotShapeSeries, RuntimeResult,
};
use super::strategy::StrategyResult;

pub fn public_runtime_result_json(result: &RuntimeResult) -> String {
    let mut output = format!("{{\"schemaVersion\":{},", PUBLIC_RUNTIME_SCHEMA_VERSION);
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
    output.push_str(",\"lines\":");
    output.push_str(&lines_json(&result.lines));
    output.push_str(",\"boxes\":");
    output.push_str(&boxes_json(&result.boxes));
    output.push_str(",\"tables\":");
    output.push_str(&tables_json(&result.tables));
    output.push_str(",\"alerts\":");
    output.push_str(&alerts_json(&result.alerts));
    if let Some(strategy) = &result.strategy {
        output.push_str(",\"strategy\":");
        output.push_str(&strategy_json(strategy));
    }
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
            "\"labelSnapshotCapacity\":{},",
            "\"lines\":{},",
            "\"lineSnapshots\":{},",
            "\"lineCapacity\":{},",
            "\"lineSnapshotCapacity\":{},",
            "\"boxes\":{},",
            "\"boxSnapshots\":{},",
            "\"boxCapacity\":{},",
            "\"boxSnapshotCapacity\":{},",
            "\"tables\":{},",
            "\"tableCells\":{},",
            "\"tableCapacity\":{},",
            "\"tableSnapshotCapacity\":{},",
            "\"tableCellCapacity\":{}",
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
        profile.label_snapshot_capacity,
        profile.lines,
        profile.line_snapshots,
        profile.line_capacity,
        profile.line_snapshot_capacity,
        profile.boxes,
        profile.box_snapshots,
        profile.box_capacity,
        profile.box_snapshot_capacity,
        profile.tables,
        profile.table_cells,
        profile.table_capacity,
        profile.table_snapshot_capacity,
        profile.table_cell_capacity
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

fn lines_json(lines: &[LineOutput]) -> String {
    let mut output = String::from("[");
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", line.id));
        for (snapshot_index, snapshot) in line.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"x1\":");
                output.push_str(&value_json(&snapshot.x1));
                output.push_str(",\"y1\":");
                output.push_str(&value_json(&snapshot.y1));
                output.push_str(",\"x2\":");
                output.push_str(&value_json(&snapshot.x2));
                output.push_str(",\"y2\":");
                output.push_str(&value_json(&snapshot.y2));
                output.push_str(",\"color\":");
                output.push_str(&value_json(&snapshot.color));
                output.push_str(",\"width\":");
                output.push_str(&value_json(&snapshot.width));
                output.push_str(",\"style\":");
                output.push_str(&value_json(&snapshot.style));
                output.push_str(",\"extend\":");
                output.push_str(&value_json(&snapshot.extend));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn boxes_json(boxes: &[BoxOutput]) -> String {
    let mut output = String::from("[");
    for (index, box_output) in boxes.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"snapshots\":[", box_output.id));
        for (snapshot_index, snapshot) in box_output.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"exists\":{}",
                snapshot.bar_index, snapshot.exists
            ));
            if snapshot.exists {
                output.push_str(",\"left\":");
                output.push_str(&value_json(&snapshot.left));
                output.push_str(",\"top\":");
                output.push_str(&value_json(&snapshot.top));
                output.push_str(",\"right\":");
                output.push_str(&value_json(&snapshot.right));
                output.push_str(",\"bottom\":");
                output.push_str(&value_json(&snapshot.bottom));
                output.push_str(",\"bgColor\":");
                output.push_str(&value_json(&snapshot.bg_color));
                output.push_str(",\"borderColor\":");
                output.push_str(&value_json(&snapshot.border_color));
                output.push_str(",\"borderWidth\":");
                output.push_str(&value_json(&snapshot.border_width));
                output.push_str(",\"borderStyle\":");
                output.push_str(&value_json(&snapshot.border_style));
            }
            output.push('}');
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn tables_json(tables: &[TableOutput]) -> String {
    let mut output = String::from("[");
    for (index, table) in tables.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!("{{\"id\":{},\"position\":", table.id));
        output.push_str(&value_json(&table.position));
        output.push_str(&format!(
            ",\"columns\":{},\"rows\":{},\"snapshots\":[",
            table.columns, table.rows
        ));
        for (snapshot_index, snapshot) in table.snapshots.iter().enumerate() {
            if snapshot_index > 0 {
                output.push(',');
            }
            output.push_str(&format!(
                "{{\"barIndex\":{},\"cells\":[",
                snapshot.bar_index
            ));
            for (cell_index, cell) in snapshot.cells.iter().enumerate() {
                if cell_index > 0 {
                    output.push(',');
                }
                output.push_str(&format!(
                    "{{\"column\":{},\"row\":{},\"text\":",
                    cell.column, cell.row
                ));
                output.push_str(&value_json(&cell.text));
                output.push_str(",\"bgColor\":");
                output.push_str(&value_json(&cell.bg_color));
                output.push_str(",\"textColor\":");
                output.push_str(&value_json(&cell.text_color));
                output.push('}');
            }
            output.push_str("]}");
        }
        output.push_str("]}");
    }
    output.push(']');
    output
}

fn alerts_json(alerts: &[AlertEvent]) -> String {
    let mut output = String::from("[");
    for (index, alert) in alerts.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":{},\"barIndex\":{},\"time\":{},\"message\":\"{}\",\"source\":\"{}\"}}",
            alert.id,
            alert.bar_index,
            alert.time,
            json_escape(&alert.message),
            json_escape(&alert.source)
        ));
    }
    output.push(']');
    output
}

fn strategy_json(strategy: &StrategyResult) -> String {
    format!(
        "{{\"orders\":{},\"trades\":{},\"position\":{},\"equity\":{},\"diagnostics\":{}}}",
        strategy_orders_json(&strategy.orders),
        strategy_trades_json(&strategy.trades),
        strategy_position_json(&strategy.position),
        strategy_equity_json(&strategy.equity),
        runtime_diagnostics_json(&strategy.diagnostics)
    )
}

fn strategy_orders_json(orders: &[crate::StrategyOrderEvent]) -> String {
    let mut output = String::from("[");
    for (index, order) in orders.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":\"{}\",\"barIndex\":{},\"time\":{},\"direction\":\"{}\",\"qty\":{},\"price\":{}}}",
            json_escape(&order.id),
            order.bar_index,
            order.time,
            json_escape(&order.direction),
            order.qty,
            order.price
        ));
    }
    output.push(']');
    output
}

fn strategy_trades_json(trades: &[crate::StrategyTrade]) -> String {
    let mut output = String::from("[");
    for (index, trade) in trades.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"id\":\"{}\",\"entryBarIndex\":{},\"exitBarIndex\":{},\"entryTime\":{},\"exitTime\":{},\"entryPrice\":{},\"exitPrice\":{},\"qty\":{},\"profit\":{}}}",
            json_escape(&trade.id),
            trade.entry_bar_index,
            trade.exit_bar_index,
            trade.entry_time,
            trade.exit_time,
            trade.entry_price,
            trade.exit_price,
            trade.qty,
            trade.profit
        ));
    }
    output.push(']');
    output
}

fn strategy_position_json(position: &[crate::StrategyPositionSnapshot]) -> String {
    let mut output = String::from("[");
    for (index, snapshot) in position.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"barIndex\":{},\"size\":{},\"avgPrice\":{}}}",
            snapshot.bar_index,
            snapshot.size,
            option_f64_json(snapshot.avg_price)
        ));
    }
    output.push(']');
    output
}

fn strategy_equity_json(equity: &[crate::StrategyEquitySnapshot]) -> String {
    let mut output = String::from("[");
    for (index, snapshot) in equity.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"barIndex\":{},\"cash\":{},\"marketValue\":{},\"equity\":{},\"netProfit\":{}}}",
            snapshot.bar_index,
            snapshot.cash,
            snapshot.market_value,
            snapshot.equity,
            snapshot.net_profit
        ));
    }
    output.push(']');
    output
}

fn option_f64_json(value: Option<f64>) -> String {
    value.map_or_else(|| "null".to_owned(), |value| value.to_string())
}

fn runtime_diagnostics_json(diagnostics: &[crate::RuntimeDiagnostic]) -> String {
    let mut output = String::from("[");
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push(',');
        }
        output.push_str(&format!(
            "{{\"code\":\"{}\",\"message\":\"{}\"}}",
            json_escape(&diagnostic.code),
            json_escape(&diagnostic.message)
        ));
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
        PineValue::Plot(value)
        | PineValue::HLine(value)
        | PineValue::Label(value)
        | PineValue::Line(value)
        | PineValue::Box(value)
        | PineValue::Table(value) => value.to_string(),
        PineValue::UserType(values) | PineValue::Tuple(values) => {
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
