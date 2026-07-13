pub(super) const LOCAL_UDT_ARRAY_CALL_RETURN_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/user_type_array_scalar_tree.pine",
    "tests/fixtures/sema/supported_user_type_array_udf_method_returns.pine",
    "tests/fixtures/sema/unsupported_user_type_array_udf_method_return_identities.pine",
    "tests/fixtures/runtime/user_type_array_tuple_returns.pine",
    "tests/fixtures/sema/supported_user_type_array_tuple_returns.pine",
    "tests/fixtures/sema/unsupported_user_type_array_tuple_return_identities.pine",
    "tests/fixtures/sema/unsupported_user_type_array_tuple_alias_mutation.pine",
    "tests/fixtures/sema/unsupported_local_user_type_array_call_result_chaining.pine",
];

pub(super) const IMPORTED_UDT_ARRAY_CALL_RETURN_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/import_udt_array_udf_method_returns.pine",
    "tests/fixtures/sema/supported_imported_user_type_array_udf_method_returns.pine",
    "tests/fixtures/sema/unsupported_imported_user_type_array_udf_method_return_identities.pine",
    "tests/fixtures/runtime/import_udt_array_tuple_returns.pine",
    "tests/fixtures/sema/supported_imported_user_type_array_tuple_returns.pine",
    "tests/fixtures/sema/unsupported_imported_user_type_array_tuple_return_identities.pine",
    "tests/fixtures/sema/unsupported_imported_user_type_array_tuple_alias_mutation.pine",
    "tests/fixtures/sema/unsupported_imported_user_type_array_call_result_chaining.pine",
    "tests/fixtures/libraries/import_udt_array_return_lib.pine",
];

const UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/user_type_array_scalar_tree.pine",
    "tests/fixtures/sema/supported_user_type_array_udf_method_returns.pine",
    "tests/fixtures/sema/unsupported_local_user_type_array_call_result_chaining.pine",
    "tests/fixtures/runtime/import_udt_array_udf_method_returns.pine",
    "tests/fixtures/sema/supported_imported_user_type_array_udf_method_returns.pine",
    "tests/fixtures/sema/unsupported_imported_user_type_array_call_result_chaining.pine",
    "tests/fixtures/libraries/import_udt_array_return_lib.pine",
];

pub(super) const BUILTIN_ARRAY_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/builtin_array_call_result_reads.pine",
    "tests/fixtures/sema/supported_builtin_array_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_builtin_array_call_result_reads.pine",
];

const BUILTIN_ARRAY_CALL_RESULT_FEATURES: &[&str] = &[
    "array.new_float",
    "array.new_int",
    "array.new_bool",
    "array.new_string",
    "array.new_color",
    "array.new_line",
    "array.new_linefill",
    "array.new_polyline",
    "array.new_label",
    "array.new_box",
    "array.new_table",
    "array.new<chart.point>",
    "array.new<UDT>",
    "array.from",
    "array.size",
    "array.get",
    "array.first",
    "array.last",
    "array.copy",
    "array.abs",
    "array.standardize",
    "array.sort_indices",
    "array.slice",
    "array.concat",
    "array method calls",
    "expression-body functions",
    "multi-statement functions",
    "typed declarations",
    "array.*",
    "import",
    "user-defined types",
    "user-defined methods",
];

const UDT_IDENTITY_BUILTIN_ARRAY_CALL_RESULT_FEATURES: &[&str] = &[
    "array.new<UDT>",
    "array.from",
    "array.copy",
    "array.slice",
    "array.concat",
];

pub(super) const BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/builtin_namespace_array_call_result_reads.pine",
    "tests/fixtures/sema/supported_builtin_namespace_array_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_builtin_namespace_array_call_result_reads.pine",
];

pub(super) const BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/builtin_namespace_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_builtin_namespace_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_builtin_namespace_matrix_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_copy_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_copy_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_copy_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_transpose_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_transpose_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_transpose_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_submatrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_submatrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_submatrix_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_kron_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_kron_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_kron_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_diff_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_diff_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_diff_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_pow_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_pow_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_pow_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_inv_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_inv_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_inv_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_pinv_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_pinv_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_pinv_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_eigenvectors_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_eigenvectors_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_eigenvectors_call_result_reads.pine",
    "tests/fixtures/runtime/bound_matrix_mult_call_result_reads.pine",
    "tests/fixtures/sema/supported_bound_matrix_mult_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_bound_matrix_mult_call_result_reads.pine",
    "tests/fixtures/runtime/local_udf_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_local_udf_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_local_udf_matrix_call_result_reads.pine",
    "tests/fixtures/runtime/user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/runtime/import_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/runtime/import_function_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_function_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_function_matrix_call_result_reads.pine",
    "tests/fixtures/libraries/import_udt_lib.pine",
];

const USER_METHOD_MATRIX_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/runtime/import_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_matrix_call_result_reads.pine",
];

const IMPORTED_USER_METHOD_MATRIX_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/import_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_matrix_call_result_reads.pine",
    "tests/fixtures/libraries/import_udt_lib.pine",
];

const IMPORTED_FUNCTION_MATRIX_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/import_function_matrix_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_function_matrix_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_function_matrix_call_result_reads.pine",
    "tests/fixtures/libraries/import_udt_lib.pine",
];

pub(super) const BUILTIN_MAP_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/builtin_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_builtin_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_builtin_map_call_result_reads.pine",
    "tests/fixtures/runtime/builtin_map_copy_call_result_reads.pine",
    "tests/fixtures/sema/supported_builtin_map_copy_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_builtin_map_copy_call_result_reads.pine",
    "tests/fixtures/runtime/local_udf_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_local_udf_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_local_udf_map_call_result_reads.pine",
    "tests/fixtures/runtime/local_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_local_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_local_user_method_map_call_result_reads.pine",
    "tests/fixtures/runtime/import_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_map_call_result_reads.pine",
    "tests/fixtures/runtime/import_function_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_function_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_function_map_call_result_reads.pine",
];

const USER_METHOD_MAP_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/local_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_local_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_local_user_method_map_call_result_reads.pine",
    "tests/fixtures/runtime/import_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_map_call_result_reads.pine",
];

const IMPORTED_USER_METHOD_MAP_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/import_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_user_method_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_user_method_map_call_result_reads.pine",
    "tests/fixtures/libraries/import_udt_lib.pine",
];

const IMPORTED_FUNCTION_MAP_CALL_RESULT_FIXTURES: &[&str] = &[
    "tests/fixtures/runtime/import_function_map_call_result_reads.pine",
    "tests/fixtures/sema/supported_imported_function_map_call_result_reads.pine",
    "tests/fixtures/sema/unsupported_imported_function_map_call_result_reads.pine",
    "tests/fixtures/libraries/import_udt_lib.pine",
];

const BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FEATURES: &[&str] = &[
    "str.split",
    "ta.pivot_point_levels",
    "array.size",
    "array.get",
    "array.first",
    "array.last",
    "array.copy",
    "array method calls",
    "expression-body functions",
    "multi-statement functions",
    "typed declarations",
    "array.*",
    "map.*",
    "matrix.*",
];

const BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FEATURES: &[&str] = &[
    "expression-body functions",
    "multi-statement functions",
    "typed declarations",
    "matrix.*",
];

const BUILTIN_MAP_CALL_RESULT_FEATURES: &[&str] = &[
    "expression-body functions",
    "multi-statement functions",
    "typed declarations",
    "map.*",
];

pub(super) fn validate_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    validate_local_udt_array_call_return_fixture_paths(line_number, feature, fixtures)?;
    validate_imported_udt_array_call_return_fixture_paths(line_number, feature, fixtures)?;
    validate_builtin_array_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_udt_identity_builtin_array_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_udt_array_call_result_helper_fixture_paths(line_number, feature, fixtures)?;
    validate_builtin_namespace_array_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_builtin_namespace_matrix_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_user_method_matrix_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_imported_user_method_matrix_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_imported_function_matrix_call_result_fixture_paths(line_number, feature, fixtures)
}

pub(super) fn validate_map_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    validate_builtin_map_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_user_method_map_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_imported_user_method_map_call_result_fixture_paths(line_number, feature, fixtures)?;
    validate_imported_function_map_call_result_fixture_paths(line_number, feature, fixtures)
}

fn validate_local_udt_array_call_return_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !matches!(
        feature,
        "array.new<UDT>"
            | "array.from"
            | "array.*"
            | "expression-body functions"
            | "multi-statement functions"
            | "typed declarations"
            | "user-defined types"
            | "user-defined methods"
    ) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        LOCAL_UDT_ARRAY_CALL_RETURN_FIXTURES,
        "fixture-backed local UDF/user-method UDT array return and per-slot tuple-return identity",
    )
}

fn validate_imported_udt_array_call_return_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !matches!(
        feature,
        "array.new<UDT>"
            | "array.from"
            | "array.*"
            | "expression-body functions"
            | "multi-statement functions"
            | "typed declarations"
            | "import"
            | "user-defined types"
            | "user-defined methods"
    ) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        IMPORTED_UDT_ARRAY_CALL_RETURN_FIXTURES,
        "fixture-backed imported UDF/user-method UDT array return and per-slot tuple-return identity plus retained call-result-chaining boundaries",
    )
}

fn validate_udt_array_call_result_helper_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !matches!(
        feature,
        "array.size" | "array.get" | "array.first" | "array.last" | "array.copy"
    ) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES,
        "fixture-backed qualified user-defined and unqualified local-UDF array/scalar-UDT call-result dispatch plus retained fail-closed boundaries",
    )
}

fn validate_builtin_array_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !BUILTIN_ARRAY_CALL_RESULT_FEATURES.contains(&feature) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        BUILTIN_ARRAY_CALL_RESULT_FIXTURES,
        "fixture-backed static-array builtin/template call-result size/get/first/last/copy dispatch and retained producer/helper boundaries",
    )
}

fn validate_udt_identity_builtin_array_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !UDT_IDENTITY_BUILTIN_ARRAY_CALL_RESULT_FEATURES.contains(&feature) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES,
        "fixture-backed local/imported UDT identity through static-array builtin/template call-result dispatch",
    )
}

fn validate_builtin_namespace_array_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FEATURES.contains(&feature) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FIXTURES,
        "fixture-backed non-array-namespace array-capable producer call-result size/get/first/last/copy dispatch and retained result-type/helper boundaries",
    )
}

fn validate_builtin_namespace_matrix_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FEATURES.contains(&feature) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES,
        "fixture-backed exact matrix.new<float|int|bool|string|color> templates, namespace matrix.mult/matrix.copy/matrix.transpose/matrix.submatrix/matrix.kron/matrix.diff/matrix.pow/matrix.inv/matrix.pinv/matrix.eigenvectors results, bound matrix-receiver copy/transpose/submatrix/kron/diff/pow/inv/pinv/eigenvectors/matrix-valued-mult results, and concrete unqualified local-UDF, local/imported user-method, plus registered imported pure-function matrix results with rows/columns/elements_count/get/copy/row/col dispatch, fresh element-kind-preserving row/column arrays, and retained result-type/helper/unregistered-function boundaries",
    )
}

fn validate_user_method_matrix_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "user-defined methods" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        USER_METHOD_MATRIX_CALL_RESULT_FIXTURES,
        "fixture-backed concrete local/imported user-method matrix-result rows/columns/elements_count/get/copy/row/col dispatch, fresh element-kind-preserving row/column arrays, and retained helper/result-type/mutation boundaries",
    )
}

fn validate_imported_user_method_matrix_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "import" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        IMPORTED_USER_METHOD_MATRIX_CALL_RESULT_FIXTURES,
        "fixture-backed imported user-method matrix-result rows/columns/elements_count/get/copy/row/col dispatch with fresh element-kind-preserving row/column arrays, dual-alias isolation, and retained helper/result-type/mutation boundaries",
    )
}

fn validate_imported_function_matrix_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "import" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        IMPORTED_FUNCTION_MATRIX_CALL_RESULT_FIXTURES,
        "fixture-backed imported pure-function matrix-result rows/columns/elements_count/get/copy/row/col dispatch with fresh element-kind-preserving row/column arrays, dual-alias isolation, and retained helper/result-type/mutation boundaries",
    )
}

fn validate_builtin_map_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if !BUILTIN_MAP_CALL_RESULT_FEATURES.contains(&feature) {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        BUILTIN_MAP_CALL_RESULT_FIXTURES,
        "fixture-backed exact scalar map.new template, namespace map.copy result, concrete local/imported user-function and user-method map results with size/get/contains/copy/keys/values dispatch, fresh key/value-kind array reads, and retained template/mutation/terminal-reader boundaries",
    )
}

fn validate_user_method_map_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "user-defined methods" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        USER_METHOD_MAP_CALL_RESULT_FIXTURES,
        "fixture-backed concrete local/imported user-method map-result size/get/contains/copy/keys/values dispatch, fresh key/value-kind array reads, and retained template/mutation/terminal-reader boundaries",
    )
}

fn validate_imported_user_method_map_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "import" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        IMPORTED_USER_METHOD_MAP_CALL_RESULT_FIXTURES,
        "fixture-backed imported user-method map-result size/get/contains/copy/keys/values dispatch with fresh key/value-kind arrays, dual-alias isolation, and retained template/mutation/terminal-reader boundaries",
    )
}

fn validate_imported_function_map_call_result_fixture_paths(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature != "import" {
        return Ok(());
    }

    require_fixtures(
        line_number,
        feature,
        fixtures,
        IMPORTED_FUNCTION_MAP_CALL_RESULT_FIXTURES,
        "fixture-backed imported pure-function map-result size/get/contains/copy/keys/values dispatch with fresh key/value-kind arrays, dual-alias isolation, and retained template/mutation/terminal-reader boundaries",
    )
}

fn require_fixtures(
    line_number: usize,
    feature: &str,
    fixtures: &[&str],
    required: &[&str],
    reason: &str,
) -> Result<(), String> {
    for fixture in required {
        if !fixtures.contains(fixture) {
            return Err(format!(
                "line {line_number}: `{feature}` must reference `{fixture}` for {reason}"
            ));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conformance::try_conformance_entries_from_tsv;

    #[test]
    fn rejects_local_udt_array_return_rows_without_current_fixture_set() {
        let fixtures =
            &LOCAL_UDT_ARRAY_CALL_RETURN_FIXTURES[..LOCAL_UDT_ARRAY_CALL_RETURN_FIXTURES.len() - 1];
        let error = validate_fixture_paths(1, "expression-body functions", fixtures)
            .expect_err("missing local UDT array call-return fixture should fail");

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_local_user_type_array_call_result_chaining.pine"
        ));
    }

    #[test]
    fn import_row_requires_imported_udt_array_return_fixture_set() {
        let error = validate_fixture_paths(1, "import", &[])
            .expect_err("import row must retain the imported UDT array return fixture set");

        assert!(error.contains("tests/fixtures/runtime/import_udt_array_udf_method_returns.pine"));
    }

    #[test]
    fn rejects_udt_array_return_rows_without_imported_call_return_library_fixture() {
        let fixtures = &IMPORTED_UDT_ARRAY_CALL_RETURN_FIXTURES
            [..IMPORTED_UDT_ARRAY_CALL_RETURN_FIXTURES.len() - 1];
        let error = validate_imported_udt_array_call_return_fixture_paths(
            1,
            "user-defined methods",
            fixtures,
        )
        .expect_err("UDT array return rows must retain the imported return library");

        assert!(error.contains("tests/fixtures/libraries/import_udt_array_return_lib.pine"));
    }

    #[test]
    fn rejects_udt_array_helper_rows_without_call_result_fixture_set() {
        let fixtures = &UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES
            [..UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES.len() - 1];
        let error = validate_udt_array_call_result_helper_fixture_paths(1, "array.get", fixtures)
            .expect_err("UDT-array helper rows must retain the call-result fixture set");

        assert!(error.contains("tests/fixtures/libraries/import_udt_array_return_lib.pine"));
    }

    #[test]
    fn rejects_builtin_array_result_rows_without_comprehensive_fixture_set() {
        let fixtures =
            &BUILTIN_ARRAY_CALL_RESULT_FIXTURES[..BUILTIN_ARRAY_CALL_RESULT_FIXTURES.len() - 1];
        let error = validate_builtin_array_call_result_fixture_paths(1, "array.abs", fixtures)
            .expect_err("builtin array-result rows must retain the comprehensive fixture set");

        assert!(
            error.contains("tests/fixtures/sema/unsupported_builtin_array_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_udt_identity_builtin_array_result_rows_without_local_import_fixture_set() {
        let fixtures = &UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES
            [..UDT_ARRAY_CALL_RESULT_HELPER_FIXTURES.len() - 1];
        let error = validate_udt_identity_builtin_array_call_result_fixture_paths(
            1,
            "array.slice",
            fixtures,
        )
        .expect_err("UDT-preserving builtin array-result rows must retain local/import fixtures");

        assert!(error.contains("tests/fixtures/libraries/import_udt_array_return_lib.pine"));
    }

    #[test]
    fn rejects_namespace_array_result_row_without_negative_fixture() {
        let fixtures = &BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FIXTURES
            [..BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FIXTURES.len() - 1];
        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nstr.split\tsupported\tdirect array call-result reads\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("namespace array-result rows must retain the negative boundary fixture");

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_builtin_namespace_array_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_namespace_array_result_row_without_runtime_fixture() {
        let fixtures = &BUILTIN_NAMESPACE_ARRAY_CALL_RESULT_FIXTURES[1..];
        let tsv = format!(
            "feature\tstatus\tnotes\tfixtures\nmatrix.*\tpartial\tdirect array call-result reads\t{}\n",
            fixtures.join(";")
        );
        let error = try_conformance_entries_from_tsv(&tsv)
            .expect_err("namespace array-result rows must retain runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/builtin_namespace_array_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_namespace_matrix_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_builtin_namespace_matrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "namespace matrix-result rows must retain the negative boundary fixture",
                );

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_builtin_namespace_matrix_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_namespace_matrix_result_row_without_runtime_fixture() {
        let fixtures = &BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES[1..];
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", fixtures)
                .expect_err("namespace matrix-result rows must retain runtime evidence");

        assert!(
            error
                .contains("tests/fixtures/runtime/builtin_namespace_matrix_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_bound_matrix_copy_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_copy_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.copy result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_copy_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_copy_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_copy_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.copy result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_copy_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_transpose_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_transpose_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.transpose result rows must retain the negative boundary fixture",
                );

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_bound_matrix_transpose_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_bound_matrix_transpose_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_transpose_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.transpose result rows must retain runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/bound_matrix_transpose_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_bound_matrix_submatrix_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_submatrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.submatrix result rows must retain the negative boundary fixture",
                );

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_bound_matrix_submatrix_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_bound_matrix_submatrix_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_submatrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.submatrix result rows must retain runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/bound_matrix_submatrix_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_bound_matrix_kron_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_kron_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.kron result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_kron_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_kron_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_kron_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.kron result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_kron_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_diff_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_diff_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.diff result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_diff_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_diff_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_diff_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.diff result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_diff_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_pow_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_pow_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.pow result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_pow_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_pow_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_pow_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.pow result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_pow_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_inv_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_inv_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.inv result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_inv_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_inv_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_inv_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.inv result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_inv_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_pinv_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_pinv_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.pinv result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_pinv_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_pinv_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_pinv_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.pinv result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_pinv_call_result_reads.pine"));
    }

    #[test]
    fn rejects_bound_matrix_eigenvectors_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_eigenvectors_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_namespace_matrix_call_result_fixture_paths(
            1, "matrix.*", &fixtures,
        )
        .expect_err(
            "bound matrix.eigenvectors result rows must retain the negative boundary fixture",
        );

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_bound_matrix_eigenvectors_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_bound_matrix_eigenvectors_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/runtime/bound_matrix_eigenvectors_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.eigenvectors result rows must retain runtime evidence");

        assert!(
            error.contains(
                "tests/fixtures/runtime/bound_matrix_eigenvectors_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_mult_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_bound_matrix_mult_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "bound matrix.mult result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_bound_matrix_mult_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_bound_matrix_mult_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/bound_matrix_mult_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("bound matrix.mult result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/bound_matrix_mult_call_result_reads.pine"));
    }

    #[test]
    fn rejects_local_udf_matrix_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_local_udf_matrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err(
                    "local UDF matrix-result rows must retain the negative boundary fixture",
                );

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_local_udf_matrix_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_local_udf_matrix_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_NAMESPACE_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/local_udf_matrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_builtin_namespace_matrix_call_result_fixture_paths(1, "matrix.*", &fixtures)
                .expect_err("local UDF matrix-result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/local_udf_matrix_call_result_reads.pine"));
    }

    #[test]
    fn rejects_user_method_matrix_result_row_without_local_runtime_fixture() {
        let fixtures: Vec<_> = USER_METHOD_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/user_method_matrix_call_result_reads.pine"
            })
            .collect();
        let error = validate_user_method_matrix_call_result_fixture_paths(
            1,
            "user-defined methods",
            &fixtures,
        )
        .expect_err("user-method rows must retain local matrix runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/user_method_matrix_call_result_reads.pine"));
    }

    #[test]
    fn rejects_user_method_matrix_result_row_without_imported_negative_fixture() {
        let fixtures: Vec<_> = USER_METHOD_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_imported_user_method_matrix_call_result_reads.pine"
            })
            .collect();
        let error = validate_user_method_matrix_call_result_fixture_paths(
            1,
            "user-defined methods",
            &fixtures,
        )
        .expect_err("user-method rows must retain imported matrix negative evidence");

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_imported_user_method_matrix_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_import_row_without_imported_user_method_matrix_library_fixture() {
        let fixtures = &IMPORTED_USER_METHOD_MATRIX_CALL_RESULT_FIXTURES
            [..IMPORTED_USER_METHOD_MATRIX_CALL_RESULT_FIXTURES.len() - 1];
        let error =
            validate_imported_user_method_matrix_call_result_fixture_paths(1, "import", fixtures)
                .expect_err("import rows must retain the imported matrix method library");

        assert!(error.contains("tests/fixtures/libraries/import_udt_lib.pine"));
    }

    #[test]
    fn rejects_import_row_without_imported_function_matrix_runtime_fixture() {
        let fixtures: Vec<_> = IMPORTED_FUNCTION_MATRIX_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/import_function_matrix_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_imported_function_matrix_call_result_fixture_paths(1, "import", &fixtures)
                .expect_err("import rows must retain imported function matrix runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/import_function_matrix_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_map_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/sema/unsupported_builtin_map_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("map-result rows must retain the negative boundary fixture");

        assert!(
            error.contains("tests/fixtures/sema/unsupported_builtin_map_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_map_result_row_without_runtime_fixture() {
        let fixtures = &BUILTIN_MAP_CALL_RESULT_FIXTURES[1..];
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", fixtures)
            .expect_err("map-result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/builtin_map_call_result_reads.pine"));
    }

    #[test]
    fn rejects_map_copy_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_builtin_map_copy_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("map.copy result rows must retain the negative boundary fixture");

        assert!(
            error.contains(
                "tests/fixtures/sema/unsupported_builtin_map_copy_call_result_reads.pine"
            )
        );
    }

    #[test]
    fn rejects_map_copy_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/builtin_map_copy_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("map.copy result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/builtin_map_copy_call_result_reads.pine"));
    }

    #[test]
    fn rejects_local_udf_map_result_row_without_negative_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/sema/unsupported_local_udf_map_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("local UDF map-result rows must retain the negative boundary fixture");

        assert!(
            error.contains("tests/fixtures/sema/unsupported_local_udf_map_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_local_udf_map_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/local_udf_map_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("local UDF map-result rows must retain runtime evidence");

        assert!(error.contains("tests/fixtures/runtime/local_udf_map_call_result_reads.pine"));
    }

    #[test]
    fn rejects_local_user_method_map_result_row_without_runtime_fixture() {
        let fixtures: Vec<_> = BUILTIN_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/local_user_method_map_call_result_reads.pine"
            })
            .collect();
        let error = validate_builtin_map_call_result_fixture_paths(1, "map.*", &fixtures)
            .expect_err("local user-method map-result rows must retain runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/local_user_method_map_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_user_method_map_result_row_without_imported_negative_fixture() {
        let fixtures: Vec<_> = USER_METHOD_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture
                    != "tests/fixtures/sema/unsupported_imported_user_method_map_call_result_reads.pine"
            })
            .collect();
        let error = validate_user_method_map_call_result_fixture_paths(
            1,
            "user-defined methods",
            &fixtures,
        )
        .expect_err("user-method rows must retain imported-method negative boundaries");

        assert!(error.contains(
            "tests/fixtures/sema/unsupported_imported_user_method_map_call_result_reads.pine"
        ));
    }

    #[test]
    fn rejects_import_row_without_imported_user_method_map_runtime_fixture() {
        let fixtures: Vec<_> = IMPORTED_USER_METHOD_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/import_user_method_map_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_imported_user_method_map_call_result_fixture_paths(1, "import", &fixtures)
                .expect_err("import rows must retain imported user-method map runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/import_user_method_map_call_result_reads.pine")
        );
    }

    #[test]
    fn rejects_import_row_without_imported_function_map_runtime_fixture() {
        let fixtures: Vec<_> = IMPORTED_FUNCTION_MAP_CALL_RESULT_FIXTURES
            .iter()
            .copied()
            .filter(|fixture| {
                *fixture != "tests/fixtures/runtime/import_function_map_call_result_reads.pine"
            })
            .collect();
        let error =
            validate_imported_function_map_call_result_fixture_paths(1, "import", &fixtures)
                .expect_err("import rows must retain imported function map runtime evidence");

        assert!(
            error.contains("tests/fixtures/runtime/import_function_map_call_result_reads.pine")
        );
    }
}
