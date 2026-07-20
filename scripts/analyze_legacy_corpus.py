#!/usr/bin/env python3
"""Build a deterministic, privacy-preserving legacy indicator corpus report."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import re
import subprocess
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterable, Mapping, Sequence


SCHEMA_VERSION = 2
TOOL_VERSION = 2
ROOT = Path(__file__).resolve().parents[1]

STABLE_MIN_ELIGIBLE_SCRIPTS = 50
STABLE_MIN_PARSE_RATE = 0.95
STABLE_MIN_ANALYZE_LOWER_RATE = 0.85
STABLE_MIN_HISTORICAL_RUN_RATE = 0.80
FAILURE_CLUSTER_DISPOSITION_SHARE = 0.02

REQUIRED_COLUMNS = (
    "id",
    "source_path",
    "declared_or_expected_version",
    "chart_bars_path",
    "chart_symbol",
    "chart_timeframe",
    "request_data_manifest",
    "reference_output_path",
    "license_class",
    "expected_scope",
    "notes",
)

EXPECTED_SCOPES = {
    "legacy_indicator",
    "modern_indicator_control",
    "legacy_strategy_excluded",
    "invalid_control",
}

LICENSE_CLASSES = {
    "original",
    "user_owned",
    "permissive",
    "private_user_authorized",
}

ELIGIBLE_SCOPE = "legacy_indicator"
EXCLUDED_STRATEGY_SCOPE = "legacy_strategy_excluded"
CONTROL_SCOPES = {"modern_indicator_control", "invalid_control"}

STAGE_PASSED = "passed"
STAGE_FAILED = "failed"
STAGE_NOT_RUN = "not_run"
STAGE_EXCLUDED = "excluded"
STAGE_MISSING_INPUT = "missing_input"

DIAGNOSTIC_RE = re.compile(
    r"^(?P<code>[EW]_[A-Z0-9_]+):(?P<severity>[A-Za-z]+):"
    r"(?P<line>[0-9]+):(?P<column>[0-9]+): (?P<message>.*)$"
)
VERSION_RE = re.compile(r"(?m)^\s*//@version=(?P<version>[0-9]+)\s*$")
DECLARATION_RE = re.compile(r"(?m)^\s*(?P<mode>study|indicator|strategy)\s*\(")

KNOWN_LEGACY_FEATURES = {
    "abs",
    "atr",
    "avg",
    "barssince",
    "bb",
    "blue",
    "cci",
    "ceil",
    "change",
    "color",
    "cross",
    "crossover",
    "crossunder",
    "dotted",
    "ema",
    "floor",
    "histogram",
    "highest",
    "heikinashi",
    "iff",
    "integer",
    "interval",
    "isintraday",
    "lowest",
    "linreg",
    "log",
    "log10",
    "max",
    "macd",
    "mfi",
    "min",
    "mom",
    "n",
    "obv",
    "offset",
    "period",
    "pivothigh",
    "pivotlow",
    "pow",
    "red",
    "rma",
    "round",
    "rsi",
    "security",
    "sign",
    "sma",
    "sqrt",
    "stdev",
    "stoch",
    "study",
    "sum",
    "ticker",
    "tickerid",
    "tostring",
    "tr",
    "valuewhen",
    "vwap",
    "transp",
    "vwma",
    "wma",
}

SAFE_CALL_ARGUMENTS = {
    "input": {"minval", "type"},
    "plot": {"series", "style", "transp"},
}

CANONICAL_CANDIDATES = {
    "abs": "math.abs",
    "atr": "ta.atr",
    "avg": "math.avg",
    "barssince": "ta.barssince",
    "bb": "ta.bb",
    "blue": "color.blue",
    "cci": "ta.cci",
    "ceil": "math.ceil",
    "change": "ta.change",
    "color": "color.new",
    "cross": "ta.cross",
    "crossover": "ta.crossover",
    "crossunder": "ta.crossunder",
    "dotted": "hline.style_dotted",
    "ema": "ta.ema",
    "floor": "math.floor",
    "highest": "ta.highest",
    "heikinashi": "ticker.heikinashi",
    "histogram": "plot.style_histogram",
    "iff": "conditional_expression",
    "input.minval": "input.int.minval",
    "input.type": "typed_input_call",
    "integer": "input.int",
    "interval": "timeframe.multiplier",
    "isintraday": "timeframe.isintraday",
    "lowest": "ta.lowest",
    "linreg": "ta.linreg",
    "log": "math.log",
    "log10": "math.log10",
    "max": "math.max",
    "macd": "ta.macd",
    "mfi": "ta.mfi",
    "min": "math.min",
    "mom": "ta.mom",
    "n": "bar_index",
    "obv": "ta.obv",
    "offset": "history_reference",
    "period": "timeframe.period",
    "pivothigh": "ta.pivothigh",
    "pivotlow": "ta.pivotlow",
    "plot.style": "plot.style_*",
    "plot.transp": "color.new",
    "pow": "math.pow",
    "red": "color.red",
    "rma": "ta.rma",
    "round": "math.round",
    "rsi": "ta.rsi",
    "security": "request.security",
    "sign": "math.sign",
    "sma": "ta.sma",
    "sqrt": "math.sqrt",
    "stdev": "ta.stdev",
    "stoch": "ta.stoch",
    "study": "indicator",
    "sum": "math.sum",
    "ticker": "syminfo.ticker",
    "tickerid": "syminfo.tickerid",
    "tostring": "str.tostring",
    "tr": "ta.tr",
    "valuewhen": "ta.valuewhen",
    "vwap": "ta.vwap",
    "vwma": "ta.vwma",
    "wma": "ta.wma",
}

SUBJECT_PATTERNS = (
    re.compile(r"unknown function `(?P<subject>[^`]+)`"),
    re.compile(r"unknown symbol `(?P<subject>[^`]+)`"),
    re.compile(r"unknown color constant `(?P<subject>[^`]+)`"),
    re.compile(
        r"`(?P<call>[^`]+)` has no argument named `(?P<argument>[^`]+)`"
    ),
    re.compile(
        r"`(?P<call>[^`]+)` argument `(?P<argument>[^`]+)` expects"
    ),
)


class CorpusError(ValueError):
    """Raised when the corpus manifest or an input bundle is malformed."""


@dataclass(frozen=True)
class CorpusRow:
    item_id: str
    source_path: str
    expected_version: int
    chart_bars_path: str
    chart_symbol: str
    chart_timeframe: str
    request_data_manifest: str
    reference_output_path: str
    license_class: str
    expected_scope: str
    notes: str


@dataclass(frozen=True)
class DiagnosticRecord:
    code: str
    severity: str
    line: int
    column: int
    subject: str | None
    feature_category: str
    canonical_candidate: str | None

    def as_dict(self, stage: str) -> dict[str, object]:
        output: dict[str, object] = {
            "stage": stage,
            "code": self.code,
            "severity": self.severity.lower(),
            "line": self.line,
            "column": self.column,
        }
        if self.subject is not None:
            output["subject"] = self.subject
        output["featureCategory"] = self.feature_category
        if self.canonical_candidate is not None:
            output["canonicalCandidate"] = self.canonical_candidate
        return output


CommandRunner = Callable[[Sequence[str], Path], subprocess.CompletedProcess[str]]


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def parse_manifest(path: Path) -> list[CorpusRow]:
    try:
        with path.open(newline="", encoding="utf-8") as handle:
            reader = csv.DictReader(handle, delimiter="\t")
            if tuple(reader.fieldnames or ()) != REQUIRED_COLUMNS:
                raise CorpusError(
                    "legacy corpus manifest columns must be exactly: "
                    + ", ".join(REQUIRED_COLUMNS)
                )
            raw_rows = list(reader)
    except OSError as exc:
        raise CorpusError(f"failed to read corpus manifest {path}: {exc}") from exc

    rows: list[CorpusRow] = []
    seen_ids: set[str] = set()
    for line_number, raw in enumerate(raw_rows, start=2):
        item_id = raw["id"].strip()
        if not item_id:
            raise CorpusError(f"manifest line {line_number}: id is required")
        if item_id in seen_ids:
            raise CorpusError(f"manifest line {line_number}: duplicate id {item_id!r}")
        seen_ids.add(item_id)

        try:
            expected_version = int(raw["declared_or_expected_version"])
        except ValueError as exc:
            raise CorpusError(
                f"manifest line {line_number}: expected version must be an integer"
            ) from exc
        if expected_version not in range(1, 7):
            raise CorpusError(
                f"manifest line {line_number}: expected version must be 1 through 6"
            )

        expected_scope = raw["expected_scope"].strip()
        if expected_scope not in EXPECTED_SCOPES:
            raise CorpusError(
                f"manifest line {line_number}: unknown expected_scope {expected_scope!r}"
            )
        license_class = raw["license_class"].strip()
        if license_class not in LICENSE_CLASSES:
            raise CorpusError(
                f"manifest line {line_number}: unknown license_class {license_class!r}"
            )
        source_path = raw["source_path"].strip()
        if not source_path:
            raise CorpusError(f"manifest line {line_number}: source_path is required")

        rows.append(
            CorpusRow(
                item_id=item_id,
                source_path=source_path,
                expected_version=expected_version,
                chart_bars_path=raw["chart_bars_path"].strip(),
                chart_symbol=raw["chart_symbol"].strip(),
                chart_timeframe=raw["chart_timeframe"].strip(),
                request_data_manifest=raw["request_data_manifest"].strip(),
                reference_output_path=raw["reference_output_path"].strip(),
                license_class=license_class,
                expected_scope=expected_scope,
                notes=raw["notes"].strip(),
            )
        )

    ids = [row.item_id for row in rows]
    if ids != sorted(ids):
        raise CorpusError("legacy corpus manifest rows must be sorted by id")
    return rows


def resolve_path(root: Path, value: str) -> Path:
    path = Path(value)
    return path if path.is_absolute() else root / path


def detected_version(source: str) -> int:
    match = VERSION_RE.search(source)
    return int(match.group("version")) if match else 1


def detected_mode(source: str) -> str:
    match = DECLARATION_RE.search(source)
    return match.group("mode") if match else "unknown"


def sanitized_subject(message: str) -> str | None:
    for pattern in SUBJECT_PATTERNS:
        match = pattern.search(message)
        if match is None:
            continue
        groups = match.groupdict()
        if groups.get("call") and groups.get("argument"):
            call = groups["call"]
            argument = groups["argument"]
            if argument in SAFE_CALL_ARGUMENTS.get(call, set()):
                return f"{call}.{argument}"
            return None
        candidate = groups.get("subject")
        if candidate in KNOWN_LEGACY_FEATURES:
            return candidate
        return None
    return None


def feature_category(code: str, subject: str | None) -> str:
    if subject == "study":
        return "declaration"
    if subject == "security":
        return "request_alias"
    if subject in {
        "bb",
        "atr",
        "barssince",
        "change",
        "cci",
        "cross",
        "crossover",
        "crossunder",
        "ema",
        "highest",
        "lowest",
        "linreg",
        "macd",
        "mfi",
        "mom",
        "obv",
        "pivothigh",
        "pivotlow",
        "rma",
        "rsi",
        "sma",
        "stdev",
        "stoch",
        "valuewhen",
        "tr",
        "vwap",
        "vwma",
        "wma",
    }:
        return "ta_alias"
    if subject in {
        "abs",
        "avg",
        "ceil",
        "floor",
        "log",
        "log10",
        "max",
        "min",
        "pow",
        "round",
        "sign",
        "sqrt",
        "sum",
    }:
        return "math_alias"
    if subject == "heikinashi":
        return "ticker_alias"
    if subject == "tostring":
        return "string_alias"
    if subject in {"blue", "color", "red"}:
        return "color_compatibility"
    if subject in {"dotted", "histogram", "plot.style", "plot.transp"}:
        return "output_option"
    if subject in {"input.minval", "input.type", "integer"}:
        return "input_overload"
    if subject in {"interval", "isintraday", "n", "period", "ticker", "tickerid"}:
        return "legacy_metadata"
    if subject in {"iff", "offset"}:
        return "legacy_semantics"
    if code.startswith("E_PARSE_") or code.startswith("E_LEX_"):
        return "syntax"
    if code.startswith("E_CALL_"):
        return "call_shape"
    if code.startswith("E_ASSIGN_"):
        return "legacy_type_rule"
    if code.startswith("E_OPERATOR_"):
        return "operator_type"
    if code == "E_LEGACY_INDICATOR_DECLARATION":
        return "legacy_declaration"
    if code == "E_LEGACY_STRATEGY_OUT_OF_SCOPE":
        return "scope_exclusion"
    if code.startswith("E_LANGUAGE_VERSION_"):
        return "version_policy"
    if code in {"E_UNKNOWN_FUNCTION", "E_UNKNOWN_SYMBOL", "E_UNKNOWN_COLOR"}:
        return "name_resolution"
    if code == "E_UNSUPPORTED_FEATURE":
        return "known_unsupported"
    return "unclassified"


def parse_diagnostics(*outputs: str) -> list[DiagnosticRecord]:
    diagnostics: list[DiagnosticRecord] = []
    seen: set[tuple[str, str, int, int, str | None, str, str | None]] = set()
    for output in outputs:
        for line in output.splitlines():
            match = DIAGNOSTIC_RE.match(line.strip())
            if match is None:
                continue
            code = match.group("code")
            subject = sanitized_subject(match.group("message"))
            record = DiagnosticRecord(
                code=code,
                severity=match.group("severity"),
                line=int(match.group("line")),
                column=int(match.group("column")),
                subject=subject,
                feature_category=feature_category(code, subject),
                canonical_candidate=CANONICAL_CANDIDATES.get(subject or ""),
            )
            key = (
                record.code,
                record.severity,
                record.line,
                record.column,
                record.subject,
                record.feature_category,
                record.canonical_candidate,
            )
            if key not in seen:
                diagnostics.append(record)
                seen.add(key)
    return diagnostics


def is_syntax_diagnostic(code: str) -> bool:
    return code.startswith("E_LEX_") or code.startswith("E_PARSE_")


def default_command_runner(
    command: Sequence[str], root: Path
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        list(command),
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )


def stage(status: str, **details: object) -> dict[str, object]:
    return {"status": status, **details}


def optional_file_status(root: Path, value: str) -> str:
    if not value:
        return "not_supplied"
    return STAGE_PASSED if resolve_path(root, value).is_file() else STAGE_MISSING_INPUT


def request_specs(root: Path, manifest_value: str) -> tuple[str, list[str]]:
    if not manifest_value:
        return "not_supplied", []
    manifest_path = resolve_path(root, manifest_value)
    if not manifest_path.is_file():
        return STAGE_MISSING_INPUT, []
    try:
        payload = json.loads(manifest_path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise CorpusError(f"invalid request data manifest {manifest_path}: {exc}") from exc
    if not isinstance(payload, dict) or not all(
        isinstance(key, str) and isinstance(value, str)
        for key, value in payload.items()
    ):
        raise CorpusError(
            f"request data manifest {manifest_path} must be a string-to-string object"
        )

    specs: list[str] = []
    for key in sorted(payload):
        bars_path = resolve_path(root, payload[key])
        if not bars_path.is_file():
            return STAGE_MISSING_INPUT, []
        specs.append(f"{key}={bars_path}")
    return STAGE_PASSED, specs


def compare_reference_output(path: Path, runtime_output: object) -> str:
    if not path.is_file():
        return STAGE_MISSING_INPUT
    try:
        expected = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise CorpusError(f"invalid reference output {path}: {exc}") from exc
    return STAGE_PASSED if expected == runtime_output else STAGE_FAILED


def _base_item(row: CorpusRow) -> dict[str, object]:
    return {
        "id": row.item_id,
        "expectedScope": row.expected_scope,
        "classifiedScope": row.expected_scope,
        "expectedVersion": row.expected_version,
        "licenseClass": row.license_class,
        "chartContext": {
            "symbolProvided": bool(row.chart_symbol),
            "timeframeProvided": bool(row.chart_timeframe),
        },
    }


def analyze_item(
    row: CorpusRow,
    *,
    root: Path,
    pine_compat: Path,
    command_runner: CommandRunner,
) -> dict[str, object]:
    item = _base_item(row)
    source_path = resolve_path(root, row.source_path)
    provider_status, provider_specs = request_specs(root, row.request_data_manifest)
    inputs = {
        "source": optional_file_status(root, row.source_path),
        "chartBars": optional_file_status(root, row.chart_bars_path),
        "requestData": provider_status,
        "referenceOutput": optional_file_status(root, row.reference_output_path),
    }
    item["inputAvailability"] = inputs
    stages: dict[str, dict[str, object]] = {
        "discovered": stage(STAGE_PASSED),
    }
    item["stages"] = stages

    if not source_path.is_file():
        stages["sourceRead"] = stage(STAGE_MISSING_INPUT)
        for name in (
            "versionDetected",
            "modeClassified",
            "parse",
            "analyze",
            "lower",
            "historicalRun",
            "incrementalRun",
            "realtimeRun",
            "outputCompared",
        ):
            stages[name] = stage(STAGE_NOT_RUN)
        item["diagnostics"] = []
        return item

    try:
        source_bytes = source_path.read_bytes()
        source = source_bytes.decode("utf-8")
    except (OSError, UnicodeDecodeError):
        stages["sourceRead"] = stage(STAGE_FAILED)
        for name in (
            "versionDetected",
            "modeClassified",
            "parse",
            "analyze",
            "lower",
            "historicalRun",
            "incrementalRun",
            "realtimeRun",
            "outputCompared",
        ):
            stages[name] = stage(STAGE_NOT_RUN)
        item["diagnostics"] = []
        return item

    item["sourceSha256"] = sha256_bytes(source_bytes)
    stages["sourceRead"] = stage(STAGE_PASSED)
    version = detected_version(source)
    mode = detected_mode(source)
    item["detectedVersion"] = version
    item["versionMatchesExpected"] = version == row.expected_version
    item["detectedMode"] = mode
    if mode == "strategy":
        item["classifiedScope"] = EXCLUDED_STRATEGY_SCOPE
    item["scopeMatchesExpected"] = (
        item["classifiedScope"] == item["expectedScope"]
    )
    stages["versionDetected"] = stage(STAGE_PASSED)
    stages["modeClassified"] = stage(STAGE_PASSED)

    if item["classifiedScope"] == EXCLUDED_STRATEGY_SCOPE:
        for name in (
            "parse",
            "analyze",
            "lower",
            "historicalRun",
            "incrementalRun",
            "realtimeRun",
            "outputCompared",
        ):
            stages[name] = stage(STAGE_EXCLUDED)
        item["diagnostics"] = []
        return item

    analyze_command = [str(pine_compat), "analyze", str(source_path)]
    analyzed = command_runner(analyze_command, root)
    diagnostics = parse_diagnostics(analyzed.stdout, analyzed.stderr)
    syntax_errors = [record for record in diagnostics if is_syntax_diagnostic(record.code)]
    stages["parse"] = stage(STAGE_FAILED if syntax_errors else STAGE_PASSED)
    analyze_status = STAGE_PASSED if analyzed.returncode == 0 else STAGE_FAILED
    stages["analyze"] = stage(analyze_status, returnCode=analyzed.returncode)
    stages["lower"] = stage(
        STAGE_PASSED if analyze_status == STAGE_PASSED else STAGE_NOT_RUN
    )
    item["diagnostics"] = [record.as_dict("analyze") for record in diagnostics]

    if analyze_status != STAGE_PASSED:
        stages["historicalRun"] = stage(STAGE_NOT_RUN)
        stages["incrementalRun"] = stage(STAGE_NOT_RUN)
        stages["realtimeRun"] = stage(STAGE_NOT_RUN)
        stages["outputCompared"] = stage(STAGE_NOT_RUN)
        return item

    if inputs["chartBars"] != STAGE_PASSED:
        stages["historicalRun"] = stage(STAGE_MISSING_INPUT)
        stages["incrementalRun"] = stage(STAGE_NOT_RUN)
        stages["realtimeRun"] = stage(STAGE_NOT_RUN)
        stages["outputCompared"] = stage(STAGE_NOT_RUN)
        return item
    chart_bars_path = resolve_path(root, row.chart_bars_path)
    if provider_status == STAGE_MISSING_INPUT:
        stages["historicalRun"] = stage(STAGE_MISSING_INPUT)
        stages["incrementalRun"] = stage(STAGE_NOT_RUN)
        stages["realtimeRun"] = stage(STAGE_NOT_RUN)
        stages["outputCompared"] = stage(STAGE_NOT_RUN)
        return item

    run_command = [
        str(pine_compat),
        "run",
        str(source_path),
        "--bars",
        str(chart_bars_path),
    ]
    if row.chart_symbol:
        run_command.extend(("--chart-symbol", row.chart_symbol))
    if row.chart_timeframe:
        run_command.extend(("--chart-timeframe", row.chart_timeframe))
    for spec in provider_specs:
        run_command.extend(("--request-bars", spec))
    executed = command_runner(run_command, root)
    runtime_diagnostics = parse_diagnostics(executed.stdout, executed.stderr)
    if runtime_diagnostics:
        item["diagnostics"].extend(
            record.as_dict("run") for record in runtime_diagnostics
        )

    runtime_output: object | None = None
    if executed.returncode == 0:
        try:
            runtime_output = json.loads(executed.stdout)
        except json.JSONDecodeError:
            stages["historicalRun"] = stage(
                STAGE_FAILED, returnCode=executed.returncode, errorKind="invalid_json"
            )
        else:
            stages["historicalRun"] = stage(STAGE_PASSED, returnCode=0)
    else:
        error_kind = (
            "missing_provider_data"
            if "missing request data" in executed.stderr
            else "runtime_or_host_error"
        )
        stages["historicalRun"] = stage(
            STAGE_FAILED,
            returnCode=executed.returncode,
            errorKind=error_kind,
        )

    stages["incrementalRun"] = stage(STAGE_NOT_RUN)
    stages["realtimeRun"] = stage(STAGE_NOT_RUN)
    if runtime_output is None or inputs["referenceOutput"] == "not_supplied":
        stages["outputCompared"] = stage(STAGE_NOT_RUN)
    elif inputs["referenceOutput"] == STAGE_MISSING_INPUT:
        stages["outputCompared"] = stage(STAGE_MISSING_INPUT)
    else:
        reference_path = resolve_path(root, row.reference_output_path)
        stages["outputCompared"] = stage(
            compare_reference_output(reference_path, runtime_output)
        )
    return item


def _metric(items: Iterable[Mapping[str, object]], stage_name: str) -> dict[str, object]:
    selected = list(items)
    statuses = Counter(
        str(item["stages"][stage_name]["status"])  # type: ignore[index]
        for item in selected
    )
    attempted = statuses[STAGE_PASSED] + statuses[STAGE_FAILED]
    rate = statuses[STAGE_PASSED] / attempted if attempted else None
    eligible_rate = statuses[STAGE_PASSED] / len(selected) if selected else None
    output: dict[str, object] = {
        "passed": statuses[STAGE_PASSED],
        "failed": statuses[STAGE_FAILED],
        "notRun": statuses[STAGE_NOT_RUN],
        "missingInput": statuses[STAGE_MISSING_INPUT],
        "excluded": statuses[STAGE_EXCLUDED],
        "attempted": attempted,
        "successRate": rate,
        "eligibleSuccessRate": eligible_rate,
    }
    return output


def _reference_output_gate(items: Sequence[Mapping[str, object]]) -> dict[str, object]:
    supplied = 0
    missing = 0
    compared_passed = 0
    compared_failed = 0
    compared_not_run = 0
    for item in items:
        availability = str(item["inputAvailability"]["referenceOutput"])  # type: ignore[index]
        comparison = str(item["stages"]["outputCompared"]["status"])  # type: ignore[index]
        if availability == STAGE_PASSED:
            supplied += 1
            if comparison == STAGE_PASSED:
                compared_passed += 1
            elif comparison == STAGE_FAILED:
                compared_failed += 1
            else:
                compared_not_run += 1
        elif availability == STAGE_MISSING_INPUT:
            missing += 1

    return {
        "supplied": supplied,
        "passed": compared_passed,
        "failed": compared_failed,
        "notRun": compared_not_run,
        "missingInput": missing,
        "availableOutputsPass": (
            missing == 0
            and compared_failed == 0
            and compared_not_run == 0
            and compared_passed == supplied
        ),
    }


def _rate_threshold(
    metric: Mapping[str, object], required_rate: float
) -> dict[str, object]:
    actual_rate = metric["eligibleSuccessRate"]
    return {
        "actual": actual_rate,
        "required": required_rate,
        "met": actual_rate is not None and float(actual_rate) >= required_rate,
    }


def _stable_baseline_gate(
    items: Sequence[Mapping[str, object]],
    version: int,
    clusters: Sequence[Mapping[str, object]],
) -> dict[str, object]:
    parse_gate = _rate_threshold(
        _metric(items, "parse"), STABLE_MIN_PARSE_RATE
    )
    analyze_metric = _metric(items, "analyze")
    lower_metric = _metric(items, "lower")
    analyze_lower_rate = min(
        float(analyze_metric["eligibleSuccessRate"] or 0.0),
        float(lower_metric["eligibleSuccessRate"] or 0.0),
    )
    analyze_lower_gate = {
        "actual": analyze_lower_rate if items else None,
        "required": STABLE_MIN_ANALYZE_LOWER_RATE,
        "met": bool(items) and analyze_lower_rate >= STABLE_MIN_ANALYZE_LOWER_RATE,
    }
    historical_gate = _rate_threshold(
        _metric(items, "historicalRun"), STABLE_MIN_HISTORICAL_RUN_RATE
    )
    reference_gate = _reference_output_gate(items)
    unknown_clusters = sum(
        1
        for cluster in clusters
        if str(cluster["code"])
        in {"E_UNKNOWN_FUNCTION", "E_UNKNOWN_SYMBOL", "E_UNKNOWN_COLOR"}
        and float(cluster.get("eligibleShareByVersion", {}).get(str(version), 0.0))
        >= FAILURE_CLUSTER_DISPOSITION_SHARE
    )
    corpus_gate = {
        "actual": len(items),
        "required": STABLE_MIN_ELIGIBLE_SCRIPTS,
        "remaining": max(0, STABLE_MIN_ELIGIBLE_SCRIPTS - len(items)),
        "met": len(items) >= STABLE_MIN_ELIGIBLE_SCRIPTS,
    }

    blocking_reasons: list[str] = []
    if not corpus_gate["met"]:
        blocking_reasons.append("insufficient_eligible_scripts")
    if not parse_gate["met"]:
        blocking_reasons.append("parse_rate_below_threshold")
    if not analyze_lower_gate["met"]:
        blocking_reasons.append("analyze_lower_rate_below_threshold")
    if not historical_gate["met"]:
        blocking_reasons.append("historical_run_rate_below_threshold")
    if not reference_gate["availableOutputsPass"]:
        blocking_reasons.append("available_reference_output_failed_or_not_run")
    if unknown_clusters:
        blocking_reasons.append("unknown_failure_cluster_requires_disposition")

    return {
        "thresholdsMet": not blocking_reasons,
        "blockingReasons": blocking_reasons,
        "eligibleScripts": corpus_gate,
        "parseSuccessRate": parse_gate,
        "analyzeLowerSuccessRate": analyze_lower_gate,
        "historicalRunSuccessRate": historical_gate,
        "referenceOutputs": reference_gate,
        "unknownClustersRequiringDisposition": unknown_clusters,
        "fullExecutionAuditStillRequired": True,
    }


def _summary(items: list[dict[str, object]]) -> dict[str, object]:
    eligible = [item for item in items if item["classifiedScope"] == ELIGIBLE_SCOPE]
    controls = [item for item in items if item["classifiedScope"] in CONTROL_SCOPES]
    excluded = [
        item for item in items if item["classifiedScope"] == EXCLUDED_STRATEGY_SCOPE
    ]
    stages = {
        name: _metric(eligible, name)
        for name in (
            "sourceRead",
            "parse",
            "analyze",
            "lower",
            "historicalRun",
            "incrementalRun",
            "realtimeRun",
            "outputCompared",
        )
    }

    version_items: dict[int, list[dict[str, object]]] = defaultdict(list)
    for item in eligible:
        version_items[int(item["expectedVersion"])].append(item)
    unknown_count = 0
    unsupported_count = 0
    input_availability_counts: dict[str, Counter[str]] = defaultdict(Counter)
    cluster_counts: Counter[
        tuple[str, str, str, str | None, str | None, int]
    ] = Counter()
    cluster_items: dict[
        tuple[str, str, str, str | None, str | None, int], set[str]
    ] = defaultdict(set)
    for item in eligible:
        version = int(item["expectedVersion"])
        for input_name, status in item["inputAvailability"].items():  # type: ignore[union-attr]
            input_availability_counts[input_name][str(status)] += 1
        for diagnostic in item.get("diagnostics", []):
            code = str(diagnostic["code"])
            if code in {"E_UNKNOWN_FUNCTION", "E_UNKNOWN_SYMBOL", "E_UNKNOWN_COLOR"}:
                unknown_count += 1
            if code == "E_UNSUPPORTED_FEATURE":
                unsupported_count += 1
            cluster_key = (
                str(diagnostic["stage"]),
                code,
                str(diagnostic["featureCategory"]),
                diagnostic.get("subject"),
                diagnostic.get("canonicalCandidate"),
                version,
            )
            cluster_counts[cluster_key] += 1
            cluster_items[cluster_key].add(str(item["id"]))

    merged_clusters: dict[
        tuple[str, str, str, str | None, str | None], dict[str, object]
    ] = {}
    for (
        stage_name,
        code,
        category,
        subject,
        canonical_candidate,
        version,
    ), count in cluster_counts.items():
        key = (stage_name, code, category, subject, canonical_candidate)
        cluster = merged_clusters.setdefault(
            key,
            {
                "stage": stage_name,
                "code": code,
                "featureCategory": category,
                "count": 0,
                "versions": set(),
                "affectedItems": set(),
                "affectedItemsByVersion": defaultdict(set),
            },
        )
        cluster["count"] = int(cluster["count"]) + count
        cluster["versions"].add(version)  # type: ignore[union-attr]
        affected = cluster_items[
            (stage_name, code, category, subject, canonical_candidate, version)
        ]
        cluster["affectedItems"].update(affected)  # type: ignore[union-attr]
        cluster["affectedItemsByVersion"][version].update(affected)  # type: ignore[index,union-attr]
        if subject is not None:
            cluster["subject"] = subject
        if canonical_candidate is not None:
            cluster["canonicalCandidate"] = canonical_candidate

    top_clusters = []
    for cluster in merged_clusters.values():
        cluster["versions"] = sorted(cluster["versions"])  # type: ignore[arg-type]
        affected_items = cluster.pop("affectedItems")
        affected_by_version = cluster.pop("affectedItemsByVersion")
        cluster["affectedScripts"] = len(affected_items)
        cluster["eligibleShare"] = (
            len(affected_items) / len(eligible) if eligible else 0.0
        )
        cluster["affectedScriptsByVersion"] = {
            str(version): len(affected_by_version[version])
            for version in sorted(affected_by_version)
        }
        cluster["eligibleShareByVersion"] = {
            str(version): len(affected_by_version[version]) / len(version_items[version])
            for version in sorted(affected_by_version)
        }
        cluster["requiresDisposition"] = any(
            share >= FAILURE_CLUSTER_DISPOSITION_SHARE
            for share in cluster["eligibleShareByVersion"].values()  # type: ignore[union-attr]
        )
        top_clusters.append(cluster)
    top_clusters.sort(
        key=lambda cluster: (
            -int(cluster["affectedScripts"]),
            -int(cluster["count"]),
            str(cluster["stage"]),
            str(cluster["code"]),
            str(cluster["featureCategory"]),
            str(cluster.get("subject", "")),
        )
    )

    versions = {
        str(version): {
            "eligible": len(selected),
            "parse": _metric(selected, "parse"),
            "analyze": _metric(selected, "analyze"),
            "historicalRun": _metric(selected, "historicalRun"),
            "stableBaseline": _stable_baseline_gate(
                selected, version, top_clusters
            ),
        }
        for version, selected in sorted(version_items.items())
    }

    return {
        "corpusItems": len(items),
        "eligibleLegacyIndicators": len(eligible),
        "excludedLegacyStrategies": len(excluded),
        "controls": len(controls),
        "scopeMismatchCount": sum(
            item.get("scopeMatchesExpected") is False for item in items
        ),
        "stages": stages,
        "versions": versions,
        "unknownDiagnosticCount": unknown_count,
        "knownUnsupportedDiagnosticCount": unsupported_count,
        "stableBaselinePolicy": {
            "minimumEligibleScripts": STABLE_MIN_ELIGIBLE_SCRIPTS,
            "minimumParseSuccessRate": STABLE_MIN_PARSE_RATE,
            "minimumAnalyzeLowerSuccessRate": STABLE_MIN_ANALYZE_LOWER_RATE,
            "minimumHistoricalRunSuccessRate": STABLE_MIN_HISTORICAL_RUN_RATE,
            "failureClusterDispositionShare": FAILURE_CLUSTER_DISPOSITION_SHARE,
            "fullExecutionAuditRequiredAfterThresholds": True,
        },
        "inputAvailability": {
            name: {
                status: input_availability_counts[name][status]
                for status in (STAGE_PASSED, "not_supplied", STAGE_MISSING_INPUT)
            }
            for name in ("source", "chartBars", "requestData", "referenceOutput")
        },
        "missingInputCounts": {
            name: input_availability_counts[name][STAGE_MISSING_INPUT]
            for name in ("source", "chartBars", "requestData", "referenceOutput")
        },
        "topFailureClusters": top_clusters[:20],
    }


def build_report(
    rows: Sequence[CorpusRow],
    *,
    root: Path,
    manifest_path: Path,
    pine_compat: Path,
    build_revision: str,
    command_runner: CommandRunner = default_command_runner,
) -> dict[str, object]:
    items = [
        analyze_item(
            row,
            root=root,
            pine_compat=pine_compat,
            command_runner=command_runner,
        )
        for row in rows
    ]
    return {
        "schemaVersion": SCHEMA_VERSION,
        "toolVersion": TOOL_VERSION,
        "buildRevision": build_revision,
        "manifestSha256": sha256_file(manifest_path),
        "toolSha256": sha256_file(Path(__file__)),
        "privacy": {
            "sourceTextIncluded": False,
            "sourcePathsIncluded": False,
            "timestampsIncluded": False,
        },
        "summary": _summary(items),
        "items": items,
    }


def render_report(report: Mapping[str, object]) -> str:
    return json.dumps(report, indent=2, sort_keys=True) + "\n"


def git_revision(root: Path) -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        return "unknown"
    revision = completed.stdout.strip()
    dirty = subprocess.run(
        ["git", "status", "--porcelain"],
        cwd=root,
        text=True,
        capture_output=True,
        check=False,
    )
    return revision + ("+dirty" if dirty.stdout.strip() else "")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "tests/fixtures/legacy/corpus.tsv",
    )
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument(
        "--pine-compat",
        type=Path,
        default=ROOT / "target/debug/pine-compat",
    )
    parser.add_argument("--build-revision")
    parser.add_argument("--output", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = args.root.resolve()
    manifest_path = args.manifest.resolve()
    pine_compat = args.pine_compat.resolve()
    if not pine_compat.is_file():
        raise SystemExit(
            f"legacy corpus error: pine-compat binary not found at {pine_compat}; "
            "run `cargo build -p pine-cli` first"
        )
    try:
        rows = parse_manifest(manifest_path)
        report = build_report(
            rows,
            root=root,
            manifest_path=manifest_path,
            pine_compat=pine_compat,
            build_revision=args.build_revision or git_revision(root),
        )
    except CorpusError as exc:
        raise SystemExit(f"legacy corpus error: {exc}") from exc

    rendered = render_report(report)
    if args.output is None:
        print(rendered, end="")
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(rendered, encoding="utf-8")
        print(
            "legacy corpus report passed: "
            f"{report['summary']['corpusItems']} items; wrote {args.output}"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
