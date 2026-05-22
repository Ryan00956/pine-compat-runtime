use std::{fs, path::PathBuf};

use pine_runtime::{Bar, HistoryRetentionMode, RuntimeProfile, run_historical_profiled};
use pine_sema::analyze_source;
use pine_syntax::SourceFile;

const PROFILE_BARS: usize = 512;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn profile_fixture(path: &str) -> RuntimeProfile {
    let fixture = workspace_fixture(path);
    let text = fs::read_to_string(&fixture).expect("profile fixture should be readable");
    let source = SourceFile::new(fixture.display().to_string(), text);
    let analysis = analyze_source(&source);
    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        fixture.display(),
        analysis.diagnostics
    );

    let result = run_historical_profiled(
        &analysis.hir.expect("profile fixture should lower to HIR"),
        &profile_bars(PROFILE_BARS),
    )
    .expect("profile fixture should run");

    result.profile
}

fn profile_bars(count: usize) -> Vec<Bar> {
    (0..count)
        .map(|index| {
            let close = 100.0 + (index % 37) as f64 + index as f64 * 0.125;
            Bar {
                time: index as i64 * 60_000,
                open: close - 0.25,
                high: close + 1.0,
                low: close - 1.0,
                close,
                volume: 1_000.0 + index as f64,
            }
        })
        .collect()
}

fn assert_capacity_within(
    metric: &str,
    capacity: usize,
    values: usize,
    max_multiplier: usize,
    slack: usize,
) {
    assert!(
        capacity <= values * max_multiplier + slack,
        "{metric} capacity {capacity} should stay within {max_multiplier}x values {values} plus slack {slack}"
    );
}

fn stateful_slot_count(profile: &RuntimeProfile) -> usize {
    profile.call_state_slots
        + profile.valuewhen_state_slots
        + profile.rolling_window_slots
        + profile.rsi_state_slots
        + profile.macd_state_slots
}

#[test]
fn long_ta_profile_fixture_has_bounded_storage_growth() {
    let profile = profile_fixture("tests/fixtures/profile/long_ta.pine");

    assert_eq!(profile.bars, PROFILE_BARS);
    assert_eq!(profile.plots, 5);
    assert_eq!(profile.plot_values, PROFILE_BARS * 5);
    assert_eq!(
        profile.history_retention_mode,
        HistoryRetentionMode::StaticTrimmed
    );
    assert!(!profile.history_has_dynamic_offsets);
    assert_eq!(profile.max_series_depth, 0);
    assert!(profile.rolling_window_slots >= 1);
    assert!(profile.macd_state_slots >= 1);
    assert_capacity_within("plot", profile.plot_capacity, profile.plot_values, 2, 64);
    assert!(
        profile.rolling_window_value_capacity <= 256,
        "rolling window capacity should stay bounded for long TA profiles: {:?}",
        profile
    );
}

#[test]
fn many_callsite_profile_fixture_records_state_slots() {
    let profile = profile_fixture("tests/fixtures/profile/many_callsites.pine");

    assert_eq!(profile.bars, PROFILE_BARS);
    assert_eq!(profile.plots, 1);
    assert_eq!(profile.plot_values, PROFILE_BARS);
    assert!(
        stateful_slot_count(&profile) >= 8,
        "many-callsite fixture should record state across profile slot maps: {:?}",
        profile
    );
    assert!(profile.rolling_window_slots >= 4);
    assert!(profile.rsi_state_slots >= 1);
    assert!(profile.valuewhen_state_slots >= 1);
    assert_capacity_within("plot", profile.plot_capacity, profile.plot_values, 2, 64);
    assert!(
        profile.rolling_window_value_capacity <= 256,
        "many-callsite rolling window capacity should stay bounded: {:?}",
        profile
    );
}

#[test]
fn array_heavy_profile_fixture_bounds_array_capacity() {
    let profile = profile_fixture("tests/fixtures/profile/array_heavy.pine");

    assert_eq!(profile.bars, PROFILE_BARS);
    assert_eq!(profile.array_slots, 1);
    assert_eq!(profile.array_values, PROFILE_BARS * 3);
    assert_eq!(profile.plots, 1);
    assert_eq!(profile.plot_values, PROFILE_BARS);
    assert_capacity_within(
        "array value",
        profile.array_value_capacity,
        profile.array_values,
        2,
        64,
    );
    assert_capacity_within("plot", profile.plot_capacity, profile.plot_values, 2, 64);
}

#[test]
fn dynamic_history_profile_fixture_respects_max_bars_back() {
    let profile = profile_fixture("tests/fixtures/profile/dynamic_history_max_bars_back.pine");

    assert_eq!(profile.bars, PROFILE_BARS);
    assert_eq!(
        profile.history_retention_mode,
        HistoryRetentionMode::MaxBarsBack
    );
    assert_eq!(profile.history_max_bars_back, Some(32));
    assert!(profile.history_has_dynamic_offsets);
    assert_eq!(profile.max_series_depth, 32);
    assert!(profile.series_buffers > 0);
    assert!(
        profile.series_values <= profile.series_buffers * 32,
        "max_bars_back should bound retained values per series buffer: {:?}",
        profile
    );
    assert_capacity_within(
        "series",
        profile.series_capacity,
        profile.series_values,
        2,
        64,
    );
}
