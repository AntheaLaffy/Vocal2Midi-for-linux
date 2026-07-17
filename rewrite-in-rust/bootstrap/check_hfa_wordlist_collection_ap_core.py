"""Check HubertFA WordList collection/AP fixtures against legacy Python."""

from __future__ import annotations

import hashlib
import json
import math
import pathlib
import sys
import unicodedata
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "hfa_wordlist_collection_ap_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.HubertFA.tools.align_word import Phoneme, Word, WordList  # noqa: E402


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
        "phonemes": [encode_phoneme(phoneme) for phoneme in word.phonemes],
    }


def make_word(data: dict[str, Any]) -> Word:
    word = Word(
        decode_float(data["start"]),
        decode_float(data["end"]),
        data["text"],
        data.get("init_phoneme", False),
    )
    if "phonemes" in data:
        word.phonemes = [
            Phoneme(decode_float(item["start"]), decode_float(item["end"]), item["text"])
            for item in data["phonemes"]
        ]
    if "start_override" in data:
        word.start = decode_float(data["start_override"])
    return word


def make_entry(data: dict[str, Any]) -> Any:
    if data.get("kind") == "invalid":
        return data["value"]
    return make_word(data)


def encode_entry(entry: Any) -> dict[str, Any]:
    if isinstance(entry, Word):
        return {"kind": "word", "word": encode_word(entry)}
    return {"kind": "invalid", "value": entry}


def encode_entries(words: WordList) -> list[dict[str, Any]]:
    return [encode_entry(entry) for entry in words]


def make_word_list(seed: list[dict[str, Any]]) -> WordList:
    return WordList([make_entry(entry) for entry in seed])


def encode_intervals(intervals: list[list[float]]) -> list[list[Any]]:
    return [[encode_float(value) for value in interval] for interval in intervals]


def encode_error(callable_) -> dict[str, Any]:
    try:
        value = callable_()
    except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
        return {"error": {"type": type(error).__name__, "message": str(error)}}
    return {"ok": value}


def run_case(case: dict[str, Any]) -> Any:
    kind = case["kind"]
    if kind == "unicode_printability_digest":
        digest = hashlib.md5(usedforsecurity=False)
        scalar_count = 0
        nonprintable_count = 0
        for codepoint in range(0x110000):
            if 0xD800 <= codepoint <= 0xDFFF:
                continue
            nonprintable = not chr(codepoint).isprintable()
            digest.update(bytes((int(nonprintable),)))
            scalar_count += 1
            nonprintable_count += int(nonprintable)
        return {
            "unidata_version": unicodedata.unidata_version,
            "scalar_count": scalar_count,
            "nonprintable_count": nonprintable_count,
            "md5": digest.hexdigest(),
        }

    if kind == "sort_key_corpus":
        items = [
            {"label": item["label"], "key": decode_float(item["key"])}
            for item in case["items"]
        ]
        items.sort(key=lambda item: item["key"])
        return {"order": [item["label"] for item in items]}

    if kind == "append_sequence":
        words = WordList()
        for operation in case["operations"]:
            words.append(make_word(operation))
        return {
            "entries": encode_entries(words),
            "phonemes": words.phonemes,
            "intervals": encode_intervals(words.intervals),
            "log": words.log(),
        }

    if kind == "raw_seed_extend":
        words = make_word_list(case["seed"])
        words.extend(make_entry(entry) for entry in case["extend"])
        return {"entries": encode_entries(words), "log": words.log()}

    if kind == "overlap_scan":
        words = make_word_list(case["seed"])
        overlaps = [
            [word.text for word in words.overlapping_words(make_word(query))]
            for query in case["queries"]
        ]
        return {"seed_entries": encode_entries(words), "overlaps": overlaps}

    if kind == "log_lifecycle":
        words = WordList([make_word(item) for item in case["seed"]])
        for item in case["before_clear"]:
            words.append(make_word(item))
        before = words.log()
        words.clear_log()
        cleared = words.log()
        for item in case["after_clear"]:
            words.append(make_word(item))
        return {"before": before, "cleared": cleared, "after": words.log()}

    if kind == "remove_intervals":
        results = []
        for item in case["items"]:
            try:
                intervals = WordList.remove_overlapping_intervals(
                    tuple(decode_float(value) for value in item["raw"]),
                    tuple(decode_float(value) for value in item["remove"]),
                )
            except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
                results.append({"error": {"type": type(error).__name__, "message": str(error)}})
            else:
                results.append({"ok": encode_intervals([list(interval) for interval in intervals])})
        return results

    if kind == "add_ap":
        words = make_word_list(case["seed"])
        for call in case["calls"]:
            if "min_dur" in call:
                words.add_AP(make_word(call["word"]), min_dur=decode_float(call["min_dur"]))
            else:
                words.add_AP(make_word(call["word"]))
        return {"entries": encode_entries(words), "log": words.log()}

    if kind == "sort_add_ap":
        words = make_word_list(case["seed"])
        for call in case["calls"]:
            words.add_AP(make_word(call), min_dur=0.1)
        return {
            "order": [
                {"text": word.text, "start": encode_float(word.start)} for word in words
            ],
            "log": words.log(),
        }

    if kind == "add_ap_alias":
        words = make_word_list(case["seed"])
        new_word = make_word(case["word"])
        words.add_AP(new_word, min_dur=decode_float(case["min_dur"]))
        stored_is_original = any(entry is new_word for entry in words)
        new_word.text = case["mutate_text"]
        for phoneme in new_word.phonemes:
            phoneme.text = case["mutate_phoneme_text"]
        return {
            "stored_is_original": stored_is_original,
            "entries_after_source_mutation": encode_entries(words),
            "log": words.log(),
        }

    if kind == "projection_errors":
        def projection_result(operation: str) -> dict[str, Any]:
            words = make_word_list(case["seed"])
            if operation == "phonemes":
                return encode_error(lambda: words.phonemes)
            if operation == "intervals":
                return encode_error(lambda: encode_intervals(words.intervals))
            return encode_error(lambda: words.clear_language_prefix())

        return {operation: projection_result(operation) for operation in case["operations"]}

    if kind == "prefix_partial_error":
        words = make_word_list(case["seed"])
        result = encode_error(lambda: words.clear_language_prefix())
        return {"result": result, "entries": encode_entries(words)}

    if kind == "projections_and_prefix":
        words = make_word_list(case["seed"])
        before_phonemes = words.phonemes
        intervals = encode_intervals(words.intervals)
        words.clear_language_prefix()
        return {
            "before_phonemes": before_phonemes,
            "intervals": intervals,
            "after_phonemes": words.phonemes,
            "entries": encode_entries(words),
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
