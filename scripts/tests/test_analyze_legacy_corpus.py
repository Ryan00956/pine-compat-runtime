from __future__ import annotations

import csv
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import analyze_legacy_corpus  # noqa: E402


def write_manifest(root: Path, rows: list[dict[str, str]]) -> Path:
    path = root / "corpus.tsv"
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=analyze_legacy_corpus.REQUIRED_COLUMNS,
            delimiter="\t",
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(rows)
    return path


def row(
    item_id: str,
    source_path: str,
    *,
    version: str = "4",
    scope: str = "legacy_indicator",
) -> dict[str, str]:
    return {
        "id": item_id,
        "source_path": source_path,
        "declared_or_expected_version": version,
        "chart_bars_path": "bars.csv",
        "chart_symbol": "TEST",
        "chart_timeframe": "1",
        "execution_times_path": "",
        "request_data_manifest": "",
        "reference_output_path": "",
        "license_class": "original",
        "expected_scope": scope,
        "notes": "test fixture",
    }


class FakeRunner:
    def __init__(self) -> None:
        self.commands: list[list[str]] = []

    def __call__(
        self, command: list[str] | tuple[str, ...], root: Path
    ) -> subprocess.CompletedProcess[str]:
        del root
        rendered = list(command)
        self.commands.append(rendered)
        if rendered[1] == "analyze":
            source = Path(rendered[2]).read_text(encoding="utf-8")
            if "legacy_failure" in source:
                return subprocess.CompletedProcess(
                    rendered,
                    1,
                    stdout=(
                        "diagnostics: 1\n"
                        "supported: 0, unsupported: 0\n"
                        "E_UNKNOWN_FUNCTION:Error:2:1: unknown function `study`\n"
                    ),
                    stderr="analysis failed\n",
                )
            return subprocess.CompletedProcess(
                rendered,
                0,
                stdout="diagnostics: 0\nsupported: 1, unsupported: 0\n",
                stderr="",
            )
        provider_backed = "--request-bars" in rendered
        profile = {
            "bars": 1,
            "maxSeriesDepth": 1,
            "requestCacheEntries": 1 if provider_backed else 0,
            "requestCacheContexts": 1 if provider_backed else 0,
            "requestCacheValues": 2 if provider_backed else 0,
            "plotValues": 1,
        }
        return subprocess.CompletedProcess(
            rendered,
            0,
            stdout=json.dumps(
                {"schemaVersion": 8, "plots": [], "profile": profile}
            )
            + "\n",
            stderr="",
        )


class AnalyzeLegacyCorpusTests(unittest.TestCase):
    def test_runtime_host_failures_have_stable_error_kinds(self) -> None:
        self.assertEqual(
            analyze_legacy_corpus.runtime_error_kind(
                "runtime failed: missing request data for symbol `TEST` timeframe `D`"
            ),
            "missing_provider_data",
        )
        self.assertEqual(
            analyze_legacy_corpus.runtime_error_kind(
                "runtime failed: timenow requires an explicit execution timestamp for this script execution"
            ),
            "missing_execution_time",
        )
        self.assertEqual(
            analyze_legacy_corpus.runtime_error_kind(
                "runtime failed: execution timestamp count 1 does not match bar count 2"
            ),
            "execution_time_count_mismatch",
        )
        self.assertEqual(
            analyze_legacy_corpus.runtime_error_kind(
                "invalid execution timestamp `nope` on line 2"
            ),
            "invalid_execution_time_input",
        )
        self.assertEqual(
            analyze_legacy_corpus.runtime_error_kind("runtime failed: other"),
            "runtime_or_host_error",
        )

    def test_detected_version_accepts_whitespace_around_equals_only(self) -> None:
        self.assertEqual(
            analyze_legacy_corpus.detected_version("//@version = 4\n"),
            4,
        )
        self.assertEqual(
            analyze_legacy_corpus.detected_version("//@version\t=\t6\n"),
            6,
        )
        self.assertEqual(
            analyze_legacy_corpus.detected_version("// @version=6\n"),
            1,
        )

    def test_phase_one_diagnostics_have_actionable_categories(self) -> None:
        self.assertEqual(
            analyze_legacy_corpus.feature_category(
                "E_LEGACY_INDICATOR_DECLARATION", None
            ),
            "legacy_declaration",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category(
                "E_LEGACY_STRATEGY_OUT_OF_SCOPE", None
            ),
            "scope_exclusion",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category(
                "E_LANGUAGE_VERSION_UNSUPPORTED", None
            ),
            "version_policy",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "cross"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "round"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "stdev"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "sqrt"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "pivothigh"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "atr"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "floor"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "sum"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "barssince"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "macd"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "log10"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "heikinashi"),
            "ticker_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "tostring"),
            "string_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "cci"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_SYMBOL", "obv"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_SYMBOL", "tr"),
            "ta_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "ceil"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "pow"),
            "math_alias",
        )
        self.assertEqual(
            analyze_legacy_corpus.feature_category("E_UNKNOWN_FUNCTION", "vwap"),
            "ta_alias",
        )

    def test_report_is_deterministic_and_omits_source_text_and_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            private_marker = "PRIVATE_FORMULA_12345"
            (root / "legacy.pine").write_text(
                "//@version=4\nlegacy_failure(" + private_marker + ")\n",
                encoding="utf-8",
            )
            (root / "modern.pine").write_text(
                '//@version=6\nindicator("control")\nplot(close)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(
                root,
                [
                    row("legacy", "legacy.pine"),
                    row(
                        "modern",
                        "modern.pine",
                        version="6",
                        scope="modern_indicator_control",
                    ),
                ],
            )
            runner = FakeRunner()
            rows = analyze_legacy_corpus.parse_manifest(manifest)

            first = analyze_legacy_corpus.build_report(
                rows,
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )
            second = analyze_legacy_corpus.build_report(
                rows,
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )
            first_json = analyze_legacy_corpus.render_report(first)
            second_json = analyze_legacy_corpus.render_report(second)

            self.assertEqual(first_json, second_json)
            self.assertNotIn(private_marker, first_json)
            self.assertNotIn(str(root / "legacy.pine"), first_json)
            self.assertFalse(first["privacy"]["sourceTextIncluded"])
            self.assertFalse(first["privacy"]["sourcePathsIncluded"])
            self.assertFalse(first["privacy"]["timestampsIncluded"])
            self.assertEqual(first["summary"]["eligibleLegacyIndicators"], 1)
            self.assertEqual(
                first["summary"]["topFailureClusters"][0]["subject"], "study"
            )

    def test_excluded_strategy_is_not_sent_to_the_cli_or_counted_as_eligible(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "strategy.pine").write_text(
                '//@version=4\nstrategy("excluded")\n', encoding="utf-8"
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(
                root,
                [
                    row(
                        "excluded",
                        "strategy.pine",
                        scope="legacy_strategy_excluded",
                    )
                ],
            )
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            self.assertEqual(runner.commands, [])
            self.assertEqual(report["summary"]["eligibleLegacyIndicators"], 0)
            self.assertEqual(report["summary"]["excludedLegacyStrategies"], 1)
            self.assertEqual(
                report["items"][0]["classifiedScope"],
                "legacy_strategy_excluded",
            )
            stages = report["items"][0]["stages"]
            self.assertEqual(stages["analyze"]["status"], "excluded")

    def test_detected_strategy_is_excluded_even_if_manifest_scope_is_wrong(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "strategy.pine").write_text(
                '//@version=4\nstrategy("misclassified")\n', encoding="utf-8"
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(root, [row("strategy", "strategy.pine")])
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            self.assertEqual(runner.commands, [])
            self.assertEqual(report["summary"]["eligibleLegacyIndicators"], 0)
            self.assertEqual(report["summary"]["excludedLegacyStrategies"], 1)
            self.assertEqual(report["summary"]["scopeMismatchCount"], 1)

    def test_missing_bars_are_distinct_from_analysis_failure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("eventually supported")\n',
                encoding="utf-8",
            )
            manifest = write_manifest(root, [row("legacy", "legacy.pine")])
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            stages = report["items"][0]["stages"]
            self.assertEqual(stages["analyze"]["status"], "passed")
            self.assertEqual(stages["historicalRun"]["status"], "missing_input")
            self.assertEqual(
                report["items"][0]["inputAvailability"]["chartBars"],
                "missing_input",
            )

    def test_chart_context_is_forwarded_to_historical_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("context")\nplot(close)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(root, [row("legacy", "legacy.pine")])
            runner = FakeRunner()

            analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            run_command = next(command for command in runner.commands if command[1] == "run")
            self.assertIn("--chart-symbol", run_command)
            self.assertEqual(run_command[run_command.index("--chart-symbol") + 1], "TEST")
            self.assertIn("--chart-timeframe", run_command)
            self.assertEqual(run_command[run_command.index("--chart-timeframe") + 1], "1")

    def test_runtime_modes_are_compared_with_the_historical_result(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("modes")\nplot(close)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(root, [row("legacy", "legacy.pine")])
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            self.assertEqual(
                [
                    command[1]
                    for command in runner.commands
                    if command[1] != "analyze"
                ],
                ["run", "run-incremental", "run-realtime-history"],
            )
            stages = report["items"][0]["stages"]
            self.assertEqual(stages["historicalRun"]["status"], "passed")
            self.assertEqual(stages["incrementalRun"]["status"], "passed")
            self.assertEqual(stages["realtimeRun"]["status"], "passed")
            self.assertEqual(stages["resourceAudit"]["status"], "passed")
            resources = report["items"][0]["runtimeResources"]
            self.assertEqual(resources["modes"]["batch"]["retainedValues"], 1)
            self.assertEqual(
                resources["modes"]["realtimeHistory"],
                resources["modes"]["batch"],
            )
            self.assertTrue(
                report["summary"]["runtimeResources"]["allEligiblePassed"]
            )

    def test_resource_audit_rejects_cache_and_ceiling_regressions(self) -> None:
        snapshot = {
            "bars": 10,
            "retainedValues": 20,
            "maxSeriesDepth": 4,
            "requestCacheEntries": 2,
            "requestCacheContexts": 1,
            "requestCacheValues": 8,
        }
        resources = {
            "batch": snapshot,
            "incremental": dict(snapshot),
            "realtimeHistory": dict(snapshot),
        }
        self.assertEqual(
            analyze_legacy_corpus.resource_audit_status(
                resources, provider_supplied=True
            )["status"],
            "passed",
        )

        resources["incremental"]["requestCacheEntries"] = 1
        mismatch = analyze_legacy_corpus.resource_audit_status(
            resources, provider_supplied=True
        )
        self.assertEqual(mismatch["errorKind"], "profile_mismatch")

        missing_cache = {
            mode: {
                **snapshot,
                "requestCacheEntries": 0,
                "requestCacheContexts": 0,
                "requestCacheValues": 0,
            }
            for mode in resources
        }
        missing = analyze_legacy_corpus.resource_audit_status(
            missing_cache, provider_supplied=True
        )
        self.assertEqual(missing["errorKind"], "request_cache_missing")

        over_ceiling = {
            mode: {
                **snapshot,
                "retainedValues": (
                    analyze_legacy_corpus.CORPUS_RETAINED_VALUE_CEILING + 1
                ),
            }
            for mode in resources
        }
        ceiling = analyze_legacy_corpus.resource_audit_status(
            over_ceiling, provider_supplied=False
        )
        self.assertEqual(ceiling["errorKind"], "retained_value_ceiling_exceeded")

    def test_runtime_mode_result_mismatch_is_a_stable_failure(self) -> None:
        class MismatchRunner(FakeRunner):
            def __call__(
                self, command: list[str] | tuple[str, ...], root: Path
            ) -> subprocess.CompletedProcess[str]:
                result = super().__call__(command, root)
                if command[1] == "run-incremental":
                    profile = {
                        "bars": 1,
                        "maxSeriesDepth": 1,
                        "requestCacheEntries": 0,
                        "requestCacheContexts": 0,
                        "requestCacheValues": 0,
                        "plotValues": 1,
                    }
                    return subprocess.CompletedProcess(
                        command,
                        0,
                        stdout=json.dumps(
                            {
                                "schemaVersion": 8,
                                "plots": [{"id": 1}],
                                "profile": profile,
                            }
                        )
                        + "\n",
                        stderr="",
                    )
                return result

        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("mismatch")\nplot(close)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest = write_manifest(root, [row("legacy", "legacy.pine")])

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=MismatchRunner(),
            )

            incremental = report["items"][0]["stages"]["incrementalRun"]
            self.assertEqual(incremental["status"], "failed")
            self.assertEqual(incremental["errorKind"], "result_mismatch")
            self.assertEqual(
                report["items"][0]["stages"]["realtimeRun"]["status"],
                "passed",
            )

    def test_execution_times_are_classified_and_forwarded_without_disclosure(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("clock")\nplot(timenow)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            timestamp = "1700000000123"
            (root / "execution-times.txt").write_text(
                timestamp + "\n",
                encoding="utf-8",
            )
            manifest_row = row("legacy", "legacy.pine")
            manifest_row["execution_times_path"] = "execution-times.txt"
            manifest = write_manifest(root, [manifest_row])
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            run_command = next(command for command in runner.commands if command[1] == "run")
            self.assertEqual(
                run_command[run_command.index("--execution-times") + 1],
                str(root / "execution-times.txt"),
            )
            self.assertEqual(
                report["items"][0]["inputAvailability"]["executionTimes"],
                "passed",
            )
            self.assertFalse(report["privacy"]["timestampsIncluded"])
            self.assertNotIn(timestamp, analyze_legacy_corpus.render_report(report))

    def test_missing_execution_times_file_stops_before_runtime(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "legacy.pine").write_text(
                '//@version=4\nstudy("clock")\nplot(timenow)\n',
                encoding="utf-8",
            )
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest_row = row("legacy", "legacy.pine")
            manifest_row["execution_times_path"] = "missing-times.txt"
            manifest = write_manifest(root, [manifest_row])
            runner = FakeRunner()

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=runner,
            )

            self.assertEqual(
                report["items"][0]["inputAvailability"]["executionTimes"],
                "missing_input",
            )
            self.assertEqual(
                report["items"][0]["stages"]["historicalRun"]["status"],
                "missing_input",
            )
            self.assertFalse(any(command[1] == "run" for command in runner.commands))

    def test_all_external_inputs_are_classified_before_compilation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_row = row("legacy", "missing_source.pine")
            manifest_row["chart_bars_path"] = "missing_bars.csv"
            manifest_row["execution_times_path"] = "missing_execution_times.txt"
            manifest_row["request_data_manifest"] = "missing_requests.json"
            manifest_row["reference_output_path"] = "missing_reference.json"
            manifest = write_manifest(root, [manifest_row])

            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=FakeRunner(),
            )

            item = report["items"][0]
            self.assertEqual(
                item["inputAvailability"],
                {
                    "source": "missing_input",
                    "chartBars": "missing_input",
                    "executionTimes": "missing_input",
                    "requestData": "missing_input",
                    "referenceOutput": "missing_input",
                },
            )
            self.assertEqual(
                report["summary"]["missingInputCounts"],
                {
                    "source": 1,
                    "chartBars": 1,
                    "executionTimes": 1,
                    "requestData": 1,
                    "referenceOutput": 1,
                },
            )

    def test_manifest_rejects_duplicate_ids_and_unsorted_rows(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            duplicate = write_manifest(
                root,
                [row("same", "one.pine"), row("same", "two.pine")],
            )
            with self.assertRaisesRegex(
                analyze_legacy_corpus.CorpusError, "duplicate id"
            ):
                analyze_legacy_corpus.parse_manifest(duplicate)

            unsorted = write_manifest(
                root,
                [row("z-last", "one.pine"), row("a-first", "two.pine")],
            )
            with self.assertRaisesRegex(
                analyze_legacy_corpus.CorpusError, "must be sorted"
            ):
                analyze_legacy_corpus.parse_manifest(unsorted)

    def test_unknown_user_identifier_is_not_exposed_as_a_subject(self) -> None:
        diagnostics = analyze_legacy_corpus.parse_diagnostics(
            "E_UNKNOWN_FUNCTION:Error:4:2: unknown function `privateAlphaSignal`\n"
            "E_CALL_ARG_NAME:Error:5:3: `plot` has no argument named `transp`\n"
        )

        self.assertIsNone(diagnostics[0].subject)
        self.assertEqual(diagnostics[1].subject, "plot.transp")
        self.assertEqual(diagnostics[1].feature_category, "output_option")
        self.assertEqual(diagnostics[1].canonical_candidate, "color.new")

    def test_request_manifest_is_sorted_and_validates_bar_paths(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "a.csv").write_text("bars", encoding="utf-8")
            (root / "z.csv").write_text("bars", encoding="utf-8")
            (root / "requests.json").write_text(
                json.dumps({"Z:D": "z.csv", "A:D": "a.csv"}),
                encoding="utf-8",
            )

            status, specs = analyze_legacy_corpus.request_specs(
                root, "requests.json"
            )

            self.assertEqual(status, "passed")
            self.assertEqual(
                specs,
                [f"A:D={root / 'a.csv'}", f"Z:D={root / 'z.csv'}"],
            )

    def test_stable_baseline_thresholds_use_the_full_eligible_denominator(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest_rows = []
            for index in range(50):
                source_path = f"v4-{index:03}.pine"
                (root / source_path).write_text(
                    '//@version=4\nstudy("baseline")\nplot(close)\n',
                    encoding="utf-8",
                )
                manifest_rows.append(row(f"v4-{index:03}", source_path))

            manifest = write_manifest(root, manifest_rows)
            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=FakeRunner(),
            )

            self.assertEqual(report["schemaVersion"], 4)
            profile = report["summary"]["versions"]["4"]
            self.assertEqual(profile["historicalRun"]["eligibleSuccessRate"], 1.0)
            self.assertTrue(profile["stableBaseline"]["thresholdsMet"])
            self.assertEqual(
                profile["stableBaseline"]["eligibleScripts"],
                {"actual": 50, "required": 50, "remaining": 0, "met": True},
            )
            self.assertTrue(
                profile["stableBaseline"]["fullExecutionAuditStillRequired"]
            )

    def test_two_percent_unknown_cluster_requires_profile_disposition(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            (root / "bars.csv").write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            manifest_rows = []
            for index in range(50):
                source_path = f"v4-{index:03}.pine"
                body = "legacy_failure" if index == 0 else "plot(close)"
                (root / source_path).write_text(
                    f'//@version=4\nstudy("baseline")\n{body}\n',
                    encoding="utf-8",
                )
                manifest_rows.append(row(f"v4-{index:03}", source_path))

            manifest = write_manifest(root, manifest_rows)
            report = analyze_legacy_corpus.build_report(
                analyze_legacy_corpus.parse_manifest(manifest),
                root=root,
                manifest_path=manifest,
                pine_compat=root / "pine-compat",
                build_revision="test-revision",
                command_runner=FakeRunner(),
            )

            cluster = report["summary"]["topFailureClusters"][0]
            self.assertEqual(cluster["affectedScripts"], 1)
            self.assertEqual(cluster["affectedScriptsByVersion"], {"4": 1})
            self.assertEqual(cluster["eligibleShareByVersion"], {"4": 0.02})
            self.assertTrue(cluster["requiresDisposition"])

            baseline = report["summary"]["versions"]["4"]["stableBaseline"]
            self.assertFalse(baseline["thresholdsMet"])
            self.assertEqual(baseline["unknownClustersRequiringDisposition"], 1)
            self.assertEqual(
                baseline["blockingReasons"],
                ["unknown_failure_cluster_requires_disposition"],
            )


if __name__ == "__main__":
    unittest.main()
