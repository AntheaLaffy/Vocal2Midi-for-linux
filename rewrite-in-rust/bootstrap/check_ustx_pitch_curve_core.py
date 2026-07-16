"""Check USTX pitch curve fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from dataclasses import dataclass

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "ustx_pitch_curve_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.API.rmvpe_api import RmvpeResult  # noqa: E402
from inference.API.ustx_api import _build_pitd_curve  # noqa: E402


@dataclass
class Note:
    onset: float
    offset: float
    pitch: float


def parse_number(value: str) -> float:
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return float(value)


def parse_notes(raw_notes: list[dict[str, str]]) -> list[Note]:
    return [
        Note(
            onset=parse_number(raw_note["onset"]),
            offset=parse_number(raw_note["offset"]),
            pitch=parse_number(raw_note["pitch"]),
        )
        for raw_note in raw_notes
    ]


def parse_pitch(values: list[str]) -> np.ndarray:
    return np.array([parse_number(value) for value in values], dtype=np.float32)


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        result = RmvpeResult(
            time_step_seconds=float(case["time_step_seconds"]),
            midi_pitch=parse_pitch(case["midi_pitch"]),
            voiced_mask=None,
        )
        xs, ys = _build_pitd_curve(parse_notes(case["notes"]), result, float(case["tempo"]))

        if xs != case["expected_xs"]:
            raise AssertionError(
                f"{case['case_id']} line {line_number}: xs mismatch: {xs!r} != {case['expected_xs']!r}"
            )
        if ys != case["expected_ys"]:
            raise AssertionError(
                f"{case['case_id']} line {line_number}: ys mismatch: {ys!r} != {case['expected_ys']!r}"
            )
        if any(not math.isfinite(value) for value in xs + ys):
            raise AssertionError(f"{case['case_id']} line {line_number}: non-finite output")


if __name__ == "__main__":
    main()
