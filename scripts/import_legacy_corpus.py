#!/usr/bin/env python3
"""Import a user-authorized Pine corpus without exposing source names in reports."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import shutil
import tempfile
from collections import Counter
from dataclasses import dataclass
from pathlib import Path

from analyze_legacy_corpus import REQUIRED_COLUMNS


ROOT = Path(__file__).resolve().parents[1]

CANONICAL_VERSION_RE = re.compile(
    r"(?m)^\ufeff?[ \t]*//@version=(?P<version>[1-9][0-9]*)[ \t]*\r?$"
)
RELAXED_VERSION_RE = re.compile(
    r"(?m)^\ufeff?[ \t]*//[ \t]*@version[ \t]*=[ \t]*"
    r"(?P<version>[1-9][0-9]*)[ \t]*\r?$"
)
DECLARATION_RE = re.compile(
    r"(?m)^[ \t]*(?P<mode>study|indicator|strategy|library)[ \t]*\("
)

SOURCE_MAP_COLUMNS = (
    "id",
    "original_relative_path",
    "source_sha256",
    "source_bytes",
    "expected_version",
    "version_directive",
    "declaration_mode",
    "license_hint",
)


class CorpusImportError(ValueError):
    """Raised when a private corpus cannot be imported safely."""


@dataclass(frozen=True)
class ImportedSource:
    item_id: str
    relative_path: str
    source_bytes: bytes
    source_sha256: str
    expected_version: int
    version_directive: str
    declaration_mode: str
    license_hint: str


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def decode_source(source_bytes: bytes, relative_path: str) -> str:
    try:
        return source_bytes.decode("utf-8-sig")
    except UnicodeDecodeError as exc:
        raise CorpusImportError(f"source is not UTF-8: {relative_path}") from exc


def classify_version(source: str) -> tuple[int, str]:
    canonical = CANONICAL_VERSION_RE.search(source)
    if canonical is not None:
        version = int(canonical.group("version"))
        return version, "canonical"
    relaxed = RELAXED_VERSION_RE.search(source)
    if relaxed is not None:
        version = int(relaxed.group("version"))
        return version, "noncanonical_runtime_detects_v1"
    return 1, "implicit_v1"


def classify_mode(source: str) -> str:
    declaration = DECLARATION_RE.search(source)
    return declaration.group("mode") if declaration is not None else "unknown"


def license_hint(source: str) -> str:
    lowered = source.lower()
    if "mozilla.org/mpl/2.0" in lowered or "mozilla public license 2.0" in lowered:
        return "mpl-2.0"
    if (
        "creativecommons.org/licenses/by-nc-sa/4.0" in lowered
        or "cc by-nc-sa 4.0" in lowered
    ):
        return "cc-by-nc-sa-4.0"
    if "gnu general public license" in lowered or "gpl-3.0" in lowered:
        return "gpl"
    if re.search(r"(?im)^\s*//\s*(?:©|copyright\b)", source):
        return "copyright-only"
    return "unspecified"


def corpus_scope(source: ImportedSource) -> str:
    if source.declaration_mode == "strategy":
        return "legacy_strategy_excluded"
    if source.declaration_mode not in {"study", "indicator"}:
        return "invalid_control"
    if source.expected_version <= 4:
        return "legacy_indicator"
    return "modern_indicator_control"


def discover_sources(source_dir: Path) -> list[ImportedSource]:
    if not source_dir.is_dir():
        raise CorpusImportError(f"source directory does not exist: {source_dir}")

    paths = sorted(
        (path for path in source_dir.rglob("*") if path.is_file()),
        key=lambda path: path.relative_to(source_dir).as_posix().casefold(),
    )
    if not paths:
        raise CorpusImportError(f"source directory has no files: {source_dir}")

    imported: list[ImportedSource] = []
    ids: dict[str, str] = {}
    hashes: dict[str, str] = {}
    for path in paths:
        relative_path = path.relative_to(source_dir).as_posix()
        source_bytes = path.read_bytes()
        if not source_bytes:
            raise CorpusImportError(f"source is empty: {relative_path}")
        source_sha256 = sha256_bytes(source_bytes)
        if source_sha256 in hashes:
            raise CorpusImportError(
                "duplicate source content: "
                f"{relative_path} and {hashes[source_sha256]}"
            )
        hashes[source_sha256] = relative_path
        item_id = f"tv-r2-{source_sha256[:16]}"
        if item_id in ids:
            raise CorpusImportError(
                f"opaque id collision: {relative_path} and {ids[item_id]}"
            )
        ids[item_id] = relative_path

        source = decode_source(source_bytes, relative_path)
        expected_version, version_directive = classify_version(source)
        imported.append(
            ImportedSource(
                item_id=item_id,
                relative_path=relative_path,
                source_bytes=source_bytes,
                source_sha256=source_sha256,
                expected_version=expected_version,
                version_directive=version_directive,
                declaration_mode=classify_mode(source),
                license_hint=license_hint(source),
            )
        )
    return sorted(imported, key=lambda source: source.item_id)


def manifest_row(source: ImportedSource) -> dict[str, str]:
    note = source.version_directive
    if source.declaration_mode == "strategy":
        note += "; strategy excluded by indicator-only scope"
    elif source.declaration_mode not in {"study", "indicator"}:
        note += "; no supported declaration found"
    return {
        "id": source.item_id,
        "source_path": f"sources/{source.item_id}.pine",
        "declared_or_expected_version": str(source.expected_version),
        "chart_bars_path": "bars.csv",
        "chart_symbol": "TEST",
        "chart_timeframe": "1",
        "request_data_manifest": "",
        "reference_output_path": "",
        "license_class": "private_user_authorized",
        "expected_scope": corpus_scope(source),
        "notes": note,
    }


def source_map_row(source: ImportedSource) -> dict[str, str]:
    return {
        "id": source.item_id,
        "original_relative_path": source.relative_path,
        "source_sha256": source.source_sha256,
        "source_bytes": str(len(source.source_bytes)),
        "expected_version": str(source.expected_version),
        "version_directive": source.version_directive,
        "declaration_mode": source.declaration_mode,
        "license_hint": source.license_hint,
    }


def write_tsv(path: Path, columns: tuple[str, ...], rows: list[dict[str, str]]) -> None:
    with path.open("w", newline="", encoding="utf-8") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=columns,
            delimiter="\t",
            lineterminator="\n",
        )
        writer.writeheader()
        writer.writerows(rows)


def write_import(
    sources: list[ImportedSource],
    *,
    output_dir: Path,
    chart_bars: Path,
) -> None:
    if output_dir.exists():
        raise CorpusImportError(
            f"output already exists; refusing to overwrite: {output_dir}"
        )
    if not chart_bars.is_file():
        raise CorpusImportError(f"chart bars file does not exist: {chart_bars}")

    output_dir.parent.mkdir(parents=True, exist_ok=True)
    staging_dir = Path(
        tempfile.mkdtemp(prefix=f".{output_dir.name}.", dir=output_dir.parent)
    )
    try:
        source_output = staging_dir / "sources"
        source_output.mkdir()
        for source in sources:
            (source_output / f"{source.item_id}.pine").write_bytes(
                source.source_bytes
            )
        shutil.copyfile(chart_bars, staging_dir / "bars.csv")
        write_tsv(
            staging_dir / "corpus.tsv",
            REQUIRED_COLUMNS,
            [manifest_row(source) for source in sources],
        )
        write_tsv(
            staging_dir / "source-map.tsv",
            SOURCE_MAP_COLUMNS,
            [source_map_row(source) for source in sources],
        )

        version_counts = Counter(source.expected_version for source in sources)
        scope_counts = Counter(corpus_scope(source) for source in sources)
        directive_counts = Counter(source.version_directive for source in sources)
        mode_counts = Counter(source.declaration_mode for source in sources)
        summary = {
            "corpusItems": len(sources),
            "directives": dict(sorted(directive_counts.items())),
            "modes": dict(sorted(mode_counts.items())),
            "scopes": dict(sorted(scope_counts.items())),
            "versions": {
                str(version): count for version, count in sorted(version_counts.items())
            },
        }
        (staging_dir / "intake-summary.json").write_text(
            json.dumps(summary, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
        staging_dir.rename(output_dir)
    except BaseException:
        shutil.rmtree(staging_dir, ignore_errors=True)
        raise


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument(
        "--chart-bars",
        type=Path,
        default=ROOT / "tests/fixtures/legacy/chart_1m.csv",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        sources = discover_sources(args.source_dir.resolve())
        write_import(
            sources,
            output_dir=args.output_dir.resolve(),
            chart_bars=args.chart_bars.resolve(),
        )
    except (CorpusImportError, OSError) as exc:
        raise SystemExit(f"legacy corpus import error: {exc}") from exc
    print(f"legacy corpus imported: {len(sources)} items into {args.output_dir.resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
