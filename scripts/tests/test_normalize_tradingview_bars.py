from __future__ import annotations

from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import normalize_tradingview_bars  # noqa: E402


class NormalizeTradingViewBarsTests(unittest.TestCase):
    def test_selects_ohlcv_from_combined_export_and_converts_seconds(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source = root / "combined.csv"
            source.write_text(
                "time,open,high,low,close,Volume,Indicator\n"
                "1700000000,10,12,9,11,100,42\n"
                "1700000060,11,13,10,12,110,43\n",
                encoding="utf-8",
            )

            rows = normalize_tradingview_bars.normalize_rows(
                source, time_unit="seconds"
            )

            self.assertEqual(
                rows,
                [
                    ("1700000000000", "10", "12", "9", "11", "100"),
                    ("1700000060000", "11", "13", "10", "12", "110"),
                ],
            )

    def test_rejects_duplicate_or_unsorted_timestamps(self) -> None:
        cases = (("1", "1", "duplicate"), ("2", "1", "unsorted"))
        for first, second, expected in cases:
            with self.subTest(expected=expected), tempfile.TemporaryDirectory() as temp_dir:
                source = Path(temp_dir) / "bars.csv"
                source.write_text(
                    "time,open,high,low,close,volume\n"
                    f"{first},1,1,1,1,1\n"
                    f"{second},1,1,1,1,1\n",
                    encoding="utf-8",
                )

                with self.assertRaisesRegex(
                    normalize_tradingview_bars.TradingViewBarsError, expected
                ):
                    normalize_tradingview_bars.normalize_rows(
                        source, time_unit="milliseconds"
                    )

    def test_rejects_invalid_ohlc_and_missing_volume(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            invalid_ohlc = root / "invalid.csv"
            invalid_ohlc.write_text(
                "time,open,high,low,close,volume\n1,10,9,8,10,1\n",
                encoding="utf-8",
            )
            missing_volume = root / "missing.csv"
            missing_volume.write_text(
                "time,open,high,low,close\n1,10,11,8,10\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                normalize_tradingview_bars.TradingViewBarsError, "high is below"
            ):
                normalize_tradingview_bars.normalize_rows(
                    invalid_ohlc, time_unit="seconds"
                )
            with self.assertRaisesRegex(
                normalize_tradingview_bars.TradingViewBarsError, "missing required"
            ):
                normalize_tradingview_bars.normalize_rows(
                    missing_volume, time_unit="seconds"
                )


if __name__ == "__main__":
    unittest.main()
