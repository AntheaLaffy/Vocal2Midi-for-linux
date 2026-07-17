"""Check HubertFA Phoneme/Word fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import pathlib
import sys
import warnings
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "hfa_word_model_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.HubertFA.tools.align_word import Phoneme, Word  # noqa: E402


def decode_float(value: Any) -> Any:
    if not isinstance(value, dict) or set(value) != {"$float"}:
        return value
    return {
        "nan": float("nan"),
        "+inf": float("inf"),
        "-inf": float("-inf"),
        "-0.0": -0.0,
    }[value["$float"]]


def encode_float(value: float) -> float | dict[str, str]:
    if math.isnan(value):
        return {"$float": "nan"}
    if math.isinf(value):
        return {"$float": "+inf" if value > 0 else "-inf"}
    if value == 0.0 and math.copysign(1.0, value) < 0:
        return {"$float": "-0.0"}
    return value


def assert_close(case_id: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict) or set(actual) != set(expected):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
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
            float(actual), expected, rel_tol=1e-12, abs_tol=1e-12
        ):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
        return

    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def encode_phoneme(phoneme: Phoneme) -> dict[str, Any]:
    return {
        "start": encode_float(phoneme.start),
        "end": encode_float(phoneme.end),
        "text": phoneme.text,
    }


def encode_word(word: Word) -> dict[str, Any]:
    return {
        "start": encode_float(word.start),
        "end": encode_float(word.end),
        "text": word.text,
        "dur": encode_float(word.dur),
        "phonemes": [encode_phoneme(phoneme) for phoneme in word.phonemes],
    }


def encode_constructor(callable_, *args) -> dict[str, Any]:
    try:
        value = callable_(*args)
    except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
        return {"error": {"type": type(error).__name__, "message": str(error)}}

    if isinstance(value, Phoneme):
        return {"ok": encode_phoneme(value)}
    return {"ok": encode_word(value)}


def make_phoneme(data: dict[str, Any]) -> Phoneme:
    phoneme = Phoneme(decode_float(data["start"]), decode_float(data["end"]), data["text"])
    if data.get("force_zero"):
        phoneme.end = phoneme.start
    return phoneme


def run_case(case: dict[str, Any]) -> Any:
    kind = case["kind"]
    if kind == "phoneme_constructor":
        return [
            encode_constructor(
                Phoneme,
                decode_float(item["start"]),
                decode_float(item["end"]),
                item["text"],
            )
            for item in case["items"]
        ]

    if kind == "word_constructor":
        return [
            encode_constructor(
                Word,
                decode_float(item["start"]),
                decode_float(item["end"]),
                item["text"],
                item["init_phoneme"],
            )
            for item in case["items"]
        ]

    word_data = case["word"]
    word = Word(
        decode_float(word_data["start"]),
        decode_float(word_data["end"]),
        word_data["text"],
        word_data.get("init_phoneme", False),
    )

    if kind in {"add_phoneme", "append_phoneme"}:
        logs: list[str] = []
        mutation = word.add_phoneme if kind == "add_phoneme" else word.append_phoneme
        for item in case["phonemes"]:
            mutation(make_phoneme(item), logs)
        return {"word": encode_word(word), "logs": logs}

    if kind == "move_boundaries":
        logs = []
        for move in case["moves"]:
            if move["kind"] == "start":
                word.move_start(decode_float(move["value"]), logs)
            else:
                word.move_end(decode_float(move["value"]), logs)
        return {"word": encode_word(word), "logs": logs}

    if kind == "move_boundary_errors":
        results = []
        for move in case["moves"]:
            error_word = Word(
                decode_float(word_data["start"]),
                decode_float(word_data["end"]),
                word_data["text"],
                word_data.get("init_phoneme", False),
            )
            try:
                if move["kind"] == "start":
                    error_word.move_start(decode_float(move["value"]))
                else:
                    error_word.move_end(decode_float(move["value"]))
            except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
                results.append({"error": {"type": type(error).__name__, "message": str(error)}})
            else:
                results.append({"ok": encode_word(error_word)})
        return results

    if kind == "warning_sink":
        with warnings.catch_warnings(record=True) as caught:
            warnings.simplefilter("always")
            for operation in case["operations"]:
                operation_kind = operation["kind"]
                if operation_kind == "add":
                    word.add_phoneme(make_phoneme(operation["phoneme"]))
                elif operation_kind == "append":
                    word.append_phoneme(make_phoneme(operation["phoneme"]))
                elif operation_kind == "move_start":
                    word.move_start(decode_float(operation["value"]))
                elif operation_kind == "move_end":
                    word.move_end(decode_float(operation["value"]))
                else:
                    raise AssertionError(f"unknown warning operation {operation_kind!r}")
        return {
            "warnings": [
                {"category": item.category.__name__, "message": str(item.message)} for item in caught
            ],
            "word": encode_word(word),
        }

    raise AssertionError(f"unknown fixture kind {kind!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        case = json.loads(line)
        assert_close(f"{case['case_id']} line {line_number}", run_case(case), case["expect"])


if __name__ == "__main__":
    main()
