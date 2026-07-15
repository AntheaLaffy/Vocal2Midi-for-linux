"""Check runtime device normalization fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "runtime_device_normalization.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

import inference.device_utils as device_utils  # noqa: E402


def parse_optional_value(value: str) -> str | None:
    if value == "__none__":
        return None
    if value == "__empty__":
        return ""
    if value == "__space__":
        return "   "
    if value == "__padded_directml__":
        return " DirectML "
    if value == "__padded_unknown__":
        return " Metal "
    return value


def main() -> None:
    original_is_windows = device_utils._IS_WINDOWS
    try:
        for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
            if not line or line.startswith("#"):
                continue

            platform_name, device_raw, default_raw, expected = line.split("\t")
            if platform_name == "windows":
                device_utils._IS_WINDOWS = True
            elif platform_name == "other":
                device_utils._IS_WINDOWS = False
            else:
                raise AssertionError(f"line {line_number} has unknown platform {platform_name!r}")

            device = parse_optional_value(device_raw)
            default = parse_optional_value(default_raw)
            actual = device_utils.normalize_runtime_device(device, default=default)
            if actual != expected:
                raise AssertionError(
                    f"line {line_number} mismatch: {actual!r} != {expected!r}"
                )
    finally:
        device_utils._IS_WINDOWS = original_is_windows


if __name__ == "__main__":
    main()
