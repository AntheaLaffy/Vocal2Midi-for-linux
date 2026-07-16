"""Check slicer segment-merge fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import importlib
import io
import json
import pathlib
import sys
from typing import Any

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_segment_merge_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

slicer_api = importlib.import_module("inference.API.slicer_api")


def as_waveform(value: Any) -> np.ndarray:
    return np.asarray(value, dtype=np.float32)


def as_segment(value: dict[str, Any]) -> dict[str, Any]:
    return {"offset": value["offset"], "waveform": as_waveform(value["waveform"])}


def encode_waveform(value: np.ndarray) -> list[Any]:
    return np.asarray(value).tolist()


def encode_segment(value: dict[str, Any]) -> dict[str, Any]:
    return {"offset": value["offset"], "waveform": encode_waveform(value["waveform"])}


def encode_segments(value: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [encode_segment(segment) for segment in value]


def quiet_call(func, *args):
    with contextlib.redirect_stdout(io.StringIO()):
        return func(*args)


def assert_result(case_id: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = case["case_id"]
        kind = case["kind"]

        if kind == "concat":
            actual = encode_waveform(
                slicer_api._concat_waveforms(as_waveform(case["a"]), as_waveform(case["b"]))
            )
        elif kind == "silence_like":
            actual = encode_waveform(
                slicer_api._silence_like(as_waveform(case["waveform"]), case["samples"])
            )
        elif kind == "merged_duration":
            actual = slicer_api._merged_duration_sec(
                as_segment(case["left"]),
                as_segment(case["right"]),
                case["sr"],
            )
        elif kind == "merge_segments":
            actual = encode_segment(
                slicer_api._merge_segments(
                    as_segment(case["left"]),
                    as_segment(case["right"]),
                    case["sr"],
                )
            )
        elif kind == "merge_short_segments":
            actual = encode_segments(
                quiet_call(
                    slicer_api._merge_short_segments,
                    [as_segment(segment) for segment in case["chunks"]],
                    case["sr"],
                    case["min_len_sec"],
                    case["max_len_sec"],
                )
            )
        elif kind == "merge_tiny_chunks":
            actual = encode_segments(
                quiet_call(
                    slicer_api._merge_tiny_chunks,
                    [as_segment(segment) for segment in case["chunks"]],
                    case["sr"],
                    case["tiny_sec"],
                )
            )
        else:
            raise AssertionError(f"{case_id}: unknown kind {kind!r}")

        assert_result(f"{case_id} line {line_number}", actual, case["expect"])


if __name__ == "__main__":
    main()
