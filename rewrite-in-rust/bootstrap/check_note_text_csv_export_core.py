"""Check TXT/CSV note export fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import sys
import tempfile

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "note_text_csv_export_core.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.io.note_io import NoteInfo, _save_text  # noqa: E402


def decode_escaped(value: str) -> str:
    result: list[str] = []
    index = 0
    while index < len(value):
        char = value[index]
        if char != "\\":
            result.append(char)
            index += 1
            continue
        index += 1
        if index >= len(value):
            result.append("\\")
            break
        escaped = value[index]
        if escaped == "t":
            result.append("\t")
        elif escaped == "n":
            result.append("\n")
        elif escaped == "r":
            result.append("\r")
        elif escaped == "\\":
            result.append("\\")
        else:
            result.append(escaped)
        index += 1
    return "".join(result)


def parse_number(value: str) -> float:
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return float(value)


def parse_lyric(value: str) -> str:
    if value == "__empty__":
        return ""
    return decode_escaped(value)


def parse_notes(value: str) -> list[NoteInfo]:
    if not value:
        return []

    notes = []
    for raw_note in value.split("|"):
        onset_raw, offset_raw, pitch_raw, lyric_raw = raw_note.split(",", 3)
        notes.append(
            NoteInfo(
                onset=parse_number(onset_raw),
                offset=parse_number(offset_raw),
                pitch=parse_number(pitch_raw),
                lyric=parse_lyric(lyric_raw),
            )
        )
    return notes


def parse_bool(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise AssertionError(f"unknown bool {value!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        (
            case_id,
            file_format,
            pitch_format,
            round_pitch_raw,
            notes_raw,
            skipped_raw,
            expected_raw,
        ) = line.split("\t")
        notes = parse_notes(notes_raw)
        expected = decode_escaped(expected_raw)

        with tempfile.TemporaryDirectory() as tmp_dir:
            output_path = pathlib.Path(tmp_dir) / f"{case_id}.{file_format}"
            _save_text(
                notes,
                output_path,
                file_format,  # type: ignore[arg-type]
                pitch_format,  # type: ignore[arg-type]
                parse_bool(round_pitch_raw),
            )
            actual = output_path.read_bytes().decode("utf8")

        if actual != expected:
            raise AssertionError(
                f"{case_id}: output mismatch at fixture line {line_number}: "
                f"{actual!r} != {expected!r}"
            )

        skipped = sum(
            not (
                all(value == value and value not in (float("inf"), float("-inf"))
                    for value in (note.onset, note.offset, note.pitch))
                and note.offset > note.onset
            )
            for note in notes
        )
        if skipped != int(skipped_raw):
            raise AssertionError(
                f"{case_id}: skipped count mismatch: {skipped!r} != {skipped_raw!r}"
            )


if __name__ == "__main__":
    main()
