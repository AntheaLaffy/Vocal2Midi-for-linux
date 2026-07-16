"""Check MIDI export fixtures against legacy Python and Mido."""

from __future__ import annotations

import json
import math
import pathlib
import sys
import tempfile
from typing import Any

import mido

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "midi_export_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.io.note_io import NoteInfo, _save_midi  # noqa: E402


def parse_number(value: str) -> float:
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return float(value)


def parse_notes(raw_notes: list[dict[str, str]]) -> list[NoteInfo]:
    return [
        NoteInfo(
            onset=parse_number(raw_note["onset"]),
            offset=parse_number(raw_note["offset"]),
            pitch=parse_number(raw_note["pitch"]),
            lyric=raw_note.get("lyric", ""),
        )
        for raw_note in raw_notes
    ]


def expected_skipped(raw_notes: list[dict[str, str]]) -> int:
    skipped = 0
    for raw_note in raw_notes:
        onset = parse_number(raw_note["onset"])
        offset = parse_number(raw_note["offset"])
        pitch = parse_number(raw_note["pitch"])
        if not (all(math.isfinite(value) for value in (onset, offset, pitch)) and offset > onset):
            skipped += 1
    return skipped


def event_payload(message: Any) -> dict[str, Any]:
    payload = message.dict()
    event_type = payload.pop("type")
    delta_ticks = payload.pop("time")
    return {"type": event_type, "delta_ticks": delta_ticks, **payload}


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = case["case_id"]
        raw_notes = case["notes"]
        notes = parse_notes(raw_notes)
        expect = case["expect"]

        with tempfile.TemporaryDirectory() as tmp_dir:
            output_path = pathlib.Path(tmp_dir) / f"{case_id}.mid"
            _save_midi(notes, output_path, tempo=int(case["tempo"]))
            midi_bytes = output_path.read_bytes()
            midi_file = mido.MidiFile(output_path, charset="utf8")

        actual_events = [event_payload(message) for message in midi_file.tracks[0]]
        if actual_events != expect["events"]:
            raise AssertionError(
                f"{case_id}: event mismatch at fixture line {line_number}: "
                f"{actual_events!r} != {expect['events']!r}"
            )

        if midi_file.type != expect["type"]:
            raise AssertionError(f"{case_id}: MIDI type mismatch: {midi_file.type!r} != {expect['type']!r}")

        if midi_file.ticks_per_beat != expect["ticks_per_beat"]:
            raise AssertionError(
                f"{case_id}: ticks_per_beat mismatch: "
                f"{midi_file.ticks_per_beat!r} != {expect['ticks_per_beat']!r}"
            )

        actual_hex = midi_bytes.hex()
        if actual_hex != expect["midi_hex"]:
            raise AssertionError(f"{case_id}: MIDI hex mismatch: {actual_hex!r} != {expect['midi_hex']!r}")

        skipped = expected_skipped(raw_notes)
        if skipped != expect["skipped_invalid_notes"]:
            raise AssertionError(
                f"{case_id}: skipped count mismatch: {skipped!r} != {expect['skipped_invalid_notes']!r}"
            )


if __name__ == "__main__":
    main()
