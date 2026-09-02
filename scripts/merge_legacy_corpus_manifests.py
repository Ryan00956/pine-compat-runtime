#!/usr/bin/env python3
"""Merge legacy corpus manifests while preserving each manifest's file roots."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import re

from analyze_legacy_corpus import CorpusRow, EXPECTED_SCOPES, parse_manifest
from import_legacy_corpus import REQUIRED_COLUMNS, write_tsv


class ManifestMergeError(ValueError):
    """Raised when corpus manifests cannot be merged safely."""


def absolute_input(root: Path, value: str) -> str:
    if not value:
        return ""
    path = Path(value)
    return str((path if path.is_absolute() else root / path).resolve())


def manifest_row(row: CorpusRow, *, root: Path) -> dict[str, str]:
    return {
        "id": row.item_id,
        "source_path": absolute_input(root, row.source_path),
        "declared_or_expected_version": str(row.expected_version),
        "chart_bars_path": absolute_input(root, row.chart_bars_path),
        "chart_symbol": row.chart_symbol,
        "chart_timeframe": row.chart_timeframe,
        "execution_times_path": absolute_input(root, row.execution_times_path),
        "request_data_manifest": absolute_input(root, row.request_data_manifest),
        "reference_output_path": absolute_input(root, row.reference_output_path),
        "license_class": row.license_class,
        "expected_scope": row.expected_scope,
        "notes": row.notes,
    }


def rebased_request_manifest(root: Path, value: str) -> dict[str, str]:
    manifest_path = Path(absolute_input(root, value))
    if not manifest_path.is_file():
        raise ManifestMergeError(
            f"request data manifest does not exist: {manifest_path}"
        )
    try:
        payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ManifestMergeError(
            f"failed to read request data manifest {manifest_path}: {exc}"
        ) from exc
    if not isinstance(payload, dict) or any(
        not isinstance(key, str) or not isinstance(path, str)
        for key, path in payload.items()
    ):
        raise ManifestMergeError(
            f"request data manifest {manifest_path} must map strings to strings"
        )

    rebased: dict[str, str] = {}
    for key in sorted(payload):
        bars_path = Path(absolute_input(root, payload[key]))
        if not bars_path.is_file():
            raise ManifestMergeError(
                f"request bars do not exist for {key!r}: {bars_path}"
            )
        rebased[key] = str(bars_path)
    return rebased


SAFE_SIDECAR_ID_RE = re.compile(r"[A-Za-z0-9._-]+")


def validate_sidecar_id(item_id: str) -> None:
    if not SAFE_SIDECAR_ID_RE.fullmatch(item_id) or item_id in {".", ".."}:
        raise ManifestMergeError(
            f"id {item_id!r} is not safe for a request data sidecar filename"
        )


def merge_manifests(
    manifests: list[Path],
    *,
    roots: list[Path] | None = None,
    expected_version: int | None = None,
    expected_scope: str | None = None,
    exclude_ids: set[str] | None = None,
    request_manifest_dir: Path | None = None,
) -> list[dict[str, str]]:
    if not manifests:
        raise ManifestMergeError("at least one manifest is required")
    if roots is not None and len(roots) != len(manifests):
        raise ManifestMergeError(
            "--root must be supplied once per --manifest when any root is explicit"
        )

    requested_exclusions = exclude_ids or set()
    found_exclusions: set[str] = set()
    merged: dict[str, dict[str, str]] = {}
    origins: dict[str, Path] = {}
    rebased_manifests: dict[Path, dict[str, str]] = {}
    for index, manifest in enumerate(manifests):
        resolved = manifest.resolve()
        root = roots[index].resolve() if roots is not None else resolved.parent
        for row in parse_manifest(resolved):
            if row.item_id in requested_exclusions:
                found_exclusions.add(row.item_id)
                continue
            if expected_version is not None and row.expected_version != expected_version:
                continue
            if expected_scope is not None and row.expected_scope != expected_scope:
                continue
            if row.item_id in merged:
                raise ManifestMergeError(
                    f"duplicate id {row.item_id!r} in {resolved} and {origins[row.item_id]}"
                )
            origins[row.item_id] = resolved
            output_row = manifest_row(row, root=root)
            if row.request_data_manifest and request_manifest_dir is not None:
                validate_sidecar_id(row.item_id)
                request_manifest_path = (
                    request_manifest_dir.resolve() / f"{row.item_id}.json"
                )
                rebased_manifests[request_manifest_path] = rebased_request_manifest(
                    root, row.request_data_manifest
                )
                output_row["request_data_manifest"] = str(request_manifest_path)
            merged[row.item_id] = output_row

    missing_exclusions = requested_exclusions - found_exclusions
    if missing_exclusions:
        missing = ", ".join(sorted(missing_exclusions))
        raise ManifestMergeError(f"excluded ids were not found: {missing}")
    if not merged:
        raise ManifestMergeError("manifest filters selected no rows")
    if rebased_manifests:
        assert request_manifest_dir is not None
        resolved_request_manifest_dir = request_manifest_dir.resolve()
        if resolved_request_manifest_dir.exists():
            raise ManifestMergeError(
                "refusing to overwrite request data sidecar directory "
                f"{resolved_request_manifest_dir}"
            )
        resolved_request_manifest_dir.mkdir(parents=True)
        for path in sorted(rebased_manifests):
            path.write_text(
                json.dumps(
                    rebased_manifests[path],
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )
    return [merged[item_id] for item_id in sorted(merged)]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, action="append", required=True)
    parser.add_argument(
        "--root",
        type=Path,
        action="append",
        help="file root paired by position with each --manifest",
    )
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--expected-version", type=int)
    parser.add_argument("--expected-scope", choices=sorted(EXPECTED_SCOPES))
    parser.add_argument(
        "--exclude-id",
        action="append",
        default=[],
        help="opaque id to omit; repeat only for an audited non-standalone input",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.expected_version is not None and args.expected_version not in range(1, 7):
        raise SystemExit("legacy corpus merge error: expected version must be 1 through 6")
    output = args.output.resolve()
    request_manifest_dir = output.with_name(f"{output.name}.request-data")
    if output.exists():
        raise SystemExit(f"legacy corpus merge error: refusing to overwrite {output}")
    try:
        rows = merge_manifests(
            args.manifest,
            roots=args.root,
            expected_version=args.expected_version,
            expected_scope=args.expected_scope,
            exclude_ids=set(args.exclude_id),
            request_manifest_dir=request_manifest_dir,
        )
        output.parent.mkdir(parents=True, exist_ok=True)
        write_tsv(output, REQUIRED_COLUMNS, rows)
    except (ManifestMergeError, OSError, ValueError) as exc:
        raise SystemExit(f"legacy corpus merge error: {exc}") from exc
    print(f"legacy corpus manifests merged: {len(rows)} rows into {output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
