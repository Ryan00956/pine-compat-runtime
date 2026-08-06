#!/usr/bin/env python3
"""Compare runtime plot outputs with positional TradingView CSV columns."""

from __future__ import annotations

import argparse
import csv
import json
import math
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, TextIO


SCHEMA_VERSION = 1
OUTPUT_COLLECTIONS = (
    ("plots", "plot"),
    ("plotShapes", "plotshape"),
    ("plotArrows", "plotarrow"),
    ("plotChars", "plotchar"),
)


class OutputComparisonError(ValueError):
    """Raised when the two artifacts cannot be compared without guessing."""


@dataclass(frozen=True)
class RuntimeOutput:
    output_id: int
    kind: str
    title: str | None
    offset: int
    values: list[int | float | None]


def load_runtime_result(path: Path, stdin: TextIO = sys.stdin) -> dict[str, Any]:
    try:
        if str(path) == "-":
            value = json.load(stdin)
        else:
            with path.open(encoding="utf-8") as handle:
                value = json.load(handle)
    except (OSError, json.JSONDecodeError) as exc:
        raise OutputComparisonError(f"failed to read runtime JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise OutputComparisonError("runtime JSON root must be an object")
    return value


def parse_runtime_outputs(result: dict[str, Any]) -> list[RuntimeOutput]:
    outputs: list[RuntimeOutput] = []
    for collection_name, kind in OUTPUT_COLLECTIONS:
        collection = result.get(collection_name, [])
        if not isinstance(collection, list):
            raise OutputComparisonError(
                f"runtime JSON field {collection_name!r} must be an array"
            )
        for raw in collection:
            if not isinstance(raw, dict):
                raise OutputComparisonError(
                    f"runtime JSON field {collection_name!r} contains a non-object"
                )
            output_id = raw.get("id")
            title = raw.get("title")
            offset = raw.get("offset", 0)
            values = raw.get("values")
            if not isinstance(output_id, int):
                raise OutputComparisonError(f"{kind} output id must be an integer")
            if title is not None and not isinstance(title, str):
                raise OutputComparisonError(f"output {output_id} title must be a string or null")
            if not isinstance(offset, int):
                raise OutputComparisonError(f"output {output_id} offset must be an integer")
            if not isinstance(values, list):
                raise OutputComparisonError(f"output {output_id} values must be an array")

            parsed_values: list[int | float | None] = []
            for index, value in enumerate(values):
                if value is None:
                    parsed_values.append(None)
                elif isinstance(value, bool):
                    parsed_values.append(int(value))
                elif isinstance(value, (int, float)) and math.isfinite(float(value)):
                    parsed_values.append(value)
                else:
                    raise OutputComparisonError(
                        f"output {output_id} value {index} must be finite numeric or null"
                    )
            outputs.append(RuntimeOutput(output_id, kind, title, offset, parsed_values))

    outputs.sort(key=lambda output: output.output_id)
    ids = [output.output_id for output in outputs]
    if len(ids) != len(set(ids)):
        raise OutputComparisonError("runtime output ids must be unique")
    return outputs


def load_tradingview_csv(path: Path) -> tuple[list[str], list[list[str]]]:
    try:
        with path.open(newline="", encoding="utf-8-sig") as handle:
            reader = csv.reader(handle)
            header = next(reader, None)
            rows = list(reader)
    except OSError as exc:
        raise OutputComparisonError(f"failed to read TradingView CSV {path}: {exc}") from exc
    if header is None:
        raise OutputComparisonError("TradingView CSV is missing a header row")
    if not rows:
        raise OutputComparisonError("TradingView CSV contains no bars")
    for line_number, row in enumerate(rows, start=2):
        if len(row) != len(header):
            raise OutputComparisonError(
                f"TradingView CSV line {line_number} has {len(row)} columns; "
                f"expected {len(header)}"
            )
    return header, rows


def parse_tv_value(value: str, *, line_number: int, column: int) -> float | None:
    value = value.strip()
    if not value or value.lower() in {"na", "nan", "null"}:
        return None
    try:
        parsed = float(value)
    except ValueError as exc:
        raise OutputComparisonError(
            f"TradingView CSV line {line_number} column {column} is not numeric"
        ) from exc
    if not math.isfinite(parsed):
        raise OutputComparisonError(
            f"TradingView CSV line {line_number} column {column} is not finite"
        )
    return parsed


def shifted_value(output: RuntimeOutput, bar_index: int) -> int | float | None:
    source_index = bar_index - output.offset
    if source_index < 0 or source_index >= len(output.values):
        return None
    return output.values[source_index]


def values_match(
    kind: str,
    runtime_value: int | float | None,
    tv_value: float | None,
    *,
    absolute_tolerance: float,
    relative_tolerance: float,
) -> bool:
    if kind == "plotshape":
        if runtime_value is None and tv_value == 0.0:
            return True
        if tv_value is None and runtime_value is not None and float(runtime_value) == 0.0:
            return True
    if runtime_value is None or tv_value is None:
        return runtime_value is None and tv_value is None
    return math.isclose(
        float(runtime_value),
        tv_value,
        rel_tol=relative_tolerance,
        abs_tol=absolute_tolerance,
    )


def compare_outputs(
    runtime_outputs: list[RuntimeOutput],
    tv_header: list[str],
    tv_rows: list[list[str]],
    *,
    column_start: int,
    column_count: int | None,
    skip_bars: int,
    absolute_tolerance: float,
    relative_tolerance: float,
    drop_last_bars: int = 0,
) -> dict[str, Any]:
    if column_start < 0:
        raise OutputComparisonError("column start must be non-negative")
    if skip_bars < 0:
        raise OutputComparisonError("skip bars must be non-negative")
    if drop_last_bars < 0:
        raise OutputComparisonError("drop last bars must be non-negative")
    if absolute_tolerance < 0 or relative_tolerance < 0:
        raise OutputComparisonError("tolerances must be non-negative")
    expected_count = len(runtime_outputs) if column_count is None else column_count
    if expected_count != len(runtime_outputs):
        raise OutputComparisonError(
            f"runtime has {len(runtime_outputs)} outputs but column count is {expected_count}"
        )
    if column_start + expected_count > len(tv_header):
        raise OutputComparisonError(
            f"TradingView CSV has {len(tv_header)} columns; requested range "
            f"[{column_start}, {column_start + expected_count})"
        )
    comparison_end = len(tv_rows) - drop_last_bars
    if skip_bars >= comparison_end:
        raise OutputComparisonError(
            f"skip/drop window leaves no rows from {len(tv_rows)} TradingView bars"
        )
    for output in runtime_outputs:
        if len(output.values) != len(tv_rows):
            raise OutputComparisonError(
                f"output {output.output_id} has {len(output.values)} values but "
                f"TradingView CSV has {len(tv_rows)} bars"
            )

    output_reports: list[dict[str, Any]] = []
    total_mismatches = 0
    for output_index, output in enumerate(runtime_outputs):
        column = column_start + output_index
        mismatches = 0
        max_absolute_error = 0.0
        first_mismatch: dict[str, Any] | None = None
        for bar_index in range(skip_bars, comparison_end):
            runtime_value = shifted_value(output, bar_index)
            tv_value = parse_tv_value(
                tv_rows[bar_index][column],
                line_number=bar_index + 2,
                column=column,
            )
            if values_match(
                output.kind,
                runtime_value,
                tv_value,
                absolute_tolerance=absolute_tolerance,
                relative_tolerance=relative_tolerance,
            ):
                continue
            mismatches += 1
            if runtime_value is not None and tv_value is not None:
                max_absolute_error = max(
                    max_absolute_error, abs(float(runtime_value) - tv_value)
                )
            if first_mismatch is None:
                first_mismatch = {
                    "bar_index": bar_index,
                    "time": tv_rows[bar_index][0] if tv_rows[bar_index] else None,
                    "runtime": runtime_value,
                    "tradingview": tv_value,
                }

        total_mismatches += mismatches
        output_reports.append(
            {
                "id": output.output_id,
                "kind": output.kind,
                "runtime_title": output.title,
                "tradingview_column": column,
                "tradingview_title": tv_header[column],
                "offset": output.offset,
                "compared_bars": comparison_end - skip_bars,
                "mismatches": mismatches,
                "max_absolute_error": max_absolute_error,
                "first_mismatch": first_mismatch,
            }
        )

    return {
        "schema_version": SCHEMA_VERSION,
        "status": "passed" if total_mismatches == 0 else "failed",
        "bar_count": len(tv_rows),
        "skip_bars": skip_bars,
        "drop_last_bars": drop_last_bars,
        "compared_bars": comparison_end - skip_bars,
        "output_count": len(runtime_outputs),
        "column_start": column_start,
        "absolute_tolerance": absolute_tolerance,
        "relative_tolerance": relative_tolerance,
        "mismatches": total_mismatches,
        "outputs": output_reports,
    }


def print_text_report(report: dict[str, Any]) -> None:
    print(
        f"{report['status']}: {report['output_count']} outputs, "
        f"{report['compared_bars']} bars, {report['mismatches']} mismatches"
    )
    for output in report["outputs"]:
        if output["mismatches"] == 0:
            continue
        print(
            f"  column {output['tradingview_column']} {output['tradingview_title']!r} "
            f"<- {output['kind']} {output['id']}: {output['mismatches']} mismatches; "
            f"first={output['first_mismatch']}"
        )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "runtime_json", type=Path, help="pine-compat run JSON, or - for stdin"
    )
    parser.add_argument("tradingview_csv", type=Path, help="combined chart-data CSV")
    parser.add_argument(
        "--column-start",
        type=int,
        required=True,
        help="zero-based first TradingView indicator column",
    )
    parser.add_argument("--column-count", type=int)
    parser.add_argument("--skip-bars", type=int, default=0)
    parser.add_argument(
        "--drop-last-bars",
        type=int,
        default=0,
        help="exclude incomplete/live bars at the end of the export",
    )
    parser.add_argument("--absolute-tolerance", type=float, default=1e-9)
    parser.add_argument("--relative-tolerance", type=float, default=1e-9)
    parser.add_argument("--format", choices=("text", "json"), default="text")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        runtime_result = load_runtime_result(args.runtime_json)
        outputs = parse_runtime_outputs(runtime_result)
        header, rows = load_tradingview_csv(args.tradingview_csv)
        report = compare_outputs(
            outputs,
            header,
            rows,
            column_start=args.column_start,
            column_count=args.column_count,
            skip_bars=args.skip_bars,
            absolute_tolerance=args.absolute_tolerance,
            relative_tolerance=args.relative_tolerance,
            drop_last_bars=args.drop_last_bars,
        )
    except OutputComparisonError as exc:
        print(str(exc), file=sys.stderr)
        return 2
    if args.format == "json":
        json.dump(report, sys.stdout, indent=2, sort_keys=True)
        print()
    else:
        print_text_report(report)
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
