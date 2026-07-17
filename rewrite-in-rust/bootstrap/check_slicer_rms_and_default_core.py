"""Check RMS/default slicer fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from typing import Any

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slicer_rms_and_default_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.slicer.slicer2 import Slicer, get_rms  # noqa: E402
from inference.API.slicer_api import default_slice  # noqa: E402


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


def constructor_state(slicer: Slicer) -> dict[str, Any]:
    return {
        "sr": slicer.sr,
        "threshold": slicer.threshold,
        "hop_size": slicer.hop_size,
        "win_size": slicer.win_size,
        "min_length": slicer.min_length,
        "min_interval": slicer.min_interval,
        "max_sil_kept": slicer.max_sil_kept,
    }


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        kind = case["kind"]

        if kind == "get_rms":
            actual = encode_waveform(
                get_rms(
                    as_waveform(case["input"]),
                    frame_length=case["frame_length"],
                    hop_length=case["hop_length"],
                    pad_mode=case.get("pad_mode", "constant"),
                )
            )
        elif kind == "constructor_state":
            actual = constructor_state(Slicer(**case["params"]))
        elif kind == "constructor_error":
            try:
                Slicer(**case["params"])
            except Exception as exc:  # noqa: BLE001 - the fixture owns the exact legacy type.
                actual = {"type": type(exc).__name__, "message": str(exc)}
            else:
                raise AssertionError(f"{case_id}: expected constructor error")
        elif kind == "slice":
            actual = encode_chunks(Slicer(**case["params"]).slice(as_waveform(case["waveform"])))
        elif kind == "default_slice":
            actual = encode_chunks(default_slice(as_waveform(case["waveform"]), case["sr"]))
        else:
            raise AssertionError(f"{case_id}: unknown kind {kind!r}")

        assert_close(case_id, actual, case["expect"])


if __name__ == "__main__":
    main()
