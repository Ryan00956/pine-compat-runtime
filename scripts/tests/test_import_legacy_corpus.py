from __future__ import annotations

import csv
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import import_legacy_corpus  # noqa: E402


class ImportLegacyCorpusTests(unittest.TestCase):
    def test_import_preserves_sources_and_builds_private_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            source_dir = root / "input"
            source_dir.mkdir()
            v4 = b'//@version=4\r\nstudy("old")\r\nplot(close)\r\n'
            implicit = b'study("implicit")\r\nplot(close)\r\n'
            noncanonical = (
                b'// @version = 6\r\nindicator("modern")\r\nplot(close)\r\n'
            )
            spaced = (
                b'//@version = 4\r\nstudy("spaced")\r\nplot(close)\r\n'
            )
            (source_dir / "Named v4").write_bytes(v4)
            (source_dir / "Named implicit").write_bytes(implicit)
            (source_dir / "Named modern").write_bytes(noncanonical)
            (source_dir / "Named spaced").write_bytes(spaced)
            bars = root / "bars.csv"
            bars.write_text(
                "time,open,high,low,close,volume\n0,1,1,1,1,1\n",
                encoding="utf-8",
            )
            output_dir = root / "corpus"

            sources = import_legacy_corpus.discover_sources(source_dir)
            import_legacy_corpus.write_import(
                sources, output_dir=output_dir, chart_bars=bars
            )

            with (output_dir / "corpus.tsv").open(
                newline="", encoding="utf-8"
            ) as handle:
                manifest = list(csv.DictReader(handle, delimiter="\t"))
            with (output_dir / "source-map.tsv").open(
                newline="", encoding="utf-8"
            ) as handle:
                source_map = list(csv.DictReader(handle, delimiter="\t"))

            self.assertEqual(
                [row["id"] for row in manifest],
                sorted(row["id"] for row in manifest),
            )
            self.assertNotIn("Named", (output_dir / "corpus.tsv").read_text())
            self.assertIn("Named v4", (output_dir / "source-map.tsv").read_text())
            self.assertEqual(
                {row["license_class"] for row in manifest},
                {"private_user_authorized"},
            )
            self.assertEqual(
                {row["declared_or_expected_version"] for row in manifest},
                {"1", "4", "6"},
            )
            self.assertEqual(
                {row["expected_scope"] for row in manifest},
                {"legacy_indicator", "modern_indicator_control"},
            )
            by_name = {row["original_relative_path"]: row for row in source_map}
            self.assertEqual(
                by_name["Named modern"]["version_directive"],
                "noncanonical_runtime_detects_v1",
            )
            self.assertEqual(
                by_name["Named spaced"]["version_directive"],
                "compat_spaced_equals",
            )
            copied = {
                row["source_sha256"]: (
                    output_dir / "sources" / f"{row['id']}.pine"
                ).read_bytes()
                for row in source_map
            }
            self.assertEqual(
                copied[import_legacy_corpus.sha256_bytes(v4)],
                v4,
            )

    def test_strategy_is_excluded_and_unknown_input_is_a_control(self) -> None:
        strategy = import_legacy_corpus.ImportedSource(
            item_id="strategy",
            relative_path="strategy",
            source_bytes=b"strategy()",
            source_sha256="a" * 64,
            expected_version=4,
            version_directive="canonical",
            declaration_mode="strategy",
            license_hint="unspecified",
        )
        invalid = import_legacy_corpus.ImportedSource(
            item_id="invalid",
            relative_path="invalid",
            source_bytes=b"not pine",
            source_sha256="b" * 64,
            expected_version=1,
            version_directive="implicit_v1",
            declaration_mode="unknown",
            license_hint="unspecified",
        )

        self.assertEqual(
            import_legacy_corpus.corpus_scope(strategy),
            "legacy_strategy_excluded",
        )
        self.assertEqual(
            import_legacy_corpus.corpus_scope(invalid),
            "invalid_control",
        )

    def test_existing_output_is_never_overwritten(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            root = Path(temp_dir)
            output_dir = root / "corpus"
            output_dir.mkdir()
            marker = output_dir / "keep.txt"
            marker.write_text("keep", encoding="utf-8")
            bars = root / "bars.csv"
            bars.write_text("bars", encoding="utf-8")

            with self.assertRaisesRegex(
                import_legacy_corpus.CorpusImportError, "refusing to overwrite"
            ):
                import_legacy_corpus.write_import(
                    [], output_dir=output_dir, chart_bars=bars
                )

            self.assertEqual(marker.read_text(encoding="utf-8"), "keep")


if __name__ == "__main__":
    unittest.main()
