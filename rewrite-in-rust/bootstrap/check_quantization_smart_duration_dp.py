"""Check smart-duration DP quantization fixtures against legacy Python."""

from __future__ import annotations

from dataclasses import dataclass
import pathlib
import sys

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_smart_duration_dp.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import _quantize_notes_smart  # noqa: E402

FLOAT_TOL = 1e-12


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


def assert_notes_close(
    case_id: str,
    actual: list[FixtureNote],
    expected: list[FixtureNote],
) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: length mismatch: {actual!r} != {expected!r}")

    for index, (actual_note, expected_note) in enumerate(zip(actual, expected)):
        if abs(actual_note.onset - expected_note.onset) > FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: onset mismatch at {index}: "
                f"{actual_note.onset!r} != {expected_note.onset!r}"
            )
        if abs(actual_note.offset - expected_note.offset) > FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: offset mismatch at {index}: "
                f"{actual_note.offset!r} != {expected_note.offset!r}"
            )
        if abs(actual_note.pitch - expected_note.pitch) > FLOAT_TOL:
            raise AssertionError(
                f"{case_id}: pitch mismatch at {index}: "
                f"{actual_note.pitch!r} != {expected_note.pitch!r}"
            )
        if actual_note.lyric != expected_note.lyric:
            raise AssertionError(
                f"{case_id}: lyric mismatch at {index}: "
                f"{actual_note.lyric!r} != {expected_note.lyric!r}"
            )


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case_id, tempo_raw, step_raw, input_raw, expected_raw = line.split("\t")
        notes = parse_notes(input_raw)
        expected = parse_notes(expected_raw)
        _quantize_notes_smart(notes, float(tempo_raw), int(step_raw))
        try:
            assert_notes_close(case_id, notes, expected)
        except AssertionError as exc:
            raise AssertionError(f"fixture line {line_number}: {exc}") from exc


if __name__ == "__main__":
    main()
