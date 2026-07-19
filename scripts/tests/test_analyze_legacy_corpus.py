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
        return subprocess.CompletedProcess(
            rendered,
            0,
            stdout='{"schemaVersion":7,"plots":[]}\n',
            stderr="",
        )


class AnalyzeLegacyCorpusTests(unittest.TestCase):
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

    def test_all_external_inputs_are_classified_before_compilation(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            manifest_row = row("legacy", "missing_source.pine")
            manifest_row["chart_bars_path"] = "missing_bars.csv"
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
                    "requestData": "missing_input",
                    "referenceOutput": "missing_input",
                },
            )
            self.assertEqual(
                report["summary"]["missingInputCounts"],
                {
                    "source": 1,
                    "chartBars": 1,
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


if __name__ == "__main__":
    unittest.main()
