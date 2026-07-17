"""Check lyric language processor fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "lyric_language_processor_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.LyricFA.tools import JaG2p as ja_module  # noqa: E402
from inference.LyricFA.tools.ZhG2p import ZhG2p  # noqa: E402

ja_module.pyopenjtalk = None

from inference.LyricFA.tools.language_processors import (  # noqa: E402
    ChineseProcessor,
    JapaneseProcessor,
    ProcessorFactory,
)


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


def fake_zh_g2p(dicts: dict[str, Any]) -> ZhG2p:
    g2p = object.__new__(ZhG2p)
    g2p.phrases_map = dicts.get("phrases_map", {})
    g2p.trans_dict = dicts.get("trans_dict", {})
    g2p.word_dict = dicts.get("word_dict", {})
    g2p.phrases_dict = dicts.get("phrases_dict", {})
    return g2p


def make_processor(case: dict[str, Any]):
    processor = ProcessorFactory.create_processor(case["language"])
    if isinstance(processor, ChineseProcessor) and "dicts" in case:
        processor.g2p = fake_zh_g2p(case["dicts"])
    return processor


def encode_processor_flow(case: dict[str, Any]) -> dict[str, Any]:
    processor = make_processor(case)
    cleaned = processor.clean_text(case["text"])
    text_list = processor.split_text(cleaned)
    phonetic_list = processor.get_phonetic_list(text_list)
    return {
        "processor_type": type(processor).__name__,
        "language_code": processor.language_code,
        "cleaned": cleaned,
        "text_list": text_list,
        "phonetic_list": phonetic_list,
    }


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    kind = case["kind"]
    if kind == "factory":
        created = []
        for code in case["codes"]:
            processor = ProcessorFactory.create_processor(code)
            created.append(
                {
                    "code": code,
                    "type": type(processor).__name__,
                    "language_code": processor.language_code,
                }
            )
        errors = []
        for code in case.get("unsupported", []):
            try:
                ProcessorFactory.create_processor(code)
            except Exception as error:  # noqa: BLE001 - exact legacy exception surface is fixture data.
                errors.append(
                    {
                        "code": code,
                        "type": type(error).__name__,
                        "message": str(error),
                    }
                )
            else:
                raise AssertionError(f"expected unsupported language error for {code!r}")
        return {
            "supported": ProcessorFactory.get_supported_languages(),
            "created": created,
            "errors": errors,
        }

    if kind == "processor_flow":
        return encode_processor_flow(case)

    if kind == "lyric_data_shape":
        processor = make_processor(case)
        cleaned = processor.clean_text(case["text"])
        text_list = processor.split_text(cleaned)
        phonetic_list = processor.get_phonetic_list(text_list)
        return {
            "text_list": text_list,
            "phonetic_list": phonetic_list,
            "raw_text": cleaned,
        }

    if kind == "reference_lyric_data_shape":
        processor = make_processor(case)
        cleaned = processor.clean_text(case["text"])
        build_reference_lyric = getattr(processor, "build_reference_lyric", None)
        if callable(build_reference_lyric):
            text_list, phonetic_list = build_reference_lyric(cleaned)
        else:
            text_list = processor.split_text(cleaned)
            phonetic_list = processor.get_phonetic_list(text_list)
        return {
            "text_list": text_list,
            "phonetic_list": phonetic_list,
            "raw_text": cleaned,
        }

    if kind == "japanese_reference_flow":
        processor = JapaneseProcessor()
        cleaned = processor.clean_text(case["text"])
        text_list = processor.split_text(cleaned)
        phonetic_list = processor.get_phonetic_list(text_list)
        reference_text, reference_phonetic = processor.build_reference_lyric(cleaned)
        return {
            "cleaned": cleaned,
            "text_list": text_list,
            "phonetic_list": phonetic_list,
            "reference_text": reference_text,
            "reference_phonetic": reference_phonetic,
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
