from __future__ import annotations

import json
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import compare_tradingview_outputs as compare  # noqa: E402


class CompareTradingViewOutputsTests(unittest.TestCase):
    def test_maps_duplicate_titles_by_position_and_applies_offsets(self) -> None:
        runtime = {
            "plots": [
                {"id": 7, "title": None, "values": [1, 2, 3]},
                {"id": 9, "title": None, "offset": -1, "values": [10, 20, 30]},
            ]
        }
        outputs = compare.parse_runtime_outputs(runtime)
        report = compare.compare_outputs(
            outputs,
            ["time", "Plot", "Plot"],
            [["1", "1", "20"], ["2", "2", "30"], ["3", "3", ""]],
            column_start=1,
            column_count=2,
            skip_bars=0,
            absolute_tolerance=0,
            relative_tolerance=0,
        )

        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["mismatches"], 0)
        self.assertEqual([item["tradingview_column"] for item in report["outputs"]], [1, 2])

    def test_treats_zero_and_null_as_equivalent_only_for_plotshape(self) -> None:
        shape = compare.RuntimeOutput(1, "plotshape", "Shape", 0, [None, 1])
        plot = compare.RuntimeOutput(2, "plot", "Plot", 0, [None, 1])
        rows = [["1", "0", "0"], ["2", "1", "1"]]

        shape_report = compare.compare_outputs(
            [shape],
            ["time", "Shape"],
            [row[:2] for row in rows],
            column_start=1,
            column_count=None,
            skip_bars=0,
            absolute_tolerance=0,
            relative_tolerance=0,
        )
        plot_report = compare.compare_outputs(
            [plot],
            ["time", "Plot"],
            [[row[0], row[2]] for row in rows],
            column_start=1,
            column_count=None,
            skip_bars=0,
            absolute_tolerance=0,
            relative_tolerance=0,
        )

        self.assertEqual(shape_report["status"], "passed")
        self.assertEqual(plot_report["mismatches"], 1)

    def test_reports_first_numeric_mismatch_after_warmup(self) -> None:
        output = compare.RuntimeOutput(4, "plot", "Value", 0, [100, 2, 9])
        report = compare.compare_outputs(
            [output],
            ["time", "Value"],
            [["1", "0"], ["2", "2"], ["3", "3"]],
            column_start=1,
            column_count=1,
            skip_bars=1,
            absolute_tolerance=0,
            relative_tolerance=0,
        )

        self.assertEqual(report["mismatches"], 1)
        self.assertEqual(
            report["outputs"][0]["first_mismatch"],
            {"bar_index": 2, "time": "3", "runtime": 9, "tradingview": 3.0},
        )
        self.assertEqual(report["outputs"][0]["max_absolute_error"], 6.0)

    def test_can_drop_a_live_final_bar(self) -> None:
        output = compare.RuntimeOutput(4, "plot", "Value", 0, [1, 2, 99])
        report = compare.compare_outputs(
            [output],
            ["time", "Value"],
            [["1", "1"], ["2", "2"], ["3", "3"]],
            column_start=1,
            column_count=None,
            skip_bars=0,
            absolute_tolerance=0,
            relative_tolerance=0,
            drop_last_bars=1,
        )

        self.assertEqual(report["status"], "passed")
        self.assertEqual(report["compared_bars"], 2)
        self.assertEqual(report["drop_last_bars"], 1)

    def test_loads_runtime_json_and_rejects_row_count_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            path = Path(temp_dir) / "runtime.json"
            path.write_text(
                json.dumps({"plotChars": [{"id": 3, "values": [True, None]}]}),
                encoding="utf-8",
            )
            outputs = compare.parse_runtime_outputs(compare.load_runtime_result(path))

        self.assertEqual(outputs[0].values, [1, None])
        with self.assertRaisesRegex(compare.OutputComparisonError, "has 2 values"):
            compare.compare_outputs(
                outputs,
                ["time", "Signal"],
                [["1", "1"]],
                column_start=1,
                column_count=None,
                skip_bars=0,
                absolute_tolerance=0,
                relative_tolerance=0,
            )


if __name__ == "__main__":
    unittest.main()
