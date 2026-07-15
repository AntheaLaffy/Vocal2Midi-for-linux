"""Check GAME phone/word parsing fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_DIR = REWRITE_ROOT / "fixtures"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.game.alignment_utils import (  # noqa: E402
    merge_consecutive_uv_words,
    parse_words,
    validate_phones,
)

FLOAT_TOL = 1e-12


def parse_str_list(value: str) -> list[str]:
    return [] if value == "" else value.split(",")


def parse_float_list(value: str) -> list[float]:
    return [] if value == "" else [float(item) for item in value.split(",")]


def parse_int_list(value: str) -> list[int]:
    return [] if value == "" else [int(item) for item in value.split(",")]


def parse_bool(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise AssertionError(f"unknown bool {value!r}")


def parse_uv_vocab(value: str) -> set[str] | None:
    if value == "__none__":
        return None
    return set(parse_str_list(value))


def assert_float_lists_close(line_number: int, actual: list[float], expected: list[float]) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"line {line_number} length mismatch: {actual!r} != {expected!r}")
    for index, (actual_value, expected_value) in enumerate(zip(actual, expected)):
        if abs(actual_value - expected_value) > FLOAT_TOL:
            raise AssertionError(
                f"line {line_number} float mismatch at {index}: "
                f"{actual_value!r} != {expected_value!r}"
            )


def check_validation() -> None:
    path = FIXTURE_DIR / "game_phone_word_validation.tsv"
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        ph_seq_raw, ph_dur_raw, ph_num_raw, valid_raw, expected_message = line.split("\t")
        actual_valid, actual_message = validate_phones(
            parse_str_list(ph_seq_raw),
            parse_float_list(ph_dur_raw),
            parse_int_list(ph_num_raw),
        )
        expected_valid = parse_bool(valid_raw)
        expected_message_value = expected_message or None
        if actual_valid != expected_valid or actual_message != expected_message_value:
            raise AssertionError(
                f"validation line {line_number} mismatch: "
                f"{(actual_valid, actual_message)!r} != {(expected_valid, expected_message_value)!r}"
            )


def check_parse_words() -> None:
    path = FIXTURE_DIR / "game_parse_words.tsv"
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        (
            ph_seq_raw,
            ph_dur_raw,
            ph_num_raw,
            uv_vocab_raw,
            uv_cond,
            merge_raw,
            expected_dur_raw,
            expected_vuv_raw,
        ) = line.split("\t")
        actual_dur, actual_vuv = parse_words(
            parse_str_list(ph_seq_raw),
            parse_float_list(ph_dur_raw),
            parse_int_list(ph_num_raw),
            uv_vocab=parse_uv_vocab(uv_vocab_raw),
            uv_cond=uv_cond,
            merge_consecutive_uv=parse_bool(merge_raw),
        )
        assert_float_lists_close(line_number, actual_dur, parse_float_list(expected_dur_raw))
        expected_vuv = parse_int_list(expected_vuv_raw)
        if actual_vuv != expected_vuv:
            raise AssertionError(f"parse line {line_number} vuv mismatch: {actual_vuv!r} != {expected_vuv!r}")


def check_merge_uv() -> None:
    path = FIXTURE_DIR / "game_merge_uv.tsv"
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        word_dur_raw, word_vuv_raw, expected_dur_raw, expected_vuv_raw = line.split("\t")
        actual_dur, actual_vuv = merge_consecutive_uv_words(
            parse_float_list(word_dur_raw),
            parse_int_list(word_vuv_raw),
        )
        assert_float_lists_close(line_number, actual_dur, parse_float_list(expected_dur_raw))
        expected_vuv = parse_int_list(expected_vuv_raw)
        if actual_vuv != expected_vuv:
            raise AssertionError(f"merge line {line_number} vuv mismatch: {actual_vuv!r} != {expected_vuv!r}")


def main() -> None:
    check_validation()
    check_parse_words()
    check_merge_uv()


if __name__ == "__main__":
    main()
