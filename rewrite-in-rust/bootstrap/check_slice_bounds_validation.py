"""Check slice bounds fixtures against the legacy Python implementation."""

from __future__ import annotations

import math
import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "slice_bounds_validation.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from application.config import validate_slice_bounds  # noqa: E402


def parse_number(value: str) -> float:
    if value == "nan":
        return math.nan
    if value == "inf":
        return math.inf
    if value == "-inf":
        return -math.inf
    return float(value)


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        slice_min_raw, slice_max_raw, outcome, expected_message = line.split("\t")
        slice_min_sec = parse_number(slice_min_raw)
        slice_max_sec = parse_number(slice_max_raw)

        try:
            validate_slice_bounds(slice_min_sec, slice_max_sec)
        except ValueError as exc:
            if outcome != "err":
                raise AssertionError(f"line {line_number} failed unexpectedly: {exc}") from exc
            if str(exc) != expected_message:
                raise AssertionError(
                    f"line {line_number} message mismatch: {exc!s} != {expected_message}"
                ) from exc
        else:
            if outcome != "ok":
                raise AssertionError(f"line {line_number} passed unexpectedly")


if __name__ == "__main__":
    main()
