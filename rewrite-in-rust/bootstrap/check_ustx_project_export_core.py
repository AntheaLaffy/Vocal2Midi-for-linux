"""Check USTX project export fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import math
import pathlib
import sys
import tempfile
from dataclasses import dataclass

import yaml

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "ustx_project_export_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.API.ustx_api import save_ustx  # noqa: E402


@dataclass
class Note:
    onset: float
    offset: float
    pitch: float
    lyric: str = ""


def parse_number(value: str) -> float:
    if value == "nan":
        return float("nan")
    if value == "inf":
        return float("inf")
    if value == "-inf":
        return float("-inf")
    return float(value)


def parse_notes(raw_notes: list[dict[str, str]]) -> list[Note]:
    return [
        Note(
            onset=parse_number(raw_note["onset"]),
            offset=parse_number(raw_note["offset"]),
            pitch=parse_number(raw_note["pitch"]),
            lyric=raw_note.get("lyric", ""),
        )
        for raw_note in raw_notes
    ]


def expected_skipped(notes: list[Note]) -> int:
    skipped = 0
    for note in notes:
        if not (
            all(math.isfinite(value) for value in (note.onset, note.offset, note.pitch))
            and note.offset > note.onset
        ):
            skipped += 1
    return skipped


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text(encoding="utf8").splitlines(), start=1):
        if not line or line.startswith("#"):
            continue

        case = json.loads(line)
        case_id = case["case_id"]
        notes = parse_notes(case["notes"])
        expected_path = PROJECT_ROOT / case["expected_yaml"]
        expected_yaml = expected_path.read_text(encoding="utf8")

        with tempfile.TemporaryDirectory() as tmp_dir:
            output_path = pathlib.Path(tmp_dir) / f"{case['stem']}.ustx"
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                save_ustx(notes, output_path, tempo=float(case["tempo"]), rmvpe_result=None)
            actual_yaml = output_path.read_text(encoding="utf8")

        if actual_yaml != expected_yaml:
            raise AssertionError(
                f"{case_id}: YAML mismatch at fixture line {line_number}: "
                f"{actual_yaml!r} != {expected_yaml!r}"
            )

        skipped = expected_skipped(notes)
        if skipped != int(case["skipped_invalid_notes"]):
            raise AssertionError(
                f"{case_id}: skipped count mismatch: {skipped!r} != {case['skipped_invalid_notes']!r}"
            )

        warning_output = stdout.getvalue()
        if skipped and f"Skipped {skipped} invalid note(s)" not in warning_output:
            raise AssertionError(f"{case_id}: missing skipped-note warning in stdout")
        if not skipped and warning_output:
            raise AssertionError(f"{case_id}: unexpected stdout: {warning_output!r}")

        loaded = yaml.safe_load(actual_yaml)
        voice_parts = loaded["voice_parts"]
        if len(voice_parts) != 1:
            raise AssertionError(f"{case_id}: expected one voice part")
        if voice_parts[0]["curves"] != []:
            raise AssertionError(f"{case_id}: project export core must keep curves empty")


if __name__ == "__main__":
    main()
