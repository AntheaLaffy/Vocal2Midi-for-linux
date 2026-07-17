"""Check slicer RMS-dB/window-split fixtures against legacy Python."""

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
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_rms_db_window_split_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.API.slicer_api import _sliding_window_split, get_rms_db  # noqa: E402


def as_waveform(value: Any) -> np.ndarray:
    return np.asarray(value, dtype=np.float32)


def encode_waveform(value: np.ndarray) -> Any:
    return np.asarray(value).tolist()


def encode_chunks(chunks: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [
        {
            "offset": chunk["offset"],
            "waveform": encode_waveform(chunk["waveform"]),
        }
        for chunk in chunks
    ]


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


def run_sliding_window(case: dict[str, Any]) -> list[dict[str, Any]]:
    with contextlib.redirect_stdout(io.StringIO()):
        return _sliding_window_split(
            as_waveform(case["waveform"]),
            case["sr"],
            case["min_len_sec"],
            case["max_len_sec"],
            case["target_threshold_db"],
            case["frame_length"],
            case["hop_length"],
        )


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        kind = case["kind"]

        if kind == "rms_db":
            actual = encode_waveform(
                get_rms_db(
                    as_waveform(case["waveform"]),
                    frame_length=case["frame_length"],
                    hop_length=case["hop_length"],
                )
            )
        elif kind == "sliding_window_split":
            actual = encode_chunks(run_sliding_window(case))
        elif kind == "sliding_window_error":
            try:
                run_sliding_window(case)
            except Exception as exc:  # noqa: BLE001 - fixture owns legacy exception type.
                actual = {"type": type(exc).__name__, "message": str(exc)}
            else:
                raise AssertionError(f"{case_id}: expected legacy exception")
        else:
            raise AssertionError(f"{case_id}: unknown kind {kind!r}")

        assert_close(case_id, actual, case["expect"])


if __name__ == "__main__":
    main()
