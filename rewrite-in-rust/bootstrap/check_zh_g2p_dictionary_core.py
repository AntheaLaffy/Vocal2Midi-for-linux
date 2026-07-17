"""Check Chinese G2P dictionary fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "zh_g2p_dictionary_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.LyricFA.tools.ZhG2p import ZhG2p, split_string, tone_to_normal  # noqa: E402


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


def fake_g2p(case: dict[str, Any]) -> ZhG2p:
    dicts = case["dicts"]
    g2p = object.__new__(ZhG2p)
    g2p.phrases_map = dicts.get("phrases_map", {})
    g2p.trans_dict = dicts.get("trans_dict", {})
    g2p.word_dict = dicts.get("word_dict", {})
    g2p.phrases_dict = dicts.get("phrases_dict", {})
    return g2p


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    kind = case["kind"]
    if kind == "tone_to_normal":
        return {"value": tone_to_normal(case["pinyin"], case.get("v_to_u", False))}
    if kind == "split_string":
        return {"tokens": split_string(case["text"])}
    if kind == "split_no_regex":
        return {"tokens": ZhG2p.split_string_no_regex(case["text"])}
    if kind == "convert_list":
        g2p = fake_g2p(case)
        return {
            "value": g2p.convert_list(
                case["input_list"],
                include_tone=case.get("include_tone", False),
                convert_number=case.get("convert_number", False),
            )
        }
    if kind == "convert_text":
        g2p = fake_g2p(case)
        return {
            "value": g2p.convert(
                case["text"],
                include_tone=case.get("include_tone", False),
                convert_number=case.get("convert_number", False),
            )
        }
    raise AssertionError(f"unknown kind {kind!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        case = json.loads(line)
        case_id = f"{case['case_id']} line {line_number}"
        assert_close(case_id, run_case(case), case["expect"])


if __name__ == "__main__":
    main()
