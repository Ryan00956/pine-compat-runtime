use std::{fs, path::PathBuf};

use pine_sema::{AnalysisInput, analyze_input, analyze_source};
use pine_syntax::SourceFile;

fn workspace_fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

#[test]
fn reports_unsupported_request_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_request.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(
        analysis.compatibility.unsupported[0].feature,
        "request.security"
    );
}

#[test]
fn reports_unsupported_request_lower_tf_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_lower_tf.pine",
        "request.security_lower_tf",
        "outside the supported request.security subset",
    );
}

#[test]
fn reports_unsupported_request_family_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_family.pine",
        "request.financial",
        "outside the supported request.security subset",
    );
}

#[test]
fn reports_unsupported_request_math_calls_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_math_calls.pine",
        "request.security",
        "same-context request.security",
    );
}

#[test]
fn reports_unsupported_request_merge_options_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_merge_options.pine",
        "request.security",
        "optional gaps/lookahead",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_request_merge_options.pine",
        &["barmerge.gaps_off", "barmerge.lookahead_off"],
    );
}

#[test]
fn accepts_provider_request_context_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/request_security_provider_context.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
    assert!(analysis.compatibility.unsupported.is_empty());
}

#[test]
fn reports_unsupported_ta_sma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_sma_length.pine",
        &["`ta.sma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_ema_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_ema_length.pine",
        &["`ta.ema` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_dema_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_dema_length.pine",
        &["`ta.dema` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_tema_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_tema_length.pine",
        &["`ta.tema` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_rma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_rma_length.pine",
        &["`ta.rma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_rsi_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_rsi_length.pine",
        &["`ta.rsi` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_macd_fastlen_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_macd_fastlen.pine",
        &["`ta.macd` argument `fastlen` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_macd_slowlen_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_macd_slowlen.pine",
        &["`ta.macd` argument `slowlen` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_macd_siglen_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_macd_siglen.pine",
        &["`ta.macd` argument `siglen` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_alma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_alma_length.pine",
        &["`ta.alma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_alma_offset_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_alma_offset.pine",
        &["`ta.alma` argument `offset` does not accept Const Bool"],
    );
}

#[test]
fn reports_unsupported_ta_alma_sigma_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_alma_sigma.pine",
        &["`ta.alma` argument `sigma` does not accept Const Bool"],
    );
}

#[test]
fn reports_unsupported_ta_alma_floor_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_alma_floor.pine",
        &["`ta.alma` argument `floor` does not accept Const Int"],
    );
}

#[test]
fn reports_unsupported_ta_bb_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_bb_length.pine",
        &["`ta.bb` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_bb_mult_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_bb_mult.pine",
        &["`ta.bb` argument `mult` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_bbw_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_bbw_length.pine",
        &["`ta.bbw` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_bbw_mult_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_bbw_mult.pine",
        &["`ta.bbw` argument `mult` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_kc_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kc_length.pine",
        &["`ta.kc` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_kc_mult_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kc_mult.pine",
        &["`ta.kc` argument `mult` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_ta_kc_use_true_range_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kc_use_true_range.pine",
        &["`ta.kc` argument `useTrueRange` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_kcw_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kcw_length.pine",
        &["`ta.kcw` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_kcw_mult_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kcw_mult.pine",
        &["`ta.kcw` argument `mult` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_ta_kcw_use_true_range_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_kcw_use_true_range.pine",
        &["`ta.kcw` argument `useTrueRange` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_dmi_di_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_dmi_di_length.pine",
        &["`ta.dmi` argument `diLength` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_dmi_adx_smoothing_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_dmi_adx_smoothing.pine",
        &["`ta.dmi` argument `adxSmoothing` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_tsi_short_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_tsi_short_length.pine",
        &["`ta.tsi` argument `short_length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_tsi_long_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_tsi_long_length.pine",
        &["`ta.tsi` argument `long_length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_atr_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_atr_length.pine",
        &["`ta.atr` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_cci_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_cci_length.pine",
        &["`ta.cci` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_cmo_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_cmo_length.pine",
        &["`ta.cmo` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_cog_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_cog_length.pine",
        &["`ta.cog` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_dev_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_dev_length.pine",
        &["`ta.dev` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_median_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_median_length.pine",
        &["`ta.median` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_mfi_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_mfi_length.pine",
        &["`ta.mfi` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_mode_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_mode_length.pine",
        &["`ta.mode` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_mom_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_mom_length.pine",
        &["`ta.mom` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_highest_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_highest_length.pine",
        &["`ta.highest` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_highest_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_highest_source.pine",
        &["`ta.highest` argument `source` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_lowest_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_lowest_length.pine",
        &["`ta.lowest` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_lowest_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_lowest_source.pine",
        &["`ta.lowest` argument `source` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_max_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_max_source.pine",
        &["`ta.max` argument `source` does not accept Const Bool"],
    );
}

#[test]
fn reports_unsupported_ta_min_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_min_source.pine",
        &["`ta.min` argument `source` does not accept Const Bool"],
    );
}

#[test]
fn reports_unsupported_ta_highestbars_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_highestbars_length.pine",
        &["`ta.highestbars` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_highestbars_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_highestbars_source.pine",
        &["`ta.highestbars` argument `source` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_lowestbars_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_lowestbars_length.pine",
        &["`ta.lowestbars` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_lowestbars_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_lowestbars_source.pine",
        &["`ta.lowestbars` argument `source` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_falling_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_falling_length.pine",
        &["`ta.falling` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_rising_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_rising_length.pine",
        &["`ta.rising` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_range_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_range_length.pine",
        &["`ta.range` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_roc_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_roc_length.pine",
        &["`ta.roc` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_vwma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_vwma_length.pine",
        &["`ta.vwma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_wma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_wma_length.pine",
        &["`ta.wma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_hma_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_hma_length.pine",
        &["`ta.hma` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_wpr_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_wpr_length.pine",
        &["`ta.wpr` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_correlation_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_correlation_length.pine",
        &["`ta.correlation` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_covariance_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_covariance_length.pine",
        &["`ta.covariance` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_linreg_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_linreg_length.pine",
        &["`ta.linreg` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_linreg_offset_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_linreg_offset.pine",
        &["`ta.linreg` argument `offset` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_percentile_linear_interpolation_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_percentile_linear_interpolation_length.pine",
        &["`ta.percentile_linear_interpolation` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_percentile_linear_interpolation_percentage_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_percentile_linear_interpolation_percentage.pine",
        &[
            "`ta.percentile_linear_interpolation` argument `percentage` does not accept Const String",
        ],
    );
}

#[test]
fn reports_unsupported_ta_percentile_nearest_rank_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_percentile_nearest_rank_length.pine",
        &["`ta.percentile_nearest_rank` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_percentile_nearest_rank_percentage_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_percentile_nearest_rank_percentage.pine",
        &["`ta.percentile_nearest_rank` argument `percentage` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_percentrank_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_percentrank_length.pine",
        &["`ta.percentrank` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_stdev_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_stdev_length.pine",
        &["`ta.stdev` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_stdev_biased_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_stdev_biased.pine",
        &["`ta.stdev` argument `biased` does not accept Const Int"],
    );
}

#[test]
fn reports_unsupported_ta_variance_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_variance_length.pine",
        &["`ta.variance` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_variance_biased_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_variance_biased.pine",
        &["`ta.variance` argument `biased` does not accept Const Int"],
    );
}

#[test]
fn reports_unsupported_ta_stoch_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_stoch_length.pine",
        &["`ta.stoch` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_supertrend_factor_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_supertrend_factor.pine",
        &["`ta.supertrend` argument `factor` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_supertrend_atr_period_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_supertrend_atr_period.pine",
        &["`ta.supertrend` argument `atrPeriod` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_barssince_condition_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_barssince_condition.pine",
        &["`ta.barssince` argument `condition` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_change_length_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_change_length.pine",
        &["`ta.change` argument `length` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_sar_start_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_sar_start.pine",
        &["`ta.sar` argument `start` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_sar_inc_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_sar_inc.pine",
        &["`ta.sar` argument `inc` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_sar_max_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_sar_max.pine",
        &["`ta.sar` argument `max` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_tr_handle_na_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_tr_handle_na.pine",
        &["`ta.tr` argument `handle_na` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_valuewhen_condition_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_valuewhen_condition.pine",
        &["`ta.valuewhen` argument `condition` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_valuewhen_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_valuewhen_source.pine",
        &["`ta.valuewhen` argument `source` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_valuewhen_occurrence_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_valuewhen_occurrence.pine",
        &["`ta.valuewhen` argument `occurrence` does not accept Const Float"],
    );
}

#[test]
fn reports_unsupported_ta_accdist_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_accdist_call.pine",
        &["unknown function `ta.accdist`"],
    );
}

#[test]
fn reports_unsupported_ta_ao_arguments_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_ao_arguments.pine",
        &["`ta.ao` expects at most 0 argument(s), got 1"],
    );
}

#[test]
fn reports_unsupported_ta_bop_arguments_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_bop_arguments.pine",
        &["`ta.bop` expects at most 0 argument(s), got 1"],
    );
}

#[test]
fn reports_unsupported_ta_cum_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_cum_source.pine",
        &["`ta.cum` argument `source` does not accept Const Bool"],
    );
}

#[test]
fn reports_unsupported_ta_cross_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_cross_source.pine",
        &["`ta.cross` argument `source2` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_crossover_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_crossover_source.pine",
        &["`ta.crossover` argument `source2` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_crossunder_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_crossunder_source.pine",
        &["`ta.crossunder` argument `source2` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_ta_iii_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_iii_call.pine",
        &["unknown function `ta.iii`"],
    );
}

#[test]
fn reports_unsupported_ta_nvi_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_nvi_call.pine",
        &["unknown function `ta.nvi`"],
    );
}

#[test]
fn reports_unsupported_ta_obv_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_obv_call.pine",
        &["unknown function `ta.obv`"],
    );
}

#[test]
fn reports_unsupported_ta_pvi_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_pvi_call.pine",
        &["unknown function `ta.pvi`"],
    );
}

#[test]
fn reports_unsupported_ta_pvt_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_pvt_call.pine",
        &["unknown function `ta.pvt`"],
    );
}

#[test]
fn reports_unsupported_ta_wad_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_wad_call.pine",
        &["unknown function `ta.wad`"],
    );
}

#[test]
fn reports_unsupported_ta_wvad_call_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ta_wvad_call.pine",
        &["unknown function `ta.wvad`"],
    );
}

#[test]
fn reports_unsupported_varip_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_varip.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("tuples")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_varip_drawing_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_varip_drawing.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert_eq!(analysis.compatibility.unsupported.len(), 1);
    assert_eq!(analysis.compatibility.unsupported[0].feature, "varip");
    assert!(
        analysis.compatibility.unsupported[0]
            .reason
            .contains("drawing object ids")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_strategy_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn accepts_supported_strategy_declaration_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_declaration.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy"),
        "{} supported features: {:?}",
        path.display(),
        analysis.compatibility.supported
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
}

#[test]
fn accepts_supported_strategy_pyramiding_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_pyramiding.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.pyramiding_limit, 2);
}

#[test]
fn accepts_supported_indicator_max_polylines_count_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_indicator_max_polylines_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("indicator declaration should lower");
    assert_eq!(hir.drawing_settings.max_polylines_count, Some(75));
}

#[test]
fn accepts_supported_indicator_max_lines_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_indicator_max_lines_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("indicator declaration should lower");
    assert_eq!(hir.drawing_settings.max_lines_count, Some(75));
}

#[test]
fn accepts_supported_indicator_max_labels_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_indicator_max_labels_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("indicator declaration should lower");
    assert_eq!(hir.drawing_settings.max_labels_count, Some(75));
}

#[test]
fn accepts_supported_indicator_max_boxes_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_indicator_max_boxes_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("indicator declaration should lower");
    assert_eq!(hir.drawing_settings.max_boxes_count, Some(75));
}

#[test]
fn accepts_supported_strategy_max_polylines_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_max_polylines_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
    assert_eq!(hir.drawing_settings.max_polylines_count, Some(75));
}

#[test]
fn accepts_supported_strategy_max_lines_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_max_lines_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
    assert_eq!(hir.drawing_settings.max_lines_count, Some(75));
}

#[test]
fn accepts_supported_strategy_max_labels_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_max_labels_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
    assert_eq!(hir.drawing_settings.max_labels_count, Some(75));
}

#[test]
fn accepts_supported_strategy_max_boxes_count_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_max_boxes_count.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.script_mode, pine_ir::ScriptMode::Strategy);
    assert_eq!(hir.drawing_settings.max_boxes_count, Some(75));
}

#[test]
fn accepts_supported_strategy_initial_capital_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_initial_capital.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.initial_capital, 2500.0);
}

#[test]
fn accepts_supported_strategy_default_quantity_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_entry_qty(100.0, 10.0),
        Some(3.0)
    );
}

#[test]
fn accepts_supported_strategy_percent_of_equity_default_quantity_fixture() {
    let path = workspace_fixture(
        "tests/fixtures/sema/supported_strategy_percent_of_equity_default_quantity.pine",
    );
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_qty,
        Some(pine_ir::StrategyDefaultQuantity::PercentOfEquity(25.0))
    );
    assert_eq!(
        hir.strategy_settings.default_entry_qty(1000.0, 10.0),
        Some(25.0)
    );
}

#[test]
fn accepts_supported_strategy_cash_default_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_cash_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.default_qty,
        Some(pine_ir::StrategyDefaultQuantity::Cash(100.0))
    );
    assert_eq!(
        hir.strategy_settings.default_entry_qty(1000.0, 10.0),
        Some(10.0)
    );
}

#[test]
fn accepts_supported_strategy_commission_cash_per_contract_fixture() {
    let path = workspace_fixture(
        "tests/fixtures/sema/supported_strategy_commission_cash_per_contract.pine",
    );
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::CashPerContract(0.5))
    );
}

#[test]
fn accepts_supported_strategy_commission_cash_per_order_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_commission_cash_per_order.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::CashPerOrder(1.5))
    );
}

#[test]
fn accepts_supported_strategy_commission_percent_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_commission_percent.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.commission,
        Some(pine_ir::StrategyCommission::Percent(10.0))
    );
}

#[test]
fn accepts_supported_strategy_slippage_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_slippage.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.slippage_ticks, 100.0);
}

#[test]
fn accepts_supported_strategy_limit_verification_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_limit_verification.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(hir.strategy_settings.backtest_fill_limit_ticks, 100.0);
}

#[test]
fn accepts_supported_strategy_margin_declaration_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_margin_declaration.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy declaration should lower");
    assert_eq!(
        hir.strategy_settings.margin_long,
        pine_ir::StrategyMarginSetting::explicit(25.0)
    );
    assert_eq!(
        hir.strategy_settings.margin_short,
        pine_ir::StrategyMarginSetting::explicit(50.0)
    );
}

#[test]
fn reports_strategy_initial_capital_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_initial_capital.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_default_quantity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_default_quantity.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_commission_unknown_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_commission_unknown.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_slippage_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_slippage.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_limit_verification_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_limit_verification.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_margin_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_margin_declaration.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_strategy_declaration_properties_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        "E_CALL_ARG_NAME",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_declaration_properties.pine",
        &[
            "calc_on_order_fills",
            "calc_on_every_tick",
            "process_orders_on_close",
            "currency",
            "close_entries_rule",
            "risk_free_rate",
            "use_bar_magnifier",
            "fill_orders_on_standard_ohlc",
        ],
    );
}

#[test]
fn reports_unsupported_strategy_pyramiding_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_pyramiding.pine",
        &["pyramiding"],
    );
}

#[test]
fn reports_unsupported_indicator_max_polylines_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_indicator_max_polylines_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_indicator_max_polylines_count.pine",
        &["max_polylines_count"],
    );
}

#[test]
fn reports_unsupported_indicator_max_lines_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_indicator_max_lines_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_indicator_max_lines_count.pine",
        &["max_lines_count"],
    );
}

#[test]
fn reports_unsupported_indicator_max_labels_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_indicator_max_labels_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_indicator_max_labels_count.pine",
        &["max_labels_count"],
    );
}

#[test]
fn reports_unsupported_indicator_max_boxes_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_indicator_max_boxes_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_indicator_max_boxes_count.pine",
        &["max_boxes_count"],
    );
}

#[test]
fn reports_unsupported_strategy_max_polylines_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_max_polylines_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_max_polylines_count.pine",
        &["max_polylines_count"],
    );
}

#[test]
fn reports_unsupported_strategy_max_lines_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_max_lines_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_max_lines_count.pine",
        &["max_lines_count"],
    );
}

#[test]
fn reports_unsupported_strategy_max_labels_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_max_labels_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_max_labels_count.pine",
        &["max_labels_count"],
    );
}

#[test]
fn reports_unsupported_strategy_max_boxes_count_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_max_boxes_count.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_max_boxes_count.pine",
        &["max_boxes_count"],
    );
}

#[test]
fn reports_unsupported_strategy_order_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_orders.pine",
        &["strategy.order"],
    );
}

#[test]
fn reports_unsupported_strategy_exit_variant_fixtures() {
    for (path, code) in [
        (
            "tests/fixtures/sema/unsupported_strategy_exit_stop_loss.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_limit_profit.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_three_triggers.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_four_triggers.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_qty_percent_same_side.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_qty_same_side.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_profit_trailing.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_price_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_points_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_offset_only.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trail_price_points.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_bracket.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_indicator.pine",
            "E_STRATEGY_MODE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_trailing_function_side_effect.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_request_strategy_trailing_exit.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_partial_quantity.pine",
            "E_CALL_ARITY",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_missing_entry.pine",
            "E_CALL_ARITY",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_oca_name.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
            "E_CALL_ARG_TYPE",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
            "E_CALL_ARG_NAME",
        ),
        (
            "tests/fixtures/sema/unsupported_strategy_exit_function_side_effect.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
        (
            "tests/fixtures/sema/unsupported_request_strategy_exit.pine",
            "E_UNSUPPORTED_FEATURE",
        ),
    ] {
        assert_diagnostic_fixture(path, code);
    }
}

#[test]
fn reports_strategy_exit_quantity_guardrail_messages() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_qty_percent_same_side.pine",
        &["`strategy.exit` combined trigger families are not supported"],
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_qty_same_side.pine",
        &["`strategy.exit` combined trigger families are not supported"],
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_exit_partial_quantity.pine",
        &["`strategy.exit` requires one of `stop`, `limit`, `profit`, or `loss`"],
    );
}

#[test]
fn accepts_supported_strategy_exit_fixtures() {
    for fixture in [
        "tests/fixtures/sema/supported_strategy_exit_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss.pine",
        "tests/fixtures/sema/supported_strategy_exit_stop_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_stop_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss_limit.pine",
        "tests/fixtures/sema/supported_strategy_exit_loss_profit.pine",
        "tests/fixtures/sema/supported_strategy_exit_trail_price.pine",
        "tests/fixtures/sema/supported_strategy_exit_trail_points.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_trailing.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_loss.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_percent_trailing.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_stop.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_bracket.pine",
        "tests/fixtures/sema/supported_strategy_exit_qty_and_qty_percent_trailing.pine",
        "tests/fixtures/sema/supported_strategy_order_metadata.pine",
    ] {
        let path = workspace_fixture(fixture);
        let text = fs::read_to_string(&path).expect("fixture should be readable");
        let source = SourceFile::new(path.display().to_string(), text);
        let analysis = analyze_source(&source);

        assert!(
            analysis.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == "strategy.exit")
        );
        assert!(analysis.hir.is_some());
    }
}

#[test]
fn accepts_supported_strategy_entry_default_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_entry_default_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let hir = analysis.hir.expect("strategy entry should lower");
    assert_eq!(
        hir.strategy_settings.default_entry_qty(100.0, 10.0),
        Some(1.0)
    );
}

#[test]
fn accepts_supported_strategy_entry_fixture() {
    for fixture in [
        "tests/fixtures/sema/supported_strategy_entry.pine",
        "tests/fixtures/sema/supported_strategy_entry_limit.pine",
        "tests/fixtures/sema/supported_strategy_entry_stop.pine",
        "tests/fixtures/sema/supported_strategy_entry_stop_limit.pine",
    ] {
        let path = workspace_fixture(fixture);
        let text = fs::read_to_string(&path).expect("fixture should be readable");
        let source = SourceFile::new(path.display().to_string(), text);
        let analysis = analyze_source(&source);

        assert!(
            analysis.diagnostics.is_empty(),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
        assert!(analysis.compatibility.unsupported.is_empty());
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == "strategy.entry")
        );
        assert!(analysis.hir.is_some());
    }
}

#[test]
fn reports_strategy_entry_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_entry_short_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_short.pine",
        "E_CALL_ARG_VALUE",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_entry_short.pine",
        &["strategy.long"],
    );
}

#[test]
fn reports_strategy_entry_qty_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_entry_qty.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn accepts_supported_strategy_close_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn reports_unsupported_strategy_close_partial_quantity_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/unsupported_strategy_close_partial_quantity.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_NAME"
                && diagnostic
                    .message
                    .contains("partial quantity arguments must be named")
        }),
        "{} diagnostics should reject positional strategy.close quantity: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_VALUE"
                && diagnostic
                    .message
                    .contains("argument `qty` must be finite and positive")
        }),
        "{} diagnostics should reject non-positive strategy.close qty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_VALUE"
                && diagnostic
                    .message
                    .contains("argument `qty_percent` must be finite and positive")
        }),
        "{} diagnostics should reject non-positive strategy.close qty_percent: {:?}",
        path.display(),
        analysis.diagnostics
    );
    let name = "immediately";
    assert!(
        analysis.diagnostics.iter().any(|diagnostic| {
            diagnostic.code == "E_CALL_ARG_NAME" && diagnostic.message.contains(name)
        }),
        "{} diagnostics should reject strategy.close argument `{}`: {:?}",
        path.display(),
        name,
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_strategy_order_metadata_type_guardrails() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_order_metadata_types.pine",
        &[
            "argument `comment` does not accept",
            "argument `disable_alert` does not accept",
            "argument `alert_message` does not accept",
        ],
    );
}

#[test]
fn reports_strategy_close_immediately_remains_unsupported() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_strategy_close_immediately.pine",
        &["immediately"],
    );
}

#[test]
fn accepts_supported_strategy_close_qty_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_qty_percent_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty_percent.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics should be empty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_qty_precedence_fixture() {
    let path =
        workspace_fixture("tests/fixtures/sema/supported_strategy_close_qty_precedence.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics should be empty: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_close_all_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_close_all.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.close_all")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_position_state_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_position_state.pine",
        &[
            "strategy.position_size",
            "strategy.position_avg_price",
            "strategy.max_contracts_held_all",
            "strategy.max_contracts_held_long",
            "strategy.max_contracts_held_short",
        ],
    );
}

#[test]
fn accepts_supported_strategy_profit_state_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_profit_state.pine",
        &[
            "strategy.openprofit",
            "strategy.netprofit",
            "strategy.netprofit_percent",
            "strategy.grossprofit",
            "strategy.grossprofit_percent",
            "strategy.grossloss",
            "strategy.grossloss_percent",
            "strategy.avg_trade",
            "strategy.avg_trade_percent",
            "strategy.avg_winning_trade",
            "strategy.avg_winning_trade_percent",
            "strategy.avg_losing_trade",
            "strategy.avg_losing_trade_percent",
            "strategy.max_runup",
            "strategy.max_runup_percent",
            "strategy.max_drawdown",
            "strategy.max_drawdown_percent",
            "strategy.equity",
        ],
    );
}

#[test]
fn accepts_supported_strategy_variable_interactions_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_variable_interactions.pine",
        &[
            "strategy.position_size",
            "strategy.openprofit",
            "strategy.netprofit",
        ],
    );
}

#[test]
fn accepts_supported_strategy_trade_counts_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_counts.pine",
        &["strategy.closedtrades", "strategy.opentrades"],
    );
}

#[test]
fn accepts_supported_strategy_closedtrades_fields_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_closedtrades_fields.pine",
        &[
            "strategy.closedtrades.entry_price",
            "strategy.closedtrades.entry_id",
            "strategy.closedtrades.exit_price",
            "strategy.closedtrades.exit_id",
            "strategy.closedtrades.entry_bar_index",
            "strategy.closedtrades.exit_bar_index",
            "strategy.closedtrades.entry_time",
            "strategy.closedtrades.exit_time",
            "strategy.closedtrades.commission",
            "strategy.closedtrades.size",
            "strategy.closedtrades.profit",
            "strategy.closedtrades.max_runup",
            "strategy.closedtrades.max_drawdown",
        ],
    );
}

#[test]
fn accepts_supported_strategy_opentrades_fields_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_opentrades_fields.pine",
        &[
            "strategy.opentrades.entry_price",
            "strategy.opentrades.entry_id",
            "strategy.opentrades.entry_bar_index",
            "strategy.opentrades.entry_time",
            "strategy.opentrades.size",
            "strategy.opentrades.profit",
            "strategy.opentrades.commission",
            "strategy.opentrades.max_runup",
            "strategy.opentrades.max_drawdown",
            "strategy.opentrades.capital_held",
        ],
    );
}

#[test]
fn accepts_supported_strategy_trade_count_interactions_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_count_interactions.pine",
        &["strategy.closedtrades", "strategy.opentrades"],
    );
}

#[test]
fn accepts_supported_strategy_trade_outcome_counts_fixture() {
    assert_strategy_state_supported_fixture(
        "tests/fixtures/sema/supported_strategy_trade_outcome_counts.pine",
        &[
            "strategy.wintrades",
            "strategy.losstrades",
            "strategy.eventrades",
        ],
    );
}

#[test]
fn reports_strategy_close_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_close_all_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_all_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn reports_strategy_state_indicator_fixture() {
    assert_strategy_state_mode_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_indicator.pine",
    );
}

#[test]
fn reports_strategy_state_variables_fixture() {
    assert_strategy_state_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_variables.pine",
    );
}

#[test]
fn reports_unknown_strategy_variable_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_unknown_variable.pine",
        &["strategy.future_metric"],
    );
}

#[test]
fn reports_unsupported_strategy_order_and_trade_namespace_fixture() {
    assert_strategy_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_order_and_trade_namespaces.pine",
        &["strategy.risk.max_drawdown"],
    );
}

#[test]
fn reports_strategy_closedtrades_fields_indicator_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_closedtrades_fields_indicator.pine",
        "E_STRATEGY_MODE",
    );
}

#[test]
fn accepts_supported_strategy_cancel_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_cancel.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.cancel")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn accepts_supported_strategy_cancel_all_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_strategy_cancel_all.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(
        analysis
            .compatibility
            .supported
            .iter()
            .any(|supported| supported.feature == "strategy.cancel_all")
    );
    assert!(analysis.hir.is_some());
}

#[test]
fn reports_request_strategy_state_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_request_strategy_state.pine",
        "request.security",
        "same-context request.security",
    );
}

#[test]
fn reports_strategy_state_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_state_mutation.pine",
        "strategy state variable mutation",
        "read-only",
    );
}

#[test]
fn reports_unsupported_strategy_duplicate_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_duplicate_declaration.pine",
        "E_SCRIPT_DECL_DUPLICATE",
    );
}

#[test]
fn reports_unsupported_strategy_local_declaration_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_strategy_local_declaration.pine",
        "E_SCRIPT_DECL_LOCATION",
    );
}

#[test]
fn reports_unsupported_drawing_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_drawing.pine",
        "label.set_text_wrap",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_label_new_modes_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_label_new_modes.pine",
        &["yloc.abovebar", "label.style_label_down", "size.normal"],
    );
}

#[test]
fn reports_unsupported_line_new_modes_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_line_new_modes.pine",
        &["line.style_"],
    );
}

#[test]
fn reports_unsupported_box_new_modes_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_box_new_modes.pine",
        &["text.format_"],
    );
}

#[test]
fn reports_unsupported_box_border_style_arrow_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_box_border_style_arrow.pine",
        &["line.style_solid", "line.style_dotted", "line.style_dashed"],
    );
}

#[test]
fn reports_unsupported_table_new_modes_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_table_new_modes.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_table_cell_text_formatting_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_table_cell_text_formatting.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_table_set_position_values_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_table_set_position_values.pine",
        "E_CALL_ARG_VALUE",
    );
}

#[test]
fn reports_unsupported_table_method_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_table_method.pine",
        "table.set_border_style",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_table_cell_method_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_table_cell_method.pine",
        "table.cell_set_border_color",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_if_condition_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_if_condition.pine",
        &["condition must be bool, got Const String"],
    );
}

#[test]
fn reports_unsupported_switch_condition_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_switch_condition.pine",
        &["condition must be bool, got Const String"],
    );
}

#[test]
fn reports_unsupported_switch_statement_block_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_switch_statement_block.pine",
        "E_PARSE_SWITCH_BLOCK",
    );
}

#[test]
fn reports_unsupported_while_condition_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_while_condition.pine",
        &["condition must be bool, got Const String"],
    );
}

#[test]
fn reports_unsupported_label_method_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_label_method.pine",
        "label.set_text_wrap",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_label_getter_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_label_getter.pine",
        "label.get_style",
        "drawing object",
    );
}

#[test]
fn reports_unsupported_str_tostring_color_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_tostring_color_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_tostring_label_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_tostring_label_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_tostring_chart_point_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_tostring_chart_point_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_tostring_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_tostring_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_tostring_tuple_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_tostring_tuple.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_format_color_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_format_color_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_format_label_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_format_label_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_format_chart_point_array_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_format_chart_point_array.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_format_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_format_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_str_format_tuple_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_str_format_tuple.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_new_float_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_float_initial.pine",
        &["`array.new_float` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_chart_point_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_chart_point_initial.pine",
        &["`array.new<chart.point>` argument `initial_value` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_chart_point_typed_decl_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_chart_point_typed_decl_initial.pine",
        &["cannot initialize `point` of type chart.point with Series Float"],
    );
}

#[test]
fn reports_unsupported_chart_point_array_typed_decl_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_chart_point_array_typed_decl_initial.pine",
        &["cannot initialize `points` of type array<chart.point> with Simple FloatArray"],
    );
}

#[test]
fn reports_unsupported_scalar_typed_decl_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_scalar_typed_decl_initial.pine",
        &["cannot initialize `count` of type int with Const String"],
    );
}

#[test]
fn reports_unsupported_array_typed_decl_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_typed_decl.pine",
        &["typed declaration `array` is not supported"],
    );
}

#[test]
fn reports_unsupported_array_typed_decl_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_typed_decl_initial.pine",
        &["cannot initialize `prices` of type array<float> with Simple StringArray"],
    );
}

#[test]
fn reports_unsupported_array_new_int_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_int_initial.pine",
        &["`array.new_int` argument `initial_value` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_new_bool_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_bool_initial.pine",
        &["`array.new_bool` argument `initial_value` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_new_string_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_string_initial.pine",
        &["`array.new_string` argument `initial_value` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_new_color_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_color_initial.pine",
        &["`array.new_color` argument `initial_value` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_new_line_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_line_initial.pine",
        &["`array.new_line` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_label_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_label_initial.pine",
        &["`array.new_label` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_box_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_box_initial.pine",
        &["`array.new_box` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_table_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_table_initial.pine",
        &["`array.new_table` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_linefill_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_linefill_initial.pine",
        &["`array.new_linefill` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_new_polyline_initial_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_new_polyline_initial.pine",
        &["`array.new_polyline` argument `initial_value` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_box_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_box_cast_source.pine",
        &["`box` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_label_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_label_cast_source.pine",
        &["`label` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_line_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_line_cast_source.pine",
        &["`line` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_linefill_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_linefill_cast_source.pine",
        &["`linefill` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_polyline_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_polyline_cast_source.pine",
        &["`polyline` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_table_cast_source_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_table_cast_source.pine",
        &["`table` argument `x` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_clear_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_clear_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_reverse_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_reverse_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_from_array_argument_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_from_array_argument.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_from_mixed_kinds_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_from_mixed_kinds.pine",
        &["`array.from` arguments must infer one supported array element kind"],
    );
}

#[test]
fn reports_unsupported_array_from_all_na_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_from_all_na.pine",
        &["`array.from` arguments must infer one supported array element kind"],
    );
}

#[test]
fn reports_unsupported_array_abs_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_from_polyline_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_from_polyline.pine",
        &["`array.from` arguments must infer one supported array element kind"],
    );
}

#[test]
fn reports_unsupported_array_insert_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_insert_value.pine",
        &["`array.insert` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_insert_index_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_insert_index.pine",
        &["`array.insert` argument `index` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_set_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_set_value.pine",
        &["`array.set` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_set_index_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_set_index.pine",
        &["`array.set` argument `index` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_get_index_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_get_index.pine",
        &["`array.get` argument `index` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_push_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_push_value.pine",
        &["`array.push` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_remove_index_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_remove_index.pine",
        &["`array.remove` argument `index` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_unshift_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_unshift_value.pine",
        &["`array.unshift` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_fill_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_fill_value.pine",
        &["`array.fill` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_fill_index_from_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_fill_index_from.pine",
        &["`array.fill` argument `index_from` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_fill_index_to_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_fill_index_to.pine",
        &["`array.fill` argument `index_to` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_join_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_join_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_join_separator_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_join_separator.pine",
        &["`array.join` argument `separator` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_slice_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_slice_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_slice_index_from_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_slice_index_from.pine",
        &["`array.slice` argument `index_from` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_slice_index_to_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_slice_index_to.pine",
        &["`array.slice` argument `index_to` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_includes_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_includes_value.pine",
        &["`array.includes` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_indexof_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_indexof_value.pine",
        &["`array.indexof` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_lastindexof_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_lastindexof_value.pine",
        &["`array.lastindexof` argument `value` does not accept Series Float for bool arrays"],
    );
}

#[test]
fn reports_unsupported_array_sort_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_order_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_sort_order.pine",
        &["`array.sort` argument `order` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_sort_indices_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sort_indices_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sort_indices_order_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_sort_indices_order.pine",
        &["`array.sort_indices` argument `order` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_stdev_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_stdev_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_stdev_biased_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_stdev_biased.pine",
        &["`array.stdev` argument `biased` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_variance_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_variance_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_variance_biased_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_variance_biased.pine",
        &["`array.variance` argument `biased` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_every_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_every_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_every_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_some_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_some_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_covariance_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_covariance_id2_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_covariance_id2.pine",
        &["`array.covariance` argument `id2` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_covariance_biased_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_covariance_biased.pine",
        &["`array.covariance` argument `biased` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_percentrank_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentrank_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentrank_index_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_percentrank_index.pine",
        &["`array.percentrank` argument `index` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_linear_interpolation_percentage_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_percentile_linear_interpolation_percentage.pine",
        &[
            "`array.percentile_linear_interpolation` argument `percentage` does not accept Const String",
        ],
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_percentile_nearest_rank_percentage_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_percentile_nearest_rank_percentage.pine",
        &["`array.percentile_nearest_rank` argument `percentage` does not accept Const String"],
    );
}

#[test]
fn reports_unsupported_array_mode_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_mode_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_mode_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_median_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_median_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_range_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_range_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_avg_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_avg_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_sum_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_sum_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_max_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_max_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_min_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_min_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_abs_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_abs_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_binary_search_value.pine",
        &["`array.binary_search` argument `value` does not accept Const String for int arrays"],
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_leftmost_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_binary_search_leftmost_value.pine",
        &[
            "`array.binary_search_leftmost` argument `value` does not accept Const String for int arrays",
        ],
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_binary_search_rightmost_value_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_binary_search_rightmost_value.pine",
        &[
            "`array.binary_search_rightmost` argument `value` does not accept Const String for int arrays",
        ],
    );
}

#[test]
fn reports_unsupported_array_standardize_bool_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_bool.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_string_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_string.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_color_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_color.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_label_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_label.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_line_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_line.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_box_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_box.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_table_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_table.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_linefill_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_linefill.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_polyline_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_polyline.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_standardize_chart_point_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_standardize_chart_point.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_concat_mismatch_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_concat_mismatch.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_array_concat_id2_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_array_concat_id2.pine",
        &["`array.concat` argument `id2` does not accept Series Float"],
    );
}

#[test]
fn reports_unsupported_array_concat_udt_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_array_concat_udt.pine",
        "E_CALL_ARG_TYPE",
    );
}

#[test]
fn reports_import_fixture_missing_host_library() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_import.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "E_IMPORT_MISSING_LIBRARY"
            || diagnostic.code == "E_IMPORT_ALIAS_REQUIRED"
    }));
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_imported_udt_constructor_fixture() {
    assert_import_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_imported_udt_constructor.pine",
        "E_IMPORT_UNKNOWN_EXPORT",
    );
}

#[test]
fn reports_unsupported_imported_method_fixture() {
    assert_import_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_imported_method.pine",
        "E_UNKNOWN_METHOD",
    );
}

#[test]
fn reports_unsupported_library_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_library.pine",
        "library",
        "library declarations",
    );
}

#[test]
fn reports_unsupported_export_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_export.pine",
        "export",
        "export declarations",
    );
}

#[test]
fn reports_unsupported_user_type_field_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_user_type.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.code == "E_UDT_FIELD_TYPE" })
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_user_type_varip_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_type_varip.pine",
        "varip",
        "other value families",
    );
}

#[test]
fn reports_unsupported_user_type_assign_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_assign_identity.pine",
        "E_UDT_ASSIGN_TYPE",
    );
}

#[test]
fn reports_unsupported_user_type_initializer_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_initializer_identity.pine",
        "E_UDT_ASSIGN_TYPE",
    );
}

#[test]
fn reports_unsupported_user_type_nested_field_assign_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_nested_field_assign_identity.pine",
        "E_ASSIGN_TYPE",
    );
}

#[test]
fn reports_unsupported_user_type_nested_constructor_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_nested_constructor_identity.pine",
        "E_UDT_CONSTRUCTOR_ARG",
    );
}

#[test]
fn reports_unsupported_user_type_ternary_branch_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_ternary_branch_identity.pine",
        "E_BRANCH_TYPE",
    );
}

#[test]
fn reports_unsupported_user_type_switch_branch_identity_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_type_switch_branch_identity.pine",
        "E_BRANCH_TYPE",
    );
}

#[test]
fn reports_unsupported_user_type_field_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_type_field_mutation.pine",
        "function_side_effect",
        "mutating user-defined type fields inside user-defined functions or methods",
    );
}

#[test]
fn reports_unsupported_user_type_parameter_field_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_type_parameter_field_mutation.pine",
        "function_side_effect",
        "mutating user-defined type fields inside user-defined functions or methods",
    );
}

#[test]
fn reports_unsupported_user_method_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_user_method.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_RECEIVER_TYPE")
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_user_method_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_method_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_unsupported_user_method_field_mutation_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_user_method_field_mutation.pine",
        "function_side_effect",
        "mutating user-defined type fields inside user-defined functions or methods",
    );
}

#[test]
fn reports_unsupported_user_method_arg_type_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_arg_type.pine",
        "E_METHOD_ARG_TYPE",
    );
}

#[test]
fn reports_unsupported_user_method_duplicate_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_duplicate.pine",
        "E_METHOD_DUPLICATE",
    );
}

#[test]
fn reports_unsupported_user_method_duplicate_param_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_duplicate_param.pine",
        "E_METHOD_PARAM",
    );
}

#[test]
fn reports_unsupported_user_method_missing_receiver_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_missing_receiver.pine",
        "E_METHOD_PARAM",
    );
}

#[test]
fn reports_unsupported_user_method_param_type_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_param_type.pine",
        "E_METHOD_PARAM",
    );
}

#[test]
fn reports_unsupported_user_method_recursive_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_recursive.pine",
        "E_RECURSIVE_METHOD",
    );
}

#[test]
fn reports_unsupported_user_method_unknown_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_user_method_unknown.pine",
        "E_UNKNOWN_METHOD",
    );
}

#[test]
fn reports_non_array_method_fixture_as_receiver_diagnostic() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_non_array_method.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_METHOD_RECEIVER_TYPE"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_none());
}

#[test]
fn reports_unsupported_alert_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_dynamic_frequency_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_dynamic_frequency.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_unknown_frequency_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_unknown_frequency.pine",
        "alert_frequency",
        "alert.freq_all, alert.freq_once_per_bar, and alert.freq_once_per_bar_close",
    );
}

#[test]
fn reports_unsupported_alert_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{close}}`",
    );
}

#[test]
fn reports_unsupported_alertcondition_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alertcondition_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{timenow}}`",
    );
}

#[test]
fn reports_unsupported_alertcondition_title_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alertcondition_title_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{close}}`",
    );
}

#[test]
fn reports_unsupported_alertcondition_unknown_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alertcondition_unknown_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{unknown}}`",
    );
}

#[test]
fn reports_unsupported_alertcondition_plot_placeholder_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alertcondition_plot_placeholder.pine",
        "alert_placeholders",
        "alert placeholder `{{plot_0}}`",
    );
}

#[test]
fn reports_unsupported_log_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_log.pine",
        "log.info",
        "Pine Logs output is not implemented",
    );
}

#[test]
fn reports_unsupported_ticker_constructors_fixture() {
    assert_diagnostic_fixture(
        "tests/fixtures/sema/unsupported_ticker_constructors.pine",
        "E_CALL_ARITY",
    );
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_ticker_constructors.pine",
        &[
            "ticker.new",
            "ticker.modify",
            "ticker.inherit",
            "ticker.renko",
            "ticker.linebreak",
            "ticker.kagi",
            "ticker.pointfigure",
        ],
    );
}

#[test]
fn reports_unsupported_map_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_map.pine",
        "map.put",
        "map collections are not implemented",
    );
}

#[test]
fn reports_unsupported_matrix_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_matrix.pine",
        "matrix.get",
        "matrix collections are not implemented",
    );
}

#[test]
fn reports_unsupported_alertcondition_dynamic_title_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_alertcondition_dynamic_title.pine",
        &["argument `title`", "Input String"],
    );
}

#[test]
fn reports_unsupported_alertcondition_dynamic_message_fixture() {
    assert_diagnostic_messages(
        "tests/fixtures/sema/unsupported_alertcondition_dynamic_message.pine",
        &["argument `message`", "Input String"],
    );
}

#[test]
fn reports_unsupported_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_function_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_unsupported_declaration_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_declaration_function_side_effect.pine",
        "function_side_effect",
        "indicator",
    );
}

#[test]
fn reports_unsupported_array_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_array_function_side_effect.pine",
        "function_side_effect",
        "array mutation",
    );
}

#[test]
fn reports_unsupported_input_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_input_function_side_effect.pine",
        "function_side_effect",
        "input",
    );
}

#[test]
fn reports_unsupported_drawing_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_drawing_function_side_effect.pine",
        "function_side_effect",
        "inside user-defined functions",
    );
}

#[test]
fn reports_unsupported_alert_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_alert_function_side_effect.pine",
        "function_side_effect",
        "alertcondition",
    );
}

#[test]
fn reports_unsupported_imperative_alert_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_imperative_alert_function_side_effect.pine",
        "function_side_effect",
        "alert",
    );
}

#[test]
fn reports_unsupported_strategy_order_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_order_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_strategy_close_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_strategy_close_all_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_close_all_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_strategy_cancel_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_cancel_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_strategy_cancel_all_function_side_effect_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_strategy_cancel_all_function_side_effect.pine",
        "function_side_effect",
        "strategy order calls",
    );
}

#[test]
fn reports_unsupported_dynamic_history_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_dynamic_history.pine",
        "dynamic_history_offset",
        "integer expression",
    );
}

#[test]
fn reports_unsupported_negative_history_fixture() {
    assert_unsupported_fixture(
        "tests/fixtures/sema/unsupported_negative_history.pine",
        "negative_history_offset",
        "non-negative",
    );
}

#[test]
fn reports_unsupported_recursive_function_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/unsupported_recursive_function.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "E_RECURSIVE_FUNCTION"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

#[test]
fn accepts_supported_block_statement_fixture() {
    let path = workspace_fixture("tests/fixtures/sema/supported_block_statements.pine");
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    for feature in ["if", "for"] {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == feature),
            "{} supported features: {:?}",
            path.display(),
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

fn assert_unsupported_fixture(path: &str, feature: &str, reason: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.compatibility.unsupported.iter().any(
            |unsupported| unsupported.feature == feature && unsupported.reason.contains(reason)
        ),
        "{} unsupported features: {:?}",
        path.display(),
        analysis.compatibility.unsupported
    );
    assert!(analysis.hir.is_none());
}

fn assert_diagnostic_fixture(path: &str, code: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

fn assert_diagnostic_messages(path: &str, messages: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    for message in messages {
        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(message)),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
    }
    assert!(analysis.hir.is_none());
}

fn assert_import_diagnostic_fixture(path: &str, code: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let library_path = workspace_fixture("tests/fixtures/libraries/import_udt_lib.pine");
    let library_text =
        fs::read_to_string(&library_path).expect("library fixture should be readable");
    let library_source = SourceFile::new(library_path.display().to_string(), library_text);
    let input = AnalysisInput::with_library_sources(
        source,
        vec![("user/udt/1".to_owned(), library_source)],
    )
    .expect("library fixture input should be valid");
    let analysis = analyze_input(&input);

    assert!(
        analysis
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == code),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

fn assert_strategy_unsupported_fixture(path: &str, features: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    for feature in features {
        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|unsupported| unsupported.feature == *feature
                    && unsupported.reason.contains("broker emulation")),
            "{} unsupported features: {:?}",
            path.display(),
            analysis.compatibility.unsupported
        );
    }
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "E_UNKNOWN_FUNCTION"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}

fn assert_strategy_state_mode_fixture(path: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);
    let variables = [
        "strategy.position_size",
        "strategy.position_avg_price",
        "strategy.openprofit",
        "strategy.netprofit",
        "strategy.equity",
        "strategy.closedtrades",
        "strategy.wintrades",
        "strategy.losstrades",
        "strategy.eventrades",
        "strategy.opentrades",
    ];

    for variable in variables {
        assert!(
            analysis.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "E_STRATEGY_MODE" && diagnostic.message.contains(variable)
            }),
            "{} diagnostics: {:?}",
            path.display(),
            analysis.diagnostics
        );
    }
    assert!(analysis.compatibility.unsupported.is_empty());
    assert!(analysis.hir.is_none());
}

fn assert_strategy_state_supported_fixture(path: &str, variables: &[&str]) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);

    assert!(
        analysis.diagnostics.is_empty(),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.compatibility.unsupported.is_empty());
    for variable in variables {
        assert!(
            analysis
                .compatibility
                .supported
                .iter()
                .any(|supported| supported.feature == *variable),
            "{} supported features: {:?}",
            path.display(),
            analysis.compatibility.supported
        );
    }
    assert!(analysis.hir.is_some());
}

fn assert_strategy_state_unsupported_fixture(path: &str) {
    let path = workspace_fixture(path);
    let text = fs::read_to_string(&path).expect("fixture should be readable");
    let source = SourceFile::new(path.display().to_string(), text);
    let analysis = analyze_source(&source);
    let variables = ["strategy.buy_and_hold_return_percent"];

    for variable in variables {
        assert!(
            analysis
                .compatibility
                .unsupported
                .iter()
                .any(|unsupported| {
                    unsupported.feature == variable
                        && unsupported.reason.contains("broker emulation")
                }),
            "{} unsupported features: {:?}",
            path.display(),
            analysis.compatibility.unsupported
        );
    }
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "E_UNKNOWN_SYMBOL"),
        "{} diagnostics: {:?}",
        path.display(),
        analysis.diagnostics
    );
    assert!(analysis.hir.is_none());
}
