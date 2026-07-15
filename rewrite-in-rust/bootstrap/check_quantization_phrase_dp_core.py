"""Check phrase-DP quantization fixtures against legacy Python."""

from __future__ import annotations

from dataclasses import dataclass
import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
HELPER_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_phrase_dp_helpers.tsv"
CORE_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_phrase_dp_core.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import (  # noqa: E402
    _center_adjustment,
    _decode_segment_with_center,
    _local_cost_asym,
    _quantize_notes_phrase_hybrid,
    _segment_split_indices,
)

FLOAT_TOL = 1e-9
NOTE_FLOAT_TOL = 1e-12


@dataclass
class FixtureNote:
    onset: float
    offset: float
    pitch: float
    lyric: str


def parse_lyric(value: str) -> str:
    if value == "__empty__":
        return ""
    return value


def parse_optional_int(value: str) -> int | None:
    if value == "__none__" or value == "":
        return None
    return int(value)


def parse_pair(value: str) -> dict[str, int | str]:
    raw_start, raw_end, raw_dur, lyric = value.split(",", 3)
    return {
        "raw_start": int(raw_start),
        "raw_end": int(raw_end),
        "raw_dur": int(raw_dur),
        "lyrics": parse_lyric(lyric),
    }


def parse_pairs(value: str) -> list[dict[str, int | str]]:
    if not value or value == "__empty__":
        return []
    return [parse_pair(item) for item in value.split("|")]


def parse_tick_pairs(value: str) -> list[tuple[int, int]]:
    if not value or value == "__empty__":
        return []
    pairs = []
    for item in value.split("|"):
        left, right = item.split(",", 1)
        pairs.append((int(left), int(right)))
    return pairs


def encode_tick_pairs(value: list[tuple[int, int]]) -> str:
    if not value:
        return "__empty__"
    return "|".join(f"{left},{right}" for left, right in value)


def parse_decode_expected(value: str) -> tuple[list[tuple[int, int]], float]:
    seq_raw, cost_raw = value.split(";", 1)
    return parse_tick_pairs(seq_raw), float(cost_raw)


def parse_notes(value: str) -> list[FixtureNote]:
    if not value:
        return []

    notes = []
    for raw_note in value.split("|"):
        onset_raw, offset_raw, pitch_raw, lyric_raw = raw_note.split(",", 3)
        notes.append(
            FixtureNote(
                onset=float(onset_raw),
                offset=float(offset_raw),
                pitch=float(pitch_raw),
                lyric=parse_lyric(lyric_raw),
            )
        )
    return notes


def assert_float_close(case_id: str, actual: float, expected: float) -> None:
    if abs(actual - expected) > FLOAT_TOL:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def assert_notes_close(
    case_id: str,
    actual: list[FixtureNote],
    expected: list[FixtureNote],
) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: length mismatch: {actual!r} != {expected!r}")

    for index, (actual_note, expected_note) in enumerate(zip(actual, expected)):
        if abs(actual_note.onset - expected_note.onset) > NOTE_FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: onset mismatch at {index}: "
                f"{actual_note.onset!r} != {expected_note.onset!r}"
            )
        if abs(actual_note.offset - expected_note.offset) > NOTE_FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: offset mismatch at {index}: "
                f"{actual_note.offset!r} != {expected_note.offset!r}"
            )
        if abs(actual_note.pitch - expected_note.pitch) > NOTE_FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: pitch mismatch at {index}: "
                f"{actual_note.pitch!r} != {expected_note.pitch!r}"
            )
        if actual_note.lyric != expected_note.lyric:
            raise AssertionError(
                f"{case_id}: lyric mismatch at {index}: "
                f"{actual_note.lyric!r} != {expected_note.lyric!r}"
            )


def check_helper_fixture(line_number: int, fields: list[str]) -> None:
    case_id, kind, input_a, input_b, input_c, input_d, input_e, expected = fields

    if kind == "local_cost_asym":
        actual = _local_cost_asym(
            int(input_a),
            int(input_b),
            parse_pair(input_c),
            parse_optional_int(input_d),
            parse_optional_int(input_e),
        )
        assert_float_close(case_id, actual, float(expected))
    elif kind == "segment_split_indices":
        actual = _segment_split_indices(parse_pairs(input_b), int(input_a))
        if encode_tick_pairs(actual) != expected:
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
    elif kind == "center_adjustment":
        actual = _center_adjustment(
            parse_pair(input_b),
            int(input_a),
            parse_optional_int(input_c),
            int(input_d),
        )
        assert_float_close(case_id, actual, float(expected))
    elif kind == "decode_segment_with_center":
        actual_seq, actual_cost = _decode_segment_with_center(
            parse_pairs(input_c),
            center=int(input_b),
            grid_step=int(input_a),
        )
        expected_seq, expected_cost = parse_decode_expected(expected)
        if actual_seq != expected_seq:
            raise AssertionError(f"{case_id}: {actual_seq!r} != {expected_seq!r}")
        assert_float_close(case_id, actual_cost, expected_cost)
    else:
        raise AssertionError(f"line {line_number}: unknown helper kind {kind!r}")


def check_core_fixture(line_number: int, fields: list[str]) -> None:
    case_id, tempo_raw, step_raw, input_raw, expected_raw = fields
    notes = parse_notes(input_raw)
    expected = parse_notes(expected_raw)
    _quantize_notes_phrase_hybrid(notes, float(tempo_raw), int(step_raw))
    try:
        assert_notes_close(case_id, notes, expected)
    except AssertionError as exc:
        raise AssertionError(f"fixture line {line_number}: {exc}") from exc


def main() -> None:
    for line_number, line in enumerate(HELPER_FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 8:
            raise AssertionError(f"line {line_number}: expected 8 helper columns")
        check_helper_fixture(line_number, fields)

    for line_number, line in enumerate(CORE_FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 5:
            raise AssertionError(f"line {line_number}: expected 5 core columns")
        check_core_fixture(line_number, fields)


if __name__ == "__main__":
    main()
