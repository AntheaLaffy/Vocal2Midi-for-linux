"""Check supplied-voiced-mask pitch slicer fixtures against legacy Python."""

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
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_pitch_override_core.jsonl"

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


def optional_bool_array(value: Any) -> np.ndarray | None:
    if value is None:
        return None
    return np.asarray(value, dtype=bool)


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


def run_voiced_override(case: dict[str, Any]) -> dict[str, Any]:
    f0, voiced_flag = slicer_api.get_pitch_curve(
        as_waveform(case["waveform"]),
        sr=case["sr"],
        hop_length=case["hop_length"],
        voiced_flag_override=np.asarray(case["voiced_flag_override"], dtype=bool),
        voiced_flag_override_step_sec=case["voiced_flag_override_step_sec"],
        segment_offset_sec=case["segment_offset_sec"],
    )
    return {"voiced_flag": voiced_flag.tolist(), "f0": f0.tolist()}


def run_pitch_split(case: dict[str, Any]) -> dict[str, Any]:
    original_rms_db = slicer_api.get_rms_db
    original_pitch_curve = slicer_api.get_pitch_curve

    def fake_rms_db(waveform, frame_length=2048, hop_length=512):
        return np.asarray(case["rms_db"], dtype=np.float32)

    def fake_pitch_curve(
        waveform,
        sr,
        hop_length=512,
        f0_min=65.0,
        f0_max=1100.0,
        voiced_flag_override=None,
        voiced_flag_override_step_sec=None,
        segment_offset_sec=0.0,
    ):
        voiced = np.asarray(case["direct_voiced_flag"], dtype=bool)
        return np.zeros_like(voiced, dtype=np.float32), voiced

    if "rms_db" in case:
        slicer_api.get_rms_db = fake_rms_db
    if "direct_voiced_flag" in case:
        slicer_api.get_pitch_curve = fake_pitch_curve

    try:
        with contextlib.redirect_stdout(io.StringIO()):
            chunks = slicer_api._pitch_based_split(
                as_waveform(case["waveform"]),
                sr=case["sr"],
                min_len_sec=case["min_len_sec"],
                max_len_sec=case["max_len_sec"],
                hop_length=case["hop_length"],
                voiced_flag_override=optional_bool_array(case["voiced_flag_override"]),
                voiced_flag_override_step_sec=case["voiced_flag_override_step_sec"],
                segment_offset_sec=case["segment_offset_sec"],
            )
    finally:
        slicer_api.get_rms_db = original_rms_db
        slicer_api.get_pitch_curve = original_pitch_curve

    return {"chunks": encode_segments(chunks)}


class SequentialExecutor:
    def __enter__(self):
        return self

    def __exit__(self, exc_type, exc, tb):
        return False

    def map(self, func, items):
        return [func(item) for item in items]


def run_pitch_policy(case: dict[str, Any]) -> dict[str, Any]:
    captured_slicer_calls: list[dict[str, Any]] = []
    split_calls: list[dict[str, Any]] = []
    split_outputs = [
        [as_segment(segment) for segment in group]
        for group in case.get("split_outputs", [])
    ]

    original_slicer = slicer_api.Slicer
    original_split = slicer_api._pitch_based_split
    original_executor = slicer_api.ProcessPoolExecutor

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
        hop_length,
        voiced_flag_override=None,
        voiced_flag_override_step_sec=None,
        segment_offset_sec=0.0,
    ):
        split_calls.append(
            {
                "waveform": encode_waveform(waveform),
                "sr": sr,
                "min_len_sec": min_len_sec,
                "max_len_sec": max_len_sec,
                "hop_length": hop_length,
                "voiced_flag_override": None
                if voiced_flag_override is None
                else np.asarray(voiced_flag_override, dtype=bool).tolist(),
                "voiced_flag_override_step_sec": voiced_flag_override_step_sec,
                "segment_offset_sec": segment_offset_sec,
            }
        )
        if not split_outputs:
            raise AssertionError("unexpected pitch split call")
        return [dict(segment) for segment in split_outputs.pop(0)]

    slicer_api.Slicer = FakeSlicer
    slicer_api._pitch_based_split = fake_split
    slicer_api.ProcessPoolExecutor = SequentialExecutor
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            chunks = slicer_api.pitch_based_slice(
                as_waveform(case.get("waveform", [0])),
                case["sr"],
                min_len_sec=case["config"]["min_len_sec"],
                max_len_sec=case["config"]["max_len_sec"],
                silence_removal_threshold_db=case["config"]["silence_removal_threshold_db"],
                min_silence_len_ms=case["config"]["min_silence_len_ms"],
                ultra_short_sec=case["config"]["ultra_short_sec"],
                voiced_flag_override=None
                if case["config"]["voiced_flag_override"] is None
                else np.asarray(case["config"]["voiced_flag_override"], dtype=bool),
                voiced_flag_override_step_sec=case["config"]["voiced_flag_override_step_sec"],
            )
    finally:
        slicer_api.Slicer = original_slicer
        slicer_api._pitch_based_split = original_split
        slicer_api.ProcessPoolExecutor = original_executor

    if split_outputs:
        raise AssertionError("not all fixture split outputs were consumed")

    return {
        "chunks": encode_segments(chunks),
        "slicer_calls": captured_slicer_calls,
        "split_calls": split_calls,
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    if case["kind"] == "voiced_override":
        return run_voiced_override(case)
    if case["kind"] == "pitch_split":
        return run_pitch_split(case)
    if case["kind"] == "pitch_policy":
        return run_pitch_policy(case)
    raise AssertionError(f"unknown kind {case['kind']!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        assert_close(case_id, run_case(case), case["expect"])


if __name__ == "__main__":
    main()
