"""Check quantization activation policy fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_activation_policy.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import should_apply_quantization  # noqa: E402


def parse_mode(value: str) -> str | None:
    if value == "__none__":
        return None
    if value == "__empty__":
        return ""
    if value == "__padded_dp__":
        return " dp "
    return value


def parse_bool(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise AssertionError(f"unknown bool {value!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        mode_raw, step_raw, expected_raw = line.split("\t")
        actual = should_apply_quantization(parse_mode(mode_raw), int(step_raw))
        expected = parse_bool(expected_raw)
        if actual != expected:
            raise AssertionError(
                f"line {line_number} mismatch: {actual!r} != {expected!r}"
            )


if __name__ == "__main__":
    main()
