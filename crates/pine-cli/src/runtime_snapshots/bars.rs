pub(crate) fn runtime_fixture_bars_csv(fixture: &str) -> Option<&'static str> {
    match fixture {
        "tests/fixtures/runtime/vwma_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/vwma_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/mfi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/mfi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/accdist_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/accdist_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/iii_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/iii_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/nvi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/nvi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/pvi_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/pvi_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/pvt_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/pvt_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/vwap_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/vwap_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/wad_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/wad_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/wvad_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/wvad_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/correlation_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/correlation_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/covariance_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/covariance_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/median_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/median_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/mode_edge_cases.pine" => Some(include_str!(
            "../../../../tests/fixtures/runtime/mode_edge_cases_bars.csv"
        )),
        "tests/fixtures/runtime/percentile_linear_interpolation_edge_cases.pine" => {
            Some(include_str!(
                "../../../../tests/fixtures/runtime/percentile_linear_interpolation_edge_cases_bars.csv"
            ))
        }
        _ => None,
    }
}
