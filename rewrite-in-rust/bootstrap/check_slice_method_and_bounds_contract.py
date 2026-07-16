"""Check slicing method and custom-bound fixtures against legacy Python."""

from __future__ import annotations

import importlib
import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slice_method_and_bounds_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

slicer_api = importlib.import_module("inference.API.slicer_api")
slice_asr_cli = importlib.import_module("scripts.slice_asr_cli")


def parse_number(value: Any) -> float | None:
    if value is None:
        return None
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return float(value)


def encode_number(value: float) -> float | str:
    if math.isnan(value):
        return "nan"
    if math.isinf(value):
        return "inf" if value > 0 else "-inf"
    return value


def encode_bounds(value: tuple[float, float] | None) -> list[float | str] | None:
    if value is None:
        return None
    return [encode_number(value[0]), encode_number(value[1])]


def capture_call(func, *args):
    try:
        return {"ok": func(*args)}
    except Exception as exc:  # noqa: BLE001 - checker records legacy boundary errors.
        return {"err": type(exc).__name__, "message": str(exc)}


def assert_result(case_id: str, actual: dict, expected: dict) -> None:
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = case["case_id"]
        kind = case["kind"]

        if kind == "normalize_method":
            value = case["input"]
            assert_result(
                f"{case_id} line {line_number} api",
                capture_call(slicer_api.normalize_slicing_method, value),
                case["api"],
            )
            assert_result(
                f"{case_id} line {line_number} cli",
                capture_call(slice_asr_cli.normalize_slicing_method, value),
                case["cli"],
            )
            if case["api"] != case["cli"]:
                raise AssertionError(f"{case_id}: API/CLI normalization diverged in fixture")
            continue

        min_seconds = parse_number(case["min"])
        max_seconds = parse_number(case["max"])
        if kind == "cli_bounds":
            actual = capture_call(slice_asr_cli.resolve_slice_bounds, min_seconds, max_seconds)
        elif kind == "api_bounds":
            actual = capture_call(slicer_api._resolve_custom_slice_bounds, min_seconds, max_seconds)
        else:
            raise AssertionError(f"{case_id}: unknown kind {kind!r}")

        if "ok" in actual:
            actual["ok"] = encode_bounds(actual["ok"])
        assert_result(f"{case_id} line {line_number}", actual, case["expect"])


if __name__ == "__main__":
    main()
