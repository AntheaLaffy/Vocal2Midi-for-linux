"""Check quantization candidate primitive fixtures against legacy Python."""

from __future__ import annotations

from dataclasses import dataclass
import pathlib
import sys
from types import SimpleNamespace

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
SCALAR_FIXTURE_PATH = (
    REWRITE_ROOT / "fixtures" / "quantization_candidate_scalar_primitives.tsv"
)
PAIR_FIXTURE_PATH = (
    REWRITE_ROOT / "fixtures" / "quantization_candidate_pair_primitives.tsv"
)

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import (  # noqa: E402
    _annotate_pairs_with_gap,
    _build_candidate_pairs,
    _build_duration_candidates,
    _build_gap_candidates,
    _build_note_pair,
    _candidate_values,
    _dist_grid,
    _mod_distance,
    _nearest_candidate,
    _resolve_dp_grid_step,
    _resolve_segment_shift_candidates,
)


@dataclass(frozen=True)
class RawPair:
    raw_start: int
    raw_end: int
    raw_dur: int
    lyrics: str

    def as_dict(self) -> dict[str, int | str]:
        return {
            "raw_start": self.raw_start,
            "raw_end": self.raw_end,
            "raw_dur": self.raw_dur,
            "lyrics": self.lyrics,
        }


@dataclass(frozen=True)
class AnnotatedPair:
    raw_start: int
    raw_end: int
    raw_dur: int
    lyrics: str
    raw_gap: int


def encode_list(values: list[int] | tuple[int, ...]) -> str:
    return ",".join(str(value) for value in values)


def encode_candidate_pairs(values: list[tuple[int, int]]) -> str:
    return "|".join(f"{start},{end}" for start, end in values)


def parse_int_list(value: str) -> list[int]:
    if not value:
        return []
    return [int(item) for item in value.split(",")]


def parse_lyric(value: str) -> str | None:
    if value == "__missing__":
        return None
    if value == "__empty__":
        return ""
    return value


def encode_lyric(value: object) -> str:
    return str(value) if value else "__empty__"


def parse_raw_pair(value: str) -> RawPair:
    raw_start, raw_end, raw_dur, lyric = value.split(",", 3)
    parsed_lyric = parse_lyric(lyric)
    return RawPair(int(raw_start), int(raw_end), int(raw_dur), parsed_lyric or "")


def parse_raw_pairs(value: str) -> list[RawPair]:
    if not value:
        return []
    return [parse_raw_pair(item) for item in value.split("|")]


def parse_annotated_pair(value: str) -> AnnotatedPair:
    raw_start, raw_end, raw_dur, lyric, raw_gap = value.split(",", 4)
    parsed_lyric = parse_lyric(lyric)
    return AnnotatedPair(
        int(raw_start),
        int(raw_end),
        int(raw_dur),
        parsed_lyric or "",
        int(raw_gap),
    )


def parse_annotated_pairs(value: str) -> list[AnnotatedPair]:
    if not value:
        return []
    return [parse_annotated_pair(item) for item in value.split("|")]


def normalize_raw_pair(pair: dict[str, object]) -> RawPair:
    return RawPair(
        int(pair["raw_start"]),
        int(pair["raw_end"]),
        int(pair["raw_dur"]),
        str(pair["lyrics"]),
    )


def normalize_annotated_pair(pair: dict[str, object]) -> AnnotatedPair:
    return AnnotatedPair(
        int(pair["raw_start"]),
        int(pair["raw_end"]),
        int(pair["raw_dur"]),
        str(pair["lyrics"]),
        int(pair["raw_gap"]),
    )


def check_scalar_fixture(line_number: int, fields: list[str]) -> None:
    case_id, kind, input_a, input_b, input_c, _input_d, expected = fields

    if kind == "resolve_dp_grid_step":
        actual = str(_resolve_dp_grid_step(int(input_a)))
    elif kind == "resolve_segment_shift_candidates":
        actual = encode_list(_resolve_segment_shift_candidates(int(input_a)))
    elif kind == "nearest_candidate":
        actual = str(_nearest_candidate(float(input_a), parse_int_list(input_b)))
    elif kind == "mod_distance":
        actual = str(_mod_distance(int(input_a), int(input_b), int(input_c)))
    elif kind == "dist_grid":
        actual = str(_dist_grid(int(input_a), int(input_b)))
    elif kind == "candidate_values":
        actual = encode_list(_candidate_values(int(input_a), int(input_b), int(input_c)))
    elif kind == "duration_candidates":
        actual = encode_list(_build_duration_candidates(int(input_a), int(input_b)))
    elif kind == "gap_candidates":
        actual = encode_list(_build_gap_candidates(int(input_a), int(input_b)))
    else:
        raise AssertionError(f"line {line_number}: unknown scalar kind {kind!r}")

    if actual != expected:
        raise AssertionError(
            f"{case_id}: scalar mismatch on line {line_number}: "
            f"{actual!r} != {expected!r}"
        )


def make_note(lyric_raw: str) -> object:
    lyric = parse_lyric(lyric_raw)
    if lyric is None:
        return SimpleNamespace()
    return SimpleNamespace(lyric=lyric)


def check_pair_fixture(line_number: int, fields: list[str]) -> None:
    case_id, kind, raw_start, raw_end, lyric, radius, step, input_pairs, expected = fields

    if kind == "build_note_pair":
        actual = normalize_raw_pair(
            _build_note_pair(make_note(lyric), int(raw_start), int(raw_end))
        )
        parsed_expected = parse_raw_pair(expected)
    elif kind == "annotate_pairs_with_gap":
        pairs = [pair.as_dict() for pair in parse_raw_pairs(input_pairs)]
        actual = [
            normalize_annotated_pair(pair) for pair in _annotate_pairs_with_gap(pairs)
        ]
        parsed_expected = parse_annotated_pairs(expected)
    elif kind == "build_candidate_pairs":
        pair = _build_note_pair(make_note(lyric), int(raw_start), int(raw_end))
        actual = encode_candidate_pairs(
            _build_candidate_pairs(pair, int(radius), int(step))
        )
        parsed_expected = expected
    else:
        raise AssertionError(f"line {line_number}: unknown pair kind {kind!r}")

    if actual != parsed_expected:
        raise AssertionError(
            f"{case_id}: pair mismatch on line {line_number}: "
            f"{actual!r} != {parsed_expected!r}"
        )


def main() -> None:
    for line_number, line in enumerate(
        SCALAR_FIXTURE_PATH.read_text().splitlines(), start=1
    ):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 7:
            raise AssertionError(f"line {line_number}: expected 7 scalar columns")
        check_scalar_fixture(line_number, fields)

    for line_number, line in enumerate(
        PAIR_FIXTURE_PATH.read_text().splitlines(), start=1
    ):
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) != 9:
            raise AssertionError(f"line {line_number}: expected 9 pair columns")
        check_pair_fixture(line_number, fields)


if __name__ == "__main__":
    main()
