import unittest

from scripts.profile_legacy_release import ROOT, parse_manifest, retained_values


class LegacyReleaseProfileTests(unittest.TestCase):
    def test_release_manifest_is_sorted_and_has_expected_version_counts(self) -> None:
        rows = parse_manifest(ROOT / "tests/fixtures/legacy/release_profiles.tsv")
        self.assertEqual(len(rows), 18)
        self.assertEqual(
            {version: sum(row.version == version for row in rows) for version in range(1, 5)},
            {1: 4, 2: 3, 3: 2, 4: 9},
        )
        self.assertEqual(
            {row.maturity for row in rows if row.version >= 3}, {"preview"}
        )
        self.assertEqual(
            {row.maturity for row in rows if row.version <= 2}, {"experimental"}
        )

    def test_retained_values_uses_only_pinned_storage_counts(self) -> None:
        self.assertEqual(
            retained_values(
                {
                    "seriesValues": 3,
                    "rollingWindowValues": 5,
                    "plotValues": 7,
                    "seriesCapacity": 100,
                    "unknownFutureField": 999,
                }
            ),
            15,
        )


if __name__ == "__main__":
    unittest.main()
