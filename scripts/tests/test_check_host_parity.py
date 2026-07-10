from pathlib import Path
import sys
import unittest


SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))

import check_host_parity  # noqa: E402


class HostParityGuardTests(unittest.TestCase):
    def test_parses_rustfmt_multiline_tuples_and_trailing_commas(self):
        fixtures = check_host_parity.parse_runtime_snapshot_fixtures(
            '''
const FIXTURES: &[Fixture] = &[
    ("runtime_inline.json", "tests/fixtures/runtime/inline.pine"),
    (
        "runtime_multiline.json",
        "tests/fixtures/runtime/multiline.pine",
    ),
];
''',
            Path("fixtures.rs"),
        )

        self.assertEqual(
            [(item.snapshot, item.source) for item in fixtures],
            [
                ("runtime_inline.json", "tests/fixtures/runtime/inline.pine"),
                (
                    "runtime_multiline.json",
                    "tests/fixtures/runtime/multiline.pine",
                ),
            ],
        )

    def test_deleting_either_required_host_assertion_fails(self):
        registered = {"required.json", "registered_only.json"}
        required = {"required.json"}
        both_hosts = {"required.json"}

        self.assertEqual(
            check_host_parity.parity_errors(
                registered, required, both_hosts, both_hosts
            ),
            [],
        )
        self.assertIn(
            "missing a WASM golden assertion",
            "\n".join(
                check_host_parity.parity_errors(
                    registered, required, set(), both_hosts
                )
            ),
        )
        self.assertIn(
            "missing a Python golden assertion",
            "\n".join(
                check_host_parity.parity_errors(
                    registered, required, both_hosts, set()
                )
            ),
        )

    def test_python_snapshot_path_counts_only_when_expected_is_asserted(self):
        assignment = '''
def test_contract():
    expected = json.loads(
        (ROOT / "tests/snapshots/required.json").read_text()
    )
    result = run_fixture()
'''

        self.assertEqual(
            check_host_parity.python_snapshot_assertions(
                assignment + "    assert result == expected\n"
            ),
            {"required.json"},
        )
        self.assertEqual(
            check_host_parity.python_snapshot_assertions(assignment),
            set(),
        )

    def test_wasm_snapshot_assertion_ignores_comments_and_strings(self):
        source = r'''
// assert_snapshot("commented.json", &output);
/* assert_snapshot("block_comment.json", &output); */
let ordinary = "assert_snapshot(\"ordinary_string.json\", &output);";
let raw = r#"assert_snapshot("raw_string.json", &output);"#;
let lifetime: &'static str = "still code";
assert_snapshot("real.json", &output);
'''

        self.assertEqual(
            check_host_parity.wasm_snapshot_assertions(source),
            {"real.json"},
        )

    def test_new_paired_assertion_must_be_added_to_manifest(self):
        errors = check_host_parity.parity_errors(
            {"required.json", "new.json"},
            {"required.json"},
            {"required.json", "new.json"},
            {"required.json", "new.json"},
        )

        self.assertEqual(
            errors,
            ["paired host snapshot is not recorded in the required manifest: new.json"],
        )

    def test_registered_single_host_assertions_require_pair_or_reasoned_allowlist(self):
        registered = {"wasm_only.json", "python_only.json"}

        errors = check_host_parity.parity_errors(
            registered,
            set(),
            {"wasm_only.json"},
            {"python_only.json"},
        )

        self.assertEqual(
            errors,
            [
                "registered snapshot python_only.json has only a Python golden assertion",
                "registered snapshot wasm_only.json has only a WASM golden assertion",
            ],
        )
        self.assertEqual(
            check_host_parity.parity_errors(
                registered,
                set(),
                {"wasm_only.json"},
                {"python_only.json"},
                {
                    "python_only.json": "Python-only API boundary",
                    "wasm_only.json": "WASM-only API boundary",
                },
            ),
            [],
        )

    def test_unpaired_allowlist_requires_a_live_reasoned_exception(self):
        errors = check_host_parity.parity_errors(
            {"paired.json", "wasm_only.json"},
            {"paired.json"},
            {"paired.json", "wasm_only.json"},
            {"paired.json"},
            {
                "missing.json": "not registered",
                "paired.json": "already required",
                "wasm_only.json": "",
            },
        )

        self.assertIn(
            "unpaired snapshot allowlist entry wasm_only.json must include a reason",
            errors,
        )
        self.assertIn(
            "unpaired snapshot allowlist entry is not registered by the CLI: missing.json",
            errors,
        )
        self.assertIn(
            "required snapshot cannot be exempted from host parity: paired.json",
            errors,
        )


if __name__ == "__main__":
    unittest.main()
