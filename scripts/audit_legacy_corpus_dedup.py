#!/usr/bin/env python3
"""Audit version-aware duplicate content across two legacy corpus manifests."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
from dataclasses import dataclass
from pathlib import Path

from analyze_legacy_corpus import CorpusError, CorpusRow, detected_version, parse_manifest


ROOT = Path(__file__).resolve().parents[1]
SCHEMA_VERSION = 1
TOOL_VERSION = 1
DEFAULT_STABLE_PROFILE_FLOOR = 50
NUMBER_RE = re.compile(r"(?:\d+(?:\.\d*)?|\.\d+)(?:[eE][+-]?\d+)?")


class DedupAuditError(ValueError):
    """Raised when a dedup input cannot be measured safely."""


@dataclass(frozen=True)
class FingerprintedSource:
    origin: str
    item_id: str
    expected_version: int
    source_sha256: str
    normalized_source_sha256: str
    token_fingerprint_sha256: str
    token_count: int

    def public_json(self) -> dict[str, object]:
        return {
            "origin": self.origin,
            "id": self.item_id,
            "expectedVersion": self.expected_version,
            "sourceSha256": self.source_sha256,
            "normalizedSourceSha256": self.normalized_source_sha256,
            "tokenFingerprintSha256": self.token_fingerprint_sha256,
            "tokenCount": self.token_count,
        }


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def decode_source(source_bytes: bytes, item_id: str) -> str:
    try:
        return source_bytes.decode("utf-8-sig")
    except UnicodeDecodeError as exc:
        raise DedupAuditError(f"{item_id}: source is not UTF-8") from exc


def normalized_source_bytes(source: str) -> bytes:
    lines = [line.rstrip() for line in source.replace("\r\n", "\n").replace("\r", "\n").split("\n")]
    while lines and not lines[0]:
        lines.pop(0)
    while lines and not lines[-1]:
        lines.pop()
    return ("\n".join(lines) + "\n").encode("utf-8")


def source_tokens(source: str, item_id: str) -> list[tuple[str, str]]:
    """Return a trivia-free token stream without interpreting Pine semantics."""

    source = source.replace("\r\n", "\n").replace("\r", "\n")
    tokens: list[tuple[str, str]] = []
    index = 0
    while index < len(source):
        char = source[index]
        if char.isspace():
            index += 1
            continue
        if source.startswith("//", index):
            newline = source.find("\n", index + 2)
            index = len(source) if newline < 0 else newline + 1
            continue
        if char in {'"', "'"}:
            quote = char
            end = index + 1
            while end < len(source):
                if source[end] == "\\":
                    end += 2
                elif source[end] == quote:
                    end += 1
                    break
                else:
                    end += 1
            if end > len(source) or source[end - 1] != quote:
                raise DedupAuditError(f"{item_id}: unterminated string in token audit")
            tokens.append(("string", source[index:end]))
            index = end
            continue
        if char.isalpha() or char == "_":
            end = index + 1
            while end < len(source) and (
                source[end].isalnum() or source[end] == "_"
            ):
                end += 1
            tokens.append(("identifier", source[index:end]))
            index = end
            continue
        if char.isdigit() or (
            char == "." and index + 1 < len(source) and source[index + 1].isdigit()
        ):
            match = NUMBER_RE.match(source, index)
            if match is None:
                raise DedupAuditError(f"{item_id}: invalid numeric token")
            tokens.append(("number", match.group(0)))
            index = match.end()
            continue
        tokens.append(("symbol", char))
        index += 1
    return tokens


def token_fingerprint(source: str, version: int, item_id: str) -> tuple[str, int]:
    tokens = source_tokens(source, item_id)
    encoded = json.dumps(
        {"version": version, "tokens": tokens},
        ensure_ascii=False,
        separators=(",", ":"),
        sort_keys=True,
    ).encode("utf-8")
    return sha256_bytes(encoded), len(tokens)


def fingerprint_source(
    row: CorpusRow,
    *,
    origin: str,
    root: Path,
) -> FingerprintedSource:
    path = Path(row.source_path)
    path = path if path.is_absolute() else root / path
    try:
        source_bytes = path.read_bytes()
    except OSError as exc:
        raise DedupAuditError(f"{row.item_id}: failed to read source") from exc
    source = decode_source(source_bytes, row.item_id)
    actual_version = detected_version(source)
    if actual_version != row.expected_version:
        raise DedupAuditError(
            f"{row.item_id}: manifest version {row.expected_version} "
            f"does not match detected version {actual_version}"
        )
    token_hash, token_count = token_fingerprint(
        source, row.expected_version, row.item_id
    )
    return FingerprintedSource(
        origin=origin,
        item_id=row.item_id,
        expected_version=row.expected_version,
        source_sha256=sha256_bytes(source_bytes),
        normalized_source_sha256=sha256_bytes(normalized_source_bytes(source)),
        token_fingerprint_sha256=token_hash,
        token_count=token_count,
    )


def selected_sources(
    manifest: Path,
    *,
    origin: str,
    root: Path,
    version: int,
) -> list[FingerprintedSource]:
    rows = [
        row
        for row in parse_manifest(manifest)
        if row.expected_scope == "legacy_indicator" and row.expected_version == version
    ]
    if not rows:
        raise DedupAuditError(
            f"{origin} manifest has no legacy_indicator rows for version {version}"
        )
    return [fingerprint_source(row, origin=origin, root=root) for row in rows]


def strongest_match(
    baseline: FingerprintedSource, candidate: FingerprintedSource
) -> str | None:
    if baseline.source_sha256 == candidate.source_sha256:
        return "exact"
    if baseline.normalized_source_sha256 == candidate.normalized_source_sha256:
        return "normalized"
    if baseline.token_fingerprint_sha256 == candidate.token_fingerprint_sha256:
        return "token_equivalent"
    return None


def build_report(
    *,
    baseline_manifest: Path,
    baseline_root: Path,
    candidate_manifest: Path,
    candidate_root: Path,
    version: int,
    stable_profile_floor: int,
    build_revision: str,
) -> dict[str, object]:
    baseline = selected_sources(
        baseline_manifest,
        origin="baseline",
        root=baseline_root,
        version=version,
    )
    candidate = selected_sources(
        candidate_manifest,
        origin="candidate",
        root=candidate_root,
        version=version,
    )
    matches = []
    for baseline_item in baseline:
        for candidate_item in candidate:
            match_level = strongest_match(baseline_item, candidate_item)
            if match_level is not None:
                matches.append(
                    {
                        "baselineId": baseline_item.item_id,
                        "candidateId": candidate_item.item_id,
                        "matchLevel": match_level,
                    }
                )
    matches.sort(
        key=lambda item: (
            str(item["baselineId"]),
            str(item["candidateId"]),
            str(item["matchLevel"]),
        )
    )

    baseline_unique = len(
        {item.token_fingerprint_sha256 for item in baseline}
    )
    candidate_unique = len(
        {item.token_fingerprint_sha256 for item in candidate}
    )
    combined_unique = len(
        {item.token_fingerprint_sha256 for item in [*baseline, *candidate]}
    )
    matched_candidate_ids = {str(item["candidateId"]) for item in matches}
    match_counts = {
        level: sum(item["matchLevel"] == level for item in matches)
        for level in ("exact", "normalized", "token_equivalent")
    }

    return {
        "schemaVersion": SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "toolSha256": sha256_file(Path(__file__)),
        "buildRevision": build_revision,
        "selection": {
            "expectedScope": "legacy_indicator",
            "expectedVersion": version,
            "stableProfileFloor": stable_profile_floor,
        },
        "privacy": {
            "sourceContentIncluded": False,
            "sourcePathsIncluded": False,
            "sourceTitlesIncluded": False,
            "candidateIdsOpaque": True,
            "fingerprintsAreOneWaySha256": True,
        },
        "manifests": {
            "baselineSha256": sha256_file(baseline_manifest),
            "candidateSha256": sha256_file(candidate_manifest),
        },
        "summary": {
            "baselineSelected": len(baseline),
            "baselineUnique": baseline_unique,
            "candidateSelected": len(candidate),
            "candidateUnique": candidate_unique,
            "candidateContribution": combined_unique - baseline_unique,
            "combinedSelected": len(baseline) + len(candidate),
            "combinedUnique": combined_unique,
            "crossManifestMatches": len(matches),
            "crossManifestMatchedCandidates": len(matched_candidate_ids),
            "exactMatches": match_counts["exact"],
            "normalizedMatches": match_counts["normalized"],
            "tokenEquivalentMatches": match_counts["token_equivalent"],
            "stableProfileFloor": stable_profile_floor,
            "stableProfileFloorReached": combined_unique >= stable_profile_floor,
            "stableProfileFloorRemaining": max(
                stable_profile_floor - combined_unique, 0
            ),
        },
        "crossManifestMatches": matches,
        "items": [
            item.public_json()
            for item in sorted(
                [*baseline, *candidate],
                key=lambda item: (item.origin, item.item_id),
            )
        ],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--baseline-manifest",
        type=Path,
        default=ROOT / "tests/fixtures/legacy/corpus.tsv",
    )
    parser.add_argument("--baseline-root", type=Path, default=ROOT)
    parser.add_argument("--candidate-manifest", type=Path, required=True)
    parser.add_argument("--candidate-root", type=Path)
    parser.add_argument("--version", type=int, default=4)
    parser.add_argument(
        "--stable-profile-floor", type=int, default=DEFAULT_STABLE_PROFILE_FLOOR
    )
    parser.add_argument("--build-revision", required=True)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    if args.version < 1:
        parser.error("--version must be positive")
    if args.stable_profile_floor < 1:
        parser.error("--stable-profile-floor must be positive")
    if not args.build_revision.strip():
        parser.error("--build-revision must not be empty")

    baseline_manifest = args.baseline_manifest.resolve()
    candidate_manifest = args.candidate_manifest.resolve()
    candidate_root = (
        args.candidate_root.resolve()
        if args.candidate_root is not None
        else candidate_manifest.parent
    )
    try:
        report = build_report(
            baseline_manifest=baseline_manifest,
            baseline_root=args.baseline_root.resolve(),
            candidate_manifest=candidate_manifest,
            candidate_root=candidate_root,
            version=args.version,
            stable_profile_floor=args.stable_profile_floor,
            build_revision=args.build_revision.strip(),
        )
    except (CorpusError, DedupAuditError) as exc:
        parser.error(str(exc))

    encoded = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if args.output is None:
        print(encoded, end="")
    else:
        args.output.write_text(encoded, encoding="utf-8")
        summary = report["summary"]
        assert isinstance(summary, dict)
        print(
            "legacy corpus dedup audit passed: "
            f"{summary['combinedUnique']} unique v{args.version} scripts; "
            f"{summary['stableProfileFloorRemaining']} remaining to stable floor"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
