"""Check Japanese G2P fallback fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "ja_g2p_fallback_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.LyricFA.tools import JaG2p as ja_module  # noqa: E402
from inference.LyricFA.tools.JaG2p import JaG2p  # noqa: E402


ja_module.pyopenjtalk = None


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


def classify_char(char: str) -> dict[str, Any]:
    return {
        "char": char,
        "letter": ja_module.is_letter(char),
        "special_letter": ja_module.is_special_letter(char),
        "digit": ja_module.is_digit(char),
        "numeric_like": ja_module.is_numeric_like(char),
        "kanji": ja_module.is_kanji(char),
        "kana": ja_module.is_kana(char),
        "special_kana": ja_module.is_special_kana(char),
        "japanese_symbol": ja_module.is_japanese_symbol(char),
        "japanese_char": ja_module.is_japanese_char(char),
    }


def run_case(case: dict[str, Any], g2p: JaG2p) -> dict[str, Any]:
    kind = case["kind"]
    if kind == "classify":
        return {"items": [classify_char(char) for char in case["chars"]]}
    if kind == "normalize_text":
        return {"value": JaG2p._normalize_text(case["text"])}
    if kind == "split_input":
        return {"tokens": JaG2p.split_input_string_no_regex(case["text"])}
    if kind == "split_japanese_segment":
        return {"tokens": JaG2p._split_japanese_segment(case["text"])}
    if kind == "kata_moras":
        return {
            "moras": g2p._kata2moras(case["text"]),
            "kana_moras": g2p._kata2kana_moras(case["text"]),
        }
    if kind == "convert_text":
        return {
            "value": g2p.convert(
                case["text"],
                include_tone=case.get("include_tone", False),
                convert_number=case.get("convert_number", True),
            )
        }
    if kind == "convert_list":
        return {
            "value": g2p.convert_list(
                case["input_list"],
                include_tone=case.get("include_tone", False),
                convert_number=case.get("convert_number", True),
            )
        }
    if kind == "split_romaji":
        return {"tokens": g2p.split_string_no_regex(case["text"])}
    if kind == "split_kana":
        return {"tokens": g2p.split_kana_no_regex(case["text"])}
    raise AssertionError(f"unknown kind {kind!r}")


def main() -> None:
    g2p = JaG2p()
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        assert_close(case_id, run_case(case, g2p), case["expect"])


if __name__ == "__main__":
    main()
