use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatrixEntry {
    pub(crate) feature: String,
    pub(crate) status: String,
    pub(crate) notes: String,
    pub(crate) fixtures: Vec<String>,
}

pub(crate) fn conformance_entries() -> Vec<MatrixEntry> {
    conformance_entries_from_tsv(include_str!("../../../tests/fixtures/conformance.tsv"))
}

pub(crate) fn conformance_entries_from_tsv(text: &str) -> Vec<MatrixEntry> {
    try_conformance_entries_from_tsv(text).expect("invalid conformance metadata")
}

pub(crate) fn try_conformance_entries_from_tsv(text: &str) -> Result<Vec<MatrixEntry>, String> {
    let mut entries = Vec::new();
    let mut features = HashSet::new();

    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        if index == 0 {
            if line != "feature\tstatus\tnotes\tfixtures" {
                return Err("line 1: expected conformance TSV header".to_owned());
            }
            continue;
        }

        let columns: Vec<_> = line.split('\t').collect();
        if columns.len() != 4 {
            return Err(format!(
                "line {line_number}: expected 4 tab-separated columns, found {}",
                columns.len()
            ));
        }

        let feature = columns[0].trim();
        if feature.is_empty() {
            return Err(format!("line {line_number}: feature must be non-empty"));
        }
        if !features.insert(feature.to_owned()) {
            return Err(format!("line {line_number}: duplicate feature `{feature}`"));
        }

        let status = columns[1].trim();
        if !matches!(status, "supported" | "partial" | "unsupported") {
            return Err(format!(
                "line {line_number}: invalid status `{status}` for `{feature}`"
            ));
        }

        let notes = columns[2].trim();
        if notes.is_empty() {
            return Err(format!("line {line_number}: notes must be non-empty"));
        }

        let fixture_column = columns[3].trim();
        if fixture_column.is_empty() {
            return Err(format!(
                "line {line_number}: fixtures must list at least one path for `{feature}`"
            ));
        }
        let fixtures: Vec<_> = fixture_column.split(';').map(str::trim).collect();
        if fixtures.iter().any(|fixture| fixture.is_empty()) {
            return Err(format!(
                "line {line_number}: fixtures must not contain empty paths for `{feature}`"
            ));
        }

        validate_status_fixture_paths(line_number, feature, status, &fixtures)?;
        validate_request_fixture_paths(line_number, feature, status, &fixtures)?;
        validate_array_udt_fixture_paths(line_number, feature, notes, &fixtures)?;

        entries.push(MatrixEntry {
            feature: feature.to_owned(),
            status: status.to_owned(),
            notes: notes.to_owned(),
            fixtures: fixtures.into_iter().map(str::to_owned).collect(),
        });
    }

    Ok(entries)
}

fn validate_array_udt_fixture_paths(
    line_number: usize,
    feature: &str,
    notes: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature.starts_with("array.")
        && contains_ascii_word(notes, "UDT")
        && !fixtures
            .iter()
            .any(|fixture| fixture.to_ascii_lowercase().contains("udt"))
    {
        return Err(format!(
            "line {line_number}: array feature `{feature}` with UDT notes must reference UDT fixture coverage"
        ));
    }
    Ok(())
}

fn contains_ascii_word(text: &str, needle: &str) -> bool {
    text.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
        .any(|word| word.eq_ignore_ascii_case(needle))
}

fn validate_request_fixture_paths(
    line_number: usize,
    feature: &str,
    status: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    if feature.starts_with("request.")
        && matches!(status, "supported" | "partial")
        && !fixtures.iter().any(|fixture| fixture.contains("request"))
    {
        return Err(format!(
            "line {line_number}: {status} request feature `{feature}` must reference request fixture coverage"
        ));
    }
    Ok(())
}

fn validate_status_fixture_paths(
    line_number: usize,
    feature: &str,
    status: &str,
    fixtures: &[&str],
) -> Result<(), String> {
    match status {
        "supported" | "partial" => {
            if !fixtures.iter().any(|fixture| {
                fixture.starts_with("tests/fixtures/runtime/")
                    || fixture.starts_with("tests/fixtures/realtime/")
                    || fixture.starts_with("tests/fixtures/syntax/")
                    || fixture.starts_with("tests/fixtures/request/")
                    || fixture.starts_with("tests/fixtures/sema/supported_")
                    || fixture.starts_with("tests/fixtures/regressions/")
            }) {
                return Err(format!(
                    "line {line_number}: {status} feature `{feature}` must reference runtime, realtime, syntax, supported sema, or regression fixture coverage"
                ));
            }
        }
        "unsupported" => {
            if !fixtures
                .iter()
                .any(|fixture| fixture.starts_with("tests/fixtures/sema/unsupported_"))
            {
                return Err(format!(
                    "line {line_number}: unsupported feature `{feature}` must reference unsupported sema diagnostic fixture coverage"
                ));
            }
        }
        _ => unreachable!("status was validated before fixture rules"),
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn validate_fixture_paths(
    entries: &[MatrixEntry],
    workspace: &std::path::Path,
) -> Result<(), String> {
    for entry in entries {
        for fixture in &entry.fixtures {
            if !workspace.join(fixture).exists() {
                return Err(format!(
                    "{} fixture path should exist for {}",
                    fixture, entry.feature
                ));
            }
        }
    }
    Ok(())
}
