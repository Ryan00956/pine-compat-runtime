#!/usr/bin/env python3
"""Guard CLI runtime snapshots against missing WASM/Python host assertions."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
FIXTURE_DIR = ROOT / "crates/pine-cli/src/runtime_snapshots/fixtures"
WASM_TESTS = ROOT / "crates/pine-wasm/src/tests/mod.rs"
PYTHON_TESTS = ROOT / "python/tests/test_bindings.py"

SNAPSHOT_FIXTURE = re.compile(
    r'\(\s*"([^"]+\.json)",\s*"([^"]+\.pine)"\s*\)', re.MULTILINE
)


def runtime_snapshot_fixtures() -> list[tuple[Path, str, str]]:
    fixtures: list[tuple[Path, str, str]] = []
    for path in sorted(FIXTURE_DIR.glob("*.rs")):
        for snapshot, source in SNAPSHOT_FIXTURE.findall(path.read_text()):
            fixtures.append((path, snapshot, source))
    return fixtures


def main() -> int:
    wasm_tests = WASM_TESTS.read_text()
    python_tests = PYTHON_TESTS.read_text()
    missing: list[str] = []

    for path, snapshot, source in runtime_snapshot_fixtures():
        missing_hosts = []
        if snapshot not in wasm_tests:
            missing_hosts.append("WASM")
        if snapshot not in python_tests:
            missing_hosts.append("Python")
        if missing_hosts:
            rel_path = path.relative_to(ROOT)
            missing.append(
                f"{snapshot} ({source}) from {rel_path}: missing {', '.join(missing_hosts)}"
            )

    if missing:
        print("Host parity guard failed: missing public JSON assertions.")
        for item in missing:
            print(f"- {item}")
        return 1

    print(
        "Host parity guard passed: "
        f"checked {len(runtime_snapshot_fixtures())} CLI runtime snapshots."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
