#!/usr/bin/env python3
"""Normalize a TradingView chart-data CSV to the runtime OHLCV contract."""

from __future__ import annotations

import argparse
import csv
import math
from pathlib import Path


REQUIRED_COLUMNS = ("time", "open", "high", "low", "close", "volume")


class TradingViewBarsError(ValueError):
    """Raised when an export cannot be normalized without guessing."""


def normalized_column_map(fieldnames: list[str] | None) -> dict[str, str]:
    if fieldnames is None:
        raise TradingViewBarsError("TradingView export is missing a header row")
    by_normalized_name: dict[str, list[str]] = {}
    for name in fieldnames:
        by_normalized_name.setdefault(name.strip().lower(), []).append(name)

    missing = [name for name in REQUIRED_COLUMNS if name not in by_normalized_name]
    if missing:
        raise TradingViewBarsError(
            "TradingView export is missing required column(s): " + ", ".join(missing)
        )
    duplicates = [
        name for name in REQUIRED_COLUMNS if len(by_normalized_name[name]) != 1
    ]
    if duplicates:
        raise TradingViewBarsError(
            "TradingView export has duplicate required column(s): "
            + ", ".join(duplicates)
        )
    return {name: by_normalized_name[name][0] for name in REQUIRED_COLUMNS}


def parse_integer_time(value: str, line_number: int) -> int:
    try:
        return int(value.strip())
    except ValueError as exc:
        raise TradingViewBarsError(
            f"line {line_number}: time must be an integer Unix timestamp"
        ) from exc


def parse_finite(value: str, name: str, line_number: int) -> float:
    try:
        parsed = float(value.strip())
    except ValueError as exc:
        raise TradingViewBarsError(
            f"line {line_number}: {name} must be numeric"
        ) from exc
    if not math.isfinite(parsed):
        raise TradingViewBarsError(f"line {line_number}: {name} must be finite")
    return parsed


def normalize_rows(
    input_path: Path, *, time_unit: str
) -> list[tuple[str, str, str, str, str, str]]:
    try:
        handle = input_path.open(newline="", encoding="utf-8-sig")
    except OSError as exc:
        raise TradingViewBarsError(
            f"failed to read TradingView export {input_path}: {exc}"
        ) from exc

    rows: list[tuple[str, str, str, str, str, str]] = []
    previous_time: int | None = None
    with handle:
        reader = csv.DictReader(handle)
        columns = normalized_column_map(reader.fieldnames)
        for line_number, raw in enumerate(reader, start=2):
            source_time = parse_integer_time(raw[columns["time"]], line_number)
            time = source_time * 1000 if time_unit == "seconds" else source_time
            if previous_time is not None and time <= previous_time:
                relation = "duplicate" if time == previous_time else "unsorted"
                raise TradingViewBarsError(
                    f"line {line_number}: {relation} bar timestamp {time}"
                )

            values = {
                name: raw[columns[name]].strip() for name in REQUIRED_COLUMNS[1:]
            }
            parsed = {
                name: parse_finite(value, name, line_number)
                for name, value in values.items()
            }
            if parsed["high"] < max(parsed["open"], parsed["close"], parsed["low"]):
                raise TradingViewBarsError(
                    f"line {line_number}: high is below another OHLC value"
                )
            if parsed["low"] > min(parsed["open"], parsed["close"], parsed["high"]):
                raise TradingViewBarsError(
                    f"line {line_number}: low is above another OHLC value"
                )

            rows.append(
                (
                    str(time),
                    values["open"],
                    values["high"],
                    values["low"],
                    values["close"],
                    values["volume"],
                )
            )
            previous_time = time
    if not rows:
        raise TradingViewBarsError("TradingView export contains no bars")
    return rows


def write_rows(
    output_path: Path,
    rows: list[tuple[str, str, str, str, str, str]],
    *,
    force: bool,
) -> None:
    if output_path.exists() and not force:
        raise TradingViewBarsError(
            f"refusing to overwrite existing output {output_path}; pass --force"
        )
    output_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        with output_path.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.writer(handle, lineterminator="\n")
            writer.writerow(REQUIRED_COLUMNS)
            writer.writerows(rows)
    except OSError as exc:
        raise TradingViewBarsError(
            f"failed to write normalized bars {output_path}: {exc}"
        ) from exc


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path, help="TradingView chart-data CSV")
    parser.add_argument("output", type=Path, help="runtime six-column OHLCV CSV")
    parser.add_argument(
        "--time-unit",
        choices=("seconds", "milliseconds"),
        default="seconds",
        help="input Unix timestamp unit (default: seconds)",
    )
    parser.add_argument("--force", action="store_true", help="overwrite output")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        rows = normalize_rows(args.input, time_unit=args.time_unit)
        write_rows(args.output, rows, force=args.force)
    except TradingViewBarsError as exc:
        raise SystemExit(str(exc)) from exc
    print(f"normalized {len(rows)} bars to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
