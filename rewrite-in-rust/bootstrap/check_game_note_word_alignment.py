"""Check GAME note/word alignment fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "game_note_word_alignment.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.game.alignment_utils import align_notes_to_words  # noqa: E402

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


def assert_float_lists_close(case_id: str, actual: list[float], expected: list[float]) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: length mismatch: {actual!r} != {expected!r}")
    for index, (actual_value, expected_value) in enumerate(zip(actual, expected)):
        if abs(actual_value - expected_value) > FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: float mismatch at {index}: "
                f"{actual_value!r} != {expected_value!r}"
            )


def main() -> None:
    for line in FIXTURE_PATH.read_text().splitlines():
        if not line or line.startswith("#"):
            continue

        (
            case_id,
            word_dur_raw,
            word_vuv_raw,
            note_seq_raw,
            note_dur_raw,
            tol_raw,
            apply_word_uv_raw,
            expected_seq_raw,
            expected_dur_raw,
            expected_slur_raw,
        ) = line.split("\t")
        actual_seq, actual_dur, actual_slur = align_notes_to_words(
            parse_float_list(word_dur_raw),
            parse_int_list(word_vuv_raw),
            parse_str_list(note_seq_raw),
            parse_float_list(note_dur_raw),
            tol=float(tol_raw),
            apply_word_uv=parse_bool(apply_word_uv_raw),
        )
        expected_seq = parse_str_list(expected_seq_raw)
        expected_dur = parse_float_list(expected_dur_raw)
        expected_slur = parse_int_list(expected_slur_raw)

        if actual_seq != expected_seq:
            raise AssertionError(f"{case_id}: seq mismatch: {actual_seq!r} != {expected_seq!r}")
        assert_float_lists_close(case_id, actual_dur, expected_dur)
        if actual_slur != expected_slur:
            raise AssertionError(
                f"{case_id}: slur mismatch: {actual_slur!r} != {expected_slur!r}"
            )


if __name__ == "__main__":
    main()
