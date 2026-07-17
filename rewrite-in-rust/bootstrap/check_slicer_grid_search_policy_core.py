"""Check grid-search slicer policy fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import math
import pathlib
import re
import sys
from typing import Any

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_grid_search_policy_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.API import slicer_api  # noqa: E402


SCORE_RE = re.compile(
    r"Trying params: threshold=(-?\d+)dB, min_length=(\d+)ms -> "
    r"Score=([0-9.]+) \((\d+) chunks, (\d+) short, (\d+) long\)"
)
BEST_RE = re.compile(r"Found best slicer params: threshold=(-?\d+)dB, min_length=(\d+)ms")


def as_waveform(value: Any) -> np.ndarray:
    return np.asarray(value, dtype=np.float32)


def as_segment(value: dict[str, Any]) -> dict[str, Any]:
    return {"offset": value["offset"], "waveform": as_waveform(value["waveform"])}


def encode_waveform(value: np.ndarray) -> Any:
    return np.asarray(value).tolist()


def encode_segment(value: dict[str, Any]) -> dict[str, Any]:
    return {"offset": value["offset"], "waveform": encode_waveform(value["waveform"])}


def encode_segments(value: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [encode_segment(segment) for segment in value]


def expected_call_pairs_from(expect: dict[str, Any]) -> list[list[int]] | None:
    source = expect.get("slicer_call_pairs_from")
    if source is None:
        return None
    return [
        [threshold, min_length]
        for threshold in source["thresholds"]
        for min_length in source["min_lengths_ms"]
    ]


def assert_close(case_id: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {actual!r} is not a dict")
        for key, expected_value in expected.items():
            if key == "slicer_call_pairs_from":
                expanded = expected_call_pairs_from(expected)
                assert_close(f"{case_id}.slicer_call_pairs", actual["slicer_call_pairs"], expanded)
                continue
            if key not in actual:
                raise AssertionError(f"{case_id}: missing key {key!r}")
            assert_close(f"{case_id}.{key}", actual[key], expected_value)
        return

    if isinstance(expected, list):
        if not isinstance(actual, list) or len(actual) != len(expected):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected, strict=True)):
            assert_close(f"{case_id}[{index}]", actual_item, expected_item)
        return

    if isinstance(expected, float):
        if not isinstance(actual, (float, int)) or not math.isclose(
            float(actual),
            expected,
            rel_tol=1e-6,
            abs_tol=1e-2,
        ):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        return

    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def parse_score_log(stdout: str) -> list[dict[str, Any]]:
    return [
        {
            "params": [int(match.group(1)), int(match.group(2))],
            "score": float(match.group(3)),
            "chunks": int(match.group(4)),
            "short": int(match.group(5)),
            "long": int(match.group(6)),
        }
        for match in SCORE_RE.finditer(stdout)
    ]


def parse_best_params(stdout: str) -> list[int] | None:
    match = BEST_RE.search(stdout)
    if match is None:
        return None
    return [int(match.group(1)), int(match.group(2))]


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    captured_calls: list[dict[str, Any]] = []
    outcomes = {tuple(outcome["params"]): outcome for outcome in case.get("outcomes", [])}
    default_outcome = case.get("default_outcome", {"chunks": []})

    original_slicer = slicer_api.Slicer

    class FakeSlicer:
        def __init__(self, **kwargs):
            self.params = (kwargs["threshold"], kwargs["min_length"])
            captured_calls.append(kwargs)
            outcome = outcomes.get(self.params, default_outcome)
            if outcome.get("error") and outcome.get("phase") == "init":
                raise RuntimeError(outcome["error"])

        def slice(self, waveform):
            outcome = outcomes.get(self.params, default_outcome)
            if outcome.get("error"):
                raise RuntimeError(outcome["error"])
            return [as_segment(segment) for segment in outcome.get("chunks", [])]

    slicer_api.Slicer = FakeSlicer
    try:
        captured_stdout = io.StringIO()
        with contextlib.redirect_stdout(captured_stdout):
            chunks = slicer_api.grid_search_slice(
                as_waveform(case["waveform"]),
                case["sr"],
                **case["config"],
            )
    finally:
        slicer_api.Slicer = original_slicer

    call_pairs = [[call["threshold"], call["min_length"]] for call in captured_calls]
    stdout = captured_stdout.getvalue()
    return {
        "chunks": encode_segments(chunks),
        "best_params": parse_best_params(stdout),
        "best_score": min((entry["score"] for entry in parse_score_log(stdout)), default=None),
        "call_count": len(captured_calls),
        "slicer_call_pairs": call_pairs,
        "slicer_call_static": {
            "sr_values": sorted({call["sr"] for call in captured_calls}),
            "min_interval_values": sorted({call["min_interval"] for call in captured_calls}),
            "max_sil_kept_values": sorted({call["max_sil_kept"] for call in captured_calls}),
        },
        "score_log": parse_score_log(stdout),
    }


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        if case["kind"] != "grid_search_policy":
            raise AssertionError(f"{case_id}: unknown kind {case['kind']!r}")

        assert_close(case_id, run_case(case), case["expect"])


if __name__ == "__main__":
    main()
