#!/usr/bin/env python3
"""Profile legacy release fixtures through the public CLI boundary."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import statistics
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Mapping, Sequence


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_VERSION = 1
MANIFEST_COLUMNS = (
    "id",
    "version",
    "maturity",
    "category",
    "source_path",
    "bars_profile",
    "request_profile",
    "execution_profile",
    "realtime_policy",
    "license_class",
    "max_retained_values",
)
RETAINED_PROFILE_FIELDS = (
    "requestCacheValues",
    "seriesValues",
    "rollingWindowValues",
    "valuewhenStateValues",
    "arrayValues",
    "matrixCells",
    "plotValues",
    "plotCharValues",
    "plotShapeValues",
    "plotArrowValues",
    "plotBarValues",
    "plotCandleValues",
    "bgColorValues",
    "barColorValues",
    "labelSnapshots",
    "lineSnapshots",
    "lineFillSnapshots",
    "polylineSnapshots",
    "polylinePoints",
    "boxSnapshots",
    "tableCells",
)


@dataclass(frozen=True)
class ReleaseRow:
    item_id: str
    version: int
    maturity: str
    category: str
    source_path: str
    bars_profile: str
    request_profile: str
    execution_profile: str
    realtime_policy: str
    license_class: str
    max_retained_values: int


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def parse_manifest(path: Path) -> list[ReleaseRow]:
    with path.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle, delimiter="\t")
        if tuple(reader.fieldnames or ()) != MANIFEST_COLUMNS:
            raise ValueError("unexpected legacy release manifest columns")
        rows = [
            ReleaseRow(
                item_id=raw["id"],
                version=int(raw["version"]),
                maturity=raw["maturity"],
                category=raw["category"],
                source_path=raw["source_path"],
                bars_profile=raw["bars_profile"],
                request_profile=raw["request_profile"],
                execution_profile=raw["execution_profile"],
                realtime_policy=raw["realtime_policy"],
                license_class=raw["license_class"],
                max_retained_values=int(raw["max_retained_values"]),
            )
            for raw in reader
        ]
    if not rows or [row.item_id for row in rows] != sorted(row.item_id for row in rows):
        raise ValueError("legacy release manifest must be nonempty and sorted by id")
    if len({row.item_id for row in rows}) != len(rows):
        raise ValueError("legacy release manifest contains duplicate ids")
    return rows


def retained_values(profile: Mapping[str, object]) -> int:
    return sum(int(profile.get(field, 0)) for field in RETAINED_PROFILE_FIELDS)


def bars_path(row: ReleaseRow) -> Path:
    if row.bars_profile == "security_chart":
        return ROOT / "tests/fixtures/legacy/v4/runtime/security_chart_bars.csv"
    return ROOT / "tests/fixtures/legacy/chart_1m.csv"


def request_arguments(row: ReleaseRow) -> list[str]:
    if row.request_profile == "none":
        return []
    if row.request_profile == "test_chart":
        return ["--chart-symbol", "TEST", "--chart-timeframe", "1"]
    if row.request_profile == "ibm_5":
        return [
            "--request-bars",
            "NYSE:IBM:5=tests/fixtures/legacy/v4/runtime/security_request_5m.csv",
        ]
    if row.request_profile == "ibm_1":
        return [
            "--request-bars",
            "NYSE:IBM:1=tests/fixtures/legacy/v4/runtime/security_chart_bars.csv",
        ]
    if row.request_profile == "test_daily":
        return [
            "--chart-symbol",
            "TEST",
            "--chart-timeframe",
            "1",
            "--request-bars",
            "TEST:D=tests/fixtures/legacy/request_daily.csv",
        ]
    raise ValueError(f"unknown request profile {row.request_profile!r}")


def execution_arguments(row: ReleaseRow) -> list[str]:
    if row.execution_profile == "none":
        return []
    if row.execution_profile == "deterministic_clock":
        return [
            "--execution-times",
            "tests/fixtures/legacy/timenow_execution_times.txt",
        ]
    raise ValueError(f"unknown execution profile {row.execution_profile!r}")


def run_json(command: Sequence[str]) -> dict[str, object]:
    result = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed ({result.returncode}): {' '.join(command)}\n{result.stderr}"
        )
    return json.loads(result.stdout)


def profile_fixture(binary: Path, row: ReleaseRow, iterations: int) -> dict[str, object]:
    source = ROOT / row.source_path
    analyze_command = [str(binary), "analyze", str(source), "--format", "json"]
    run_json(analyze_command)
    samples_ms: list[float] = []
    for _ in range(iterations):
        started = time.perf_counter_ns()
        run_json(analyze_command)
        samples_ms.append((time.perf_counter_ns() - started) / 1_000_000)

    run_command = [
        str(binary),
        "run",
        str(source),
        "--bars",
        str(bars_path(row)),
        *request_arguments(row),
        *execution_arguments(row),
        "--profile",
    ]
    runtime = run_json(run_command)
    profile = runtime.get("profile")
    if not isinstance(profile, dict):
        raise RuntimeError(f"{row.item_id}: profiled CLI run omitted profile data")
    retained = retained_values(profile)
    if retained > row.max_retained_values:
        raise RuntimeError(
            f"{row.item_id}: retained {retained} values above {row.max_retained_values}"
        )
    return {
        "id": row.item_id,
        "version": row.version,
        "maturity": row.maturity,
        "analyzeMinMs": round(min(samples_ms), 3),
        "analyzeMedianMs": round(statistics.median(samples_ms), 3),
        "analyzeMaxMs": round(max(samples_ms), 3),
        "bars": profile.get("bars"),
        "retainedValues": retained,
        "retainedValueCeiling": row.max_retained_values,
        "maxSeriesDepth": profile.get("maxSeriesDepth"),
        "historyRetentionMode": profile.get("historyRetentionMode"),
    }


def build_report(binary: Path, manifest: Path, iterations: int) -> dict[str, object]:
    rows = parse_manifest(manifest)
    fixtures = [profile_fixture(binary, row, iterations) for row in rows]
    medians = [float(item["analyzeMedianMs"]) for item in fixtures]
    retained = [int(item["retainedValues"]) for item in fixtures]
    return {
        "schemaVersion": SCHEMA_VERSION,
        "measurement": "end-to-end CLI process latency; indicative, not a release gate",
        "iterationsPerFixture": iterations,
        "manifestSha256": sha256_file(manifest),
        "summary": {
            "fixtures": len(fixtures),
            "analyzeMedianOfMediansMs": round(statistics.median(medians), 3),
            "analyzeMaximumMedianMs": round(max(medians), 3),
            "maximumRetainedValues": max(retained),
            "retainedValueCeiling": max(row.max_retained_values for row in rows),
        },
        "fixtures": fixtures,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--binary", type=Path, default=ROOT / "target/debug/pine-compat"
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "tests/fixtures/legacy/release_profiles.tsv",
    )
    parser.add_argument("--iterations", type=int, default=7)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.iterations < 1:
        parser.error("--iterations must be positive")
    binary = args.binary.resolve()
    if not binary.is_file():
        parser.error(f"CLI binary does not exist: {binary}; run cargo build -p pine-cli")

    report = build_report(binary, args.manifest.resolve(), args.iterations)
    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
    else:
        args.output.write_text(encoded, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
