"""Check Bayesian quantization fixtures against legacy Python."""

from __future__ import annotations

from dataclasses import dataclass
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
HELPER_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_bayesian_helpers.tsv"
CORE_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_bayesian_core.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import (  # noqa: E402
    _bayes_local_cost,
    _build_bayes_candidate_pairs,
    _build_piece_specific_priors,
    _decode_segment_bayesian,
    _estimate_segment_phase_center,
    _filter_bayes_candidate_pairs,
    _metrical_position_penalty,
    _note_value_penalty,
    _preferred_sv_duration,
    _quantize_notes_bayesian,
    _resolve_bayes_shift_limit,
    _segment_split_indices_bayesian,
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


def parse_optional_float(value: str) -> float | None:
    if value == "__none__" or value == "":
        return None
    return float(value)


def parse_pair(value: str) -> dict[str, Any]:
    parts = value.split(",")
    if len(parts) not in {4, 5}:
        raise AssertionError(f"invalid pair encoding {value!r}")

    pair: dict[str, Any] = {
        "raw_start": int(parts[0]),
        "raw_end": int(parts[1]),
        "raw_dur": int(parts[2]),
        "lyrics": parse_lyric(parts[3]),
    }
    if len(parts) == 5:
        pair["raw_gap"] = int(parts[4])
    return pair


def parse_pairs(value: str) -> list[dict[str, Any]]:
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


def parse_int_list(value: str) -> list[int]:
    if not value or value == "__empty__":
        return []
    return [int(item) for item in value.split(",")]


def parse_prior(value: str) -> dict[str, float | int]:
    count, strength, preferred_dur, preferred_gap, preferred_phase = value.split(",")
    prior: dict[str, float | int] = {
        "count": int(count),
        "strength": float(strength),
    }
    if preferred_dur != "__none__":
        prior["preferred_dur"] = int(preferred_dur)
    if preferred_gap != "__none__":
        prior["preferred_gap"] = int(preferred_gap)
    if preferred_phase != "__none__":
        prior["preferred_phase"] = int(preferred_phase)
    return prior


def parse_priors(value: str) -> list[dict[str, float | int]]:
    if not value or value == "__empty__":
        return []
    return [parse_prior(item) for item in value.split("|")]


def encode_prior(prior: dict[str, float | int]) -> str:
    return ",".join(
        [
            str(int(prior.get("count", 1))),
            f"{float(prior.get('strength', 0.0)):.12f}",
            str(int(prior["preferred_dur"])) if "preferred_dur" in prior else "__none__",
            str(int(prior["preferred_gap"])) if "preferred_gap" in prior else "__none__",
            str(int(prior["preferred_phase"])) if "preferred_phase" in prior else "__none__",
        ]
    )


def encode_priors(priors: list[dict[str, float | int]]) -> str:
    if not priors:
        return "__empty__"
    return "|".join(encode_prior(prior) for prior in priors)


def parse_decode_expected(value: str) -> tuple[list[tuple[int, int]], float]:
    seq_raw, cost_raw = value.split(";", 1)
    return parse_tick_pairs(seq_raw), float(cost_raw)


def parse_local_options(value: str) -> tuple[int, int | None, int | None, float, float]:
    step, prev_end, prev_raw_end, segment_center, segment_weight = value.split(",", 4)
    return (
        int(step),
        parse_optional_int(prev_end),
        parse_optional_int(prev_raw_end),
        float(segment_center),
        float(segment_weight),
    )


def parse_notes(value: str) -> list[FixtureNote]:
    if not value or value == "__empty__":
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


def assert_priors_equal(
    case_id: str,
    actual: list[dict[str, float | int]],
    expected: list[dict[str, float | int]],
) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: prior length mismatch: {actual!r} != {expected!r}")
    for index, (actual_prior, expected_prior) in enumerate(zip(actual, expected)):
        if set(actual_prior) != set(expected_prior):
            raise AssertionError(
                f"{case_id}: prior keys mismatch at {index}: "
                f"{actual_prior!r} != {expected_prior!r}"
            )
        for key, expected_value in expected_prior.items():
            actual_value = actual_prior[key]
            if isinstance(expected_value, float):
                assert_float_close(case_id, float(actual_value), expected_value)
            elif actual_value != expected_value:
                raise AssertionError(
                    f"{case_id}: prior value mismatch at {index}.{key}: "
                    f"{actual_value!r} != {expected_value!r}"
                )


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

    if kind == "resolve_bayes_shift_limit":
        actual = _resolve_bayes_shift_limit(
            int(input_a),
            factor=float(input_b),
            floor=int(input_c),
            cap=int(input_d),
        )
        if actual != int(expected):
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
    elif kind == "filter_bayes_candidate_pairs":
        actual = _filter_bayes_candidate_pairs(
            parse_pair(input_b),
            parse_tick_pairs(input_c),
            int(input_a),
        )
        if encode_tick_pairs(actual) != expected:
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
    elif kind == "build_bayes_candidate_pairs":
        actual = _build_bayes_candidate_pairs(parse_pair(input_b), int(input_a))
        if encode_tick_pairs(actual) != expected:
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
    elif kind == "estimate_segment_phase_center":
        actual_center, actual_weight = _estimate_segment_phase_center(
            parse_pairs(input_b),
            int(input_a),
        )
        expected_center_raw, expected_weight_raw = expected.split(",", 1)
        assert_float_close(case_id, actual_center, float(expected_center_raw))
        assert_float_close(case_id, actual_weight, float(expected_weight_raw))
    elif kind == "segment_split_indices_bayesian":
        actual = _segment_split_indices_bayesian(parse_pairs(input_b), int(input_a))
        if encode_tick_pairs(actual) != expected:
            raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")
    elif kind == "metrical_position_penalty":
        actual = _metrical_position_penalty(int(input_a), int(input_b))
        assert_float_close(case_id, actual, float(expected))
    elif kind == "note_value_penalty":
        actual = _note_value_penalty(int(input_a), int(input_b))
        assert_float_close(case_id, actual, float(expected))
    elif kind == "preferred_sv_duration":
        actual = _preferred_sv_duration(int(input_a), int(input_b))
        expected_value = parse_optional_int(expected)
        if actual != expected_value:
            raise AssertionError(f"{case_id}: {actual!r} != {expected_value!r}")
    elif kind == "build_piece_specific_priors":
        actual = _build_piece_specific_priors(
            parse_pairs(input_b),
            int(input_a),
            parse_int_list(input_c),
            parse_int_list(input_d),
        )
        assert_priors_equal(case_id, actual, parse_priors(expected))
    elif kind == "bayes_local_cost":
        pair = parse_pair(input_c)
        prior = parse_prior(input_d)
        step, prev_end, prev_raw_end, segment_center, segment_weight = parse_local_options(input_e)
        actual = _bayes_local_cost(
            int(input_a),
            int(input_b),
            pair,
            prior,
            step,
            prev_end=prev_end,
            prev_raw_end=prev_raw_end,
            segment_center=segment_center,
            segment_center_weight=segment_weight,
        )
        assert_float_close(case_id, actual, float(expected))
    elif kind == "decode_segment_bayesian":
        actual_seq, actual_cost = _decode_segment_bayesian(
            parse_pairs(input_b),
            parse_priors(input_c),
            int(input_a),
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
    _quantize_notes_bayesian(notes, float(tempo_raw), int(step_raw))
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
