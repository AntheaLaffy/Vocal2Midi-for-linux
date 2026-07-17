"""Legacy fixture checker for the HubertFA WordList finalization seam.

The JSONL table is intentionally executable by the legacy Python implementation
before a Rust finalizer exists.  ``--generate`` prints the same records with
their Python 3.12.13 results filled in, which keeps expected values produced by
the pinned compatibility runtime rather than by hand-written Rust code.
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).parents[2]
sys.path.insert(0, str(ROOT))

from inference.HubertFA.tools.align_word import Phoneme, Word, WordList


FIXTURE_PATH = Path(__file__).parents[1] / "fixtures" / "hfa_wordlist_finalize_core.jsonl"


def decode_float(value: Any) -> Any:
    if not isinstance(value, dict) or "$float" not in value:
        return value
    marker = value["$float"]
    if marker == "nan":
        return float("nan")
    if marker == "+inf":
        return float("inf")
    if marker == "-inf":
        return float("-inf")
    if marker == "-0.0":
        return -0.0
    raise AssertionError(f"unknown special float marker {marker!r}")


def encode_float(value: float) -> Any:
    if math.isnan(value):
        return {"$float": "nan"}
    if math.isinf(value):
        return {"$float": "+inf" if value > 0 else "-inf"}
    if value == 0.0 and math.copysign(1.0, value) < 0:
        return {"$float": "-0.0"}
    # Keep the fixture numeric domain explicitly f64 even when legacy code
    # assigns the integer literal 0 during negative-start repair.
    return float(value)


def make_word(item: dict[str, Any]) -> Word:
    word = Word(
        decode_float(item["start"]),
        decode_float(item["end"]),
        item["text"],
        item.get("init_phoneme", False),
    )
    if "start_override" in item:
        word.start = decode_float(item["start_override"])
    if "end_override" in item:
        word.end = decode_float(item["end_override"])
    if item.get("empty", False):
        word.phonemes = []
    if "phoneme_specs" in item:
        word.phonemes = []
        for spec in item["phoneme_specs"]:
            start = decode_float(spec["start"])
            end = decode_float(spec["end"])
            try:
                phoneme = Phoneme(start, end, spec["text"])
            except ValueError:
                # Raw legacy callers can mutate a valid Phoneme into an
                # invalid check fixture without adding a public constructor.
                phoneme = Phoneme(0.0, 1.0, spec["text"])
                phoneme.start = start
                phoneme.end = end
            word.phonemes.append(phoneme)
    for override in item.get("phoneme_overrides", []):
        index = override["index"]
        phoneme = word.phonemes[index]
        if "start" in override:
            phoneme.start = decode_float(override["start"])
        if "end" in override:
            phoneme.end = decode_float(override["end"])
        if "text" in override:
            phoneme.text = override["text"]
    if "fixture_id" in item:
        word._fixture_id = item["fixture_id"]
    return word


def make_entry(item: dict[str, Any]) -> Any:
    if item.get("kind", "word") == "invalid":
        return item.get("value", "invalid")
    return make_word(item)


def make_word_list(items: list[dict[str, Any]]) -> WordList:
    aliases: dict[str, Word] = {}
    entries = []
    for item in items:
        if "reuse_id" in item:
            entries.append(aliases[item["reuse_id"]])
            continue
        entry = make_entry(item)
        entries.append(entry)
        if isinstance(entry, Word) and hasattr(entry, "_fixture_id"):
            aliases[entry._fixture_id] = entry
    return WordList(entries)


def word_identity(word: Word, state: dict[str, Any]) -> str:
    fixture_id = getattr(word, "_fixture_id", None)
    if fixture_id is not None:
        return fixture_id
    key = id(word)
    if key not in state["new_ids"]:
        state["new_ids"][key] = f"new{len(state['new_ids'])}"
    return state["new_ids"][key]


def encode_word(word: Word, state: dict[str, Any]) -> dict[str, Any]:
    return {
        "identity": word_identity(word, state),
        "start": encode_float(word.start),
        "end": encode_float(word.end),
        "text": word.text,
        "dur": encode_float(word.dur),
        "phonemes": [
            {
                "start": encode_float(phoneme.start),
                "end": encode_float(phoneme.end),
                "text": phoneme.text,
            }
            for phoneme in word.phonemes
        ],
    }


def encode_entries(words: WordList, state: dict[str, Any]) -> list[dict[str, Any]]:
    encoded = []
    for entry in words:
        if isinstance(entry, Word):
            encoded.append({"kind": "word", "value": encode_word(entry, state)})
        else:
            encoded.append({"kind": "invalid", "value": entry})
    return encoded


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    words = make_word_list(case.get("seed", []))
    if "pre_log" in case:
        words._log = list(case["pre_log"])
    state = {"new_ids": {}}
    kind = case["kind"]

    if kind == "fill_small_gaps":
        calls = case.get("calls") or [{"wav_length": case["wav_length"]}]
        for call in calls:
            wav_length = decode_float(call["wav_length"])
            gap_length = decode_float(call.get("gap_length", 0.1))
            words.fill_small_gaps(wav_length, gap_length)
        return {
            "returns": [None] * len(calls),
            "entries": encode_entries(words, state),
            "log": words.log(),
        }

    if kind == "add_SP":
        returns = []
        calls = case.get("calls") or [{"wav_length": case["wav_length"]}]
        for call in calls:
            returns.append(
                words.add_SP(
                    decode_float(call["wav_length"]),
                    call.get("add_phone", "SP"),
                )
            )
        return {"returns": returns, "entries": encode_entries(words, state), "log": words.log()}

    if kind == "clear_extend_check":
        actions = []
        for action in case["actions"]:
            operation = action["kind"]
            if operation == "add_SP":
                actions.append(
                    words.add_SP(
                        decode_float(action["wav_length"]),
                        action.get("add_phone", "SP"),
                    )
                )
            elif operation == "clear":
                words.clear()
                actions.append(None)
            elif operation == "extend":
                words.extend(make_entry(item) for item in action["entries"])
                actions.append(None)
            elif operation == "check":
                actions.append(words.check())
            else:
                raise AssertionError(f"unknown clear/extend action {operation!r}")
        return {"returns": actions, "entries": encode_entries(words, state), "log": words.log()}

    if kind == "check":
        returns = [words.check() for _ in range(case.get("repeat", 1))]
        return {"returns": returns, "entries": encode_entries(words, state), "log": words.log()}

    raise AssertionError(f"unknown fixture kind {kind!r}")


def assert_json_close(actual: Any, expected: Any, context: str) -> None:
    if isinstance(actual, float) or isinstance(expected, float):
        actual_float = float(actual)
        expected_float = float(expected)
        if math.isnan(actual_float) and math.isnan(expected_float):
            return
        assert actual_float == expected_float, f"{context}: {actual!r} != {expected!r}"
        return
    if isinstance(actual, list) and isinstance(expected, list):
        assert len(actual) == len(expected), f"{context}: array lengths differ"
        for index, (left, right) in enumerate(zip(actual, expected)):
            assert_json_close(left, right, f"{context}[{index}]")
        return
    if isinstance(actual, dict) and isinstance(expected, dict):
        assert set(actual) == set(expected), f"{context}: keys differ: {actual.keys()} != {expected.keys()}"
        for key in expected:
            assert_json_close(actual[key], expected[key], f"{context}.{key}")
        return
    assert actual == expected, f"{context}: {actual!r} != {expected!r}"


def read_cases() -> list[dict[str, Any]]:
    return [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf8").splitlines()
        if line and not line.startswith("#")
    ]


def main() -> None:
    cases = read_cases()
    if "--generate" in sys.argv:
        for case in cases:
            output = dict(case)
            output["expect"] = run_case(case)
            print(json.dumps(output, ensure_ascii=False, separators=(",", ":"), allow_nan=False))
        return

    for line_number, case in enumerate(cases, start=1):
        actual = run_case(case)
        assert_json_close(actual, case["expect"], f"{case['case_id']} line {line_number}")


if __name__ == "__main__":
    main()
