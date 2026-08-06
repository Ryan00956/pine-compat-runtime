import csv
from pathlib import Path
import sys
import tempfile
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import audit_legacy_corpus_dedup as dedup  # noqa: E402
from analyze_legacy_corpus import REQUIRED_COLUMNS  # noqa: E402


def write_manifest(root: Path, name: str, sources: list[tuple[str, str]]) -> Path:
    manifest = root / name
    rows = []
    for item_id, source in sources:
        source_path = root / f"{item_id}.pine"
        source_path.write_text(source, encoding="utf-8", newline="")
        rows.append(
            {
                "id": item_id,
                "source_path": source_path.name,
                "declared_or_expected_version": "4",
                "chart_bars_path": "bars.csv",
                "chart_symbol": "TEST",
                "chart_timeframe": "1",
                "execution_times_path": "",
                "request_data_manifest": "",
                "reference_output_path": "",
                "license_class": "original",
                "expected_scope": "legacy_indicator",
                "notes": "test",
            }
        )
    rows.sort(key=lambda row: row["id"])
    with manifest.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle, fieldnames=REQUIRED_COLUMNS, delimiter="\t", lineterminator="\n"
        )
        writer.writeheader()
        writer.writerows(rows)
    return manifest


class LegacyCorpusDedupTests(unittest.TestCase):
    def test_reports_exact_normalized_and_token_equivalent_matches(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = write_manifest(
                root,
                "baseline.tsv",
                [("base", '//@version=4\nstudy("x")\nplot(close)\n')],
            )
            candidate = write_manifest(
                root,
                "candidate.tsv",
                [
                    ("candidate_exact", '//@version=4\nstudy("x")\nplot(close)\n'),
                    (
                        "candidate_normalized",
                        '//@version=4\r\nstudy("x")  \r\nplot(close)\r\n',
                    ),
                    (
                        "candidate_tokens",
                        '// formatting only\n//@version=4\nstudy( "x" )\nplot( close )\n',
                    ),
                    ("candidate_unique", '//@version=4\nstudy("y")\nplot(open)\n'),
                ],
            )

            report = dedup.build_report(
                baseline_manifest=baseline,
                baseline_root=root,
                candidate_manifest=candidate,
                candidate_root=root,
                version=4,
                stable_profile_floor=5,
                build_revision="test",
            )

        self.assertEqual(report["summary"]["exactMatches"], 1)
        self.assertEqual(report["summary"]["normalizedMatches"], 1)
        self.assertEqual(report["summary"]["tokenEquivalentMatches"], 1)
        self.assertEqual(report["summary"]["crossManifestMatchedCandidates"], 3)
        self.assertEqual(report["summary"]["combinedSelected"], 5)
        self.assertEqual(report["summary"]["combinedUnique"], 2)
        self.assertEqual(report["summary"]["candidateContribution"], 1)
        self.assertEqual(report["summary"]["stableProfileFloorRemaining"], 3)

    def test_token_fingerprint_is_version_aware_and_literal_sensitive(self) -> None:
        source = '//@version=4\nstudy("x")\nplot(close + 1)\n'
        v4, count = dedup.token_fingerprint(source, 4, "source")
        v3, _ = dedup.token_fingerprint(source, 3, "source")
        changed, changed_count = dedup.token_fingerprint(
            source.replace("+ 1", "+ 2"), 4, "changed"
        )

        self.assertGreater(count, 0)
        self.assertEqual(count, changed_count)
        self.assertNotEqual(v4, v3)
        self.assertNotEqual(v4, changed)

    def test_report_omits_paths_titles_and_source_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            baseline = write_manifest(
                root,
                "baseline.tsv",
                [("base", '//@version=4\nstudy("private title")\nplot(close)\n')],
            )
            candidate = write_manifest(
                root,
                "candidate.tsv",
                [("opaque-candidate", '//@version=4\nstudy("other")\nplot(open)\n')],
            )

            report = dedup.build_report(
                baseline_manifest=baseline,
                baseline_root=root,
                candidate_manifest=candidate,
                candidate_root=root,
                version=4,
                stable_profile_floor=2,
                build_revision="test",
            )

        encoded = str(report)
        self.assertNotIn("private title", encoded)
        self.assertNotIn(str(root), encoded)
        self.assertNotIn("source_path", encoded)
        self.assertTrue(report["summary"]["stableProfileFloorReached"])

    def test_rejects_manifest_version_mismatch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            manifest = write_manifest(
                root,
                "corpus.tsv",
                [("wrong", '//@version=3\nstudy("x")\nplot(close)\n')],
            )

            with self.assertRaisesRegex(dedup.DedupAuditError, "does not match"):
                dedup.selected_sources(
                    manifest, origin="candidate", root=root, version=4
                )


if __name__ == "__main__":
    unittest.main()
