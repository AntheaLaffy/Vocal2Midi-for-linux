"""Check heuristic slicer policy fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import math
import pathlib
import sys
from typing import Any

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_heuristic_policy_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.API import slicer_api  # noqa: E402


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


def assert_close(case_id: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {actual!r} is not a dict")
        if set(actual) != set(expected):
            raise AssertionError(f"{case_id}: keys {set(actual)!r} != {set(expected)!r}")
        for key in expected:
            assert_close(f"{case_id}.{key}", actual[key], expected[key])
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
            abs_tol=1e-6,
        ):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        return

    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    captured_slicer_calls: list[dict[str, Any]] = []
    split_calls: list[dict[str, Any]] = []
    split_outputs = [
        [as_segment(segment) for segment in group]
        for group in case.get("split_outputs", [])
    ]

    original_slicer = slicer_api.Slicer
    original_split = slicer_api._sliding_window_split

    class FakeSlicer:
        def __init__(self, **kwargs):
            captured_slicer_calls.append(kwargs)

        def slice(self, waveform):
            return [as_segment(segment) for segment in case["pre_segments"]]

    def fake_split(
        waveform,
        sr,
        min_len_sec,
        max_len_sec,
        target_threshold_db,
        frame_length,
        hop_length,
    ):
        split_calls.append(
            {
                "waveform": encode_waveform(waveform),
                "sr": sr,
                "min_len_sec": min_len_sec,
                "max_len_sec": max_len_sec,
                "target_threshold_db": target_threshold_db,
                "frame_length": frame_length,
                "hop_length": hop_length,
            }
        )
        if not split_outputs:
            raise AssertionError("unexpected sliding-window split call")
        return [dict(segment) for segment in split_outputs.pop(0)]

    slicer_api.Slicer = FakeSlicer
    slicer_api._sliding_window_split = fake_split
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            chunks = slicer_api.heuristic_slice(
                as_waveform(case.get("waveform", [0])),
                case["sr"],
                **case["config"],
            )
    finally:
        slicer_api.Slicer = original_slicer
        slicer_api._sliding_window_split = original_split

    if split_outputs:
        raise AssertionError("not all fixture split outputs were consumed")

    return {
        "chunks": encode_segments(chunks),
        "slicer_calls": captured_slicer_calls,
        "split_calls": split_calls,
    }


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        if case["kind"] != "heuristic_policy":
            raise AssertionError(f"{case_id}: unknown kind {case['kind']!r}")

        assert_close(case_id, run_case(case), case["expect"])


if __name__ == "__main__":
    main()
