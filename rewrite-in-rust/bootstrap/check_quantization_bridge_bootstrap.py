"""Check the quantization Rust JSON bridge against legacy Python behavior."""

from __future__ import annotations

from dataclasses import dataclass
import copy
import json
import math
import os
import pathlib
import subprocess
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
PAYLOAD_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_bridge_payloads.jsonl"
ERROR_FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "quantization_bridge_errors.jsonl"
DEFAULT_BRIDGE_BIN = REWRITE_ROOT / "rust" / "target" / "debug" / "v2m-quant-bridge"
FLOAT_TOL = 1e-12

sys.path.insert(0, str(PROJECT_ROOT))

from inference.quant.quantization import (  # noqa: E402
    quantize_notes,
    should_apply_quantization,
)
from inference.quant.rust_bridge import (  # noqa: E402
    QuantizationBridgeError,
    quantize_notes_with_backend,
)


@dataclass
class FixtureNote:
    onset: float
    offset: float
    pitch: float
    lyric: str = ""


@dataclass
class TimingOnlyNote:
    onset: float
    offset: float


def bridge_bin() -> pathlib.Path:
    return pathlib.Path(os.environ.get("V2M_QUANT_BRIDGE_BIN", DEFAULT_BRIDGE_BIN))


def iter_jsonl(path: pathlib.Path) -> list[dict[str, Any]]:
    rows = []
    for line_number, line in enumerate(path.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        try:
            rows.append(json.loads(line))
        except json.JSONDecodeError as exc:
            raise AssertionError(f"{path}:{line_number}: invalid JSONL row") from exc
    return rows


def run_bridge(stdin: str) -> subprocess.CompletedProcess[str]:
    executable = bridge_bin()
    if not executable.exists():
        raise AssertionError(
            f"bridge binary not found at {executable}; run "
            "`cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml "
            "--bin v2m-quant-bridge` first"
        )
    return subprocess.run(
        [str(executable)],
        input=stdin,
        text=True,
        capture_output=True,
        check=False,
    )


def notes_from_request(request: dict[str, Any]) -> list[FixtureNote]:
    return [
        FixtureNote(
            onset=float(item["onset"]),
            offset=float(item["offset"]),
            pitch=float(item["pitch"]),
            lyric=item.get("lyric") or "",
        )
        for item in request["notes"]
    ]


def expected_legacy_response(request: dict[str, Any]) -> dict[str, Any]:
    notes = notes_from_request(request)
    applied = should_apply_quantization(request.get("mode"), int(request["quantization_step"]))
    if applied:
        quantize_notes(
            notes,
            float(request["tempo"]),
            int(request["quantization_step"]),
            mode=request.get("mode"),
        )

    if applied:
        original_positions = sorted(
            range(len(request["notes"])),
            key=lambda index: float(request["notes"][index]["onset"]),
        )
    else:
        original_positions = list(range(len(request["notes"])))

    response_notes = []
    for note, original_position in zip(notes, original_positions):
        response_notes.append(
            {
                "index": request["notes"][original_position]["index"],
                "onset": note.onset,
                "offset": note.offset,
            }
        )
    return {"ok": True, "applied": applied, "notes": response_notes}


def assert_float_close(case_id: str, field: str, actual: float, expected: float) -> None:
    if abs(actual - expected) > FLOAT_TOL:
        raise AssertionError(f"{case_id}: {field} {actual!r} != {expected!r}")


def assert_response_close(
    case_id: str,
    actual: dict[str, Any],
    expected: dict[str, Any],
) -> None:
    if actual.get("ok") != expected.get("ok"):
        raise AssertionError(f"{case_id}: ok mismatch {actual!r} != {expected!r}")
    if actual.get("applied") != expected.get("applied"):
        raise AssertionError(f"{case_id}: applied mismatch {actual!r} != {expected!r}")

    actual_notes = actual.get("notes")
    expected_notes = expected.get("notes")
    if len(actual_notes) != len(expected_notes):
        raise AssertionError(f"{case_id}: note count mismatch {actual!r} != {expected!r}")

    for index, (actual_note, expected_note) in enumerate(zip(actual_notes, expected_notes)):
        if actual_note["index"] != expected_note["index"]:
            raise AssertionError(
                f"{case_id}: index mismatch at {index}: "
                f"{actual_note!r} != {expected_note!r}"
            )
        assert_float_close(
            case_id,
            f"notes[{index}].onset",
            actual_note["onset"],
            expected_note["onset"],
        )
        assert_float_close(
            case_id,
            f"notes[{index}].offset",
            actual_note["offset"],
            expected_note["offset"],
        )


def check_payload_fixtures() -> None:
    for row in iter_jsonl(PAYLOAD_FIXTURE_PATH):
        case_id = row["case_id"]
        request = row["request"]
        completed = run_bridge(json.dumps(request))
        if completed.returncode != 0:
            raise AssertionError(
                f"{case_id}: bridge failed with {completed.returncode}: "
                f"stdout={completed.stdout!r} stderr={completed.stderr!r}"
            )
        actual = json.loads(completed.stdout)
        assert_response_close(case_id, actual, row["expected"])
        assert_response_close(case_id, actual, expected_legacy_response(copy.deepcopy(request)))


def check_error_fixtures() -> None:
    for row in iter_jsonl(ERROR_FIXTURE_PATH):
        case_id = row["case_id"]
        stdin = row.get("stdin")
        if stdin is None:
            stdin = json.dumps(row["request"])
        completed = run_bridge(stdin)
        if completed.returncode == 0:
            raise AssertionError(f"{case_id}: expected bridge failure")
        response = json.loads(completed.stdout)
        if response.get("ok") is not False:
            raise AssertionError(f"{case_id}: expected ok=false: {response!r}")
        error = response.get("error") or {}
        if error.get("code") != row["error_code"]:
            raise AssertionError(
                f"{case_id}: error code mismatch {error!r} != {row['error_code']!r}"
            )


def check_python_wrapper() -> None:
    notes = [
        FixtureNote(onset=0.2, offset=0.31, pitch=62.0, lyric="late"),
        FixtureNote(onset=0.02, offset=0.1, pitch=60.0, lyric="early"),
    ]
    expected = copy.deepcopy(notes)
    quantize_notes(expected, 120.0, 60, mode="bayes")

    result = quantize_notes_with_backend(notes, 120.0, 60, mode="bayes", backend="legacy")
    if result is not None:
        raise AssertionError("legacy wrapper must return None")
    assert_wrapper_notes("legacy_backend", notes, expected)

    rust_notes = [
        FixtureNote(onset=0.2, offset=0.31, pitch=62.0, lyric="late"),
        FixtureNote(onset=0.02, offset=0.1, pitch=60.0, lyric="early"),
    ]
    result = quantize_notes_with_backend(
        rust_notes,
        120.0,
        60,
        mode="bayes",
        backend="rust-json",
        executable=bridge_bin(),
    )
    if result is not None:
        raise AssertionError("rust-json wrapper must return None")
    assert_wrapper_notes("rust_backend", rust_notes, expected)

    disabled_notes = [
        FixtureNote(onset=0.2, offset=0.3, pitch=62.0, lyric="beta"),
        FixtureNote(onset=0.1, offset=0.15, pitch=60.0, lyric="alpha"),
    ]
    original_ids = [id(note) for note in disabled_notes]
    quantize_notes_with_backend(
        disabled_notes,
        120.0,
        0,
        mode="simple",
        backend="rust-json",
        executable=bridge_bin(),
    )
    if [id(note) for note in disabled_notes] != original_ids:
        raise AssertionError("disabled rust-json path must preserve original order")

    timing_only_notes = [TimingOnlyNote(onset=0.03125, offset=0.09375)]
    quantize_notes_with_backend(
        timing_only_notes,
        120.0,
        60,
        mode="simple",
        backend="rust-json",
        executable=bridge_bin(),
    )
    assert_float_close("timing_only_note", "onset", timing_only_notes[0].onset, 0.0)
    assert_float_close("timing_only_note", "offset", timing_only_notes[0].offset, 0.125)
    if hasattr(timing_only_notes[0], "pitch"):
        raise AssertionError("timing-only wrapper path must not add pitch metadata")

    try:
        quantize_notes_with_backend(
            [FixtureNote(onset=math.inf, offset=0.1, pitch=60.0, lyric="bad")],
            120.0,
            60,
            mode="simple",
            backend="rust-json",
            executable=bridge_bin(),
        )
    except QuantizationBridgeError as exc:
        if "invalid_note: note 0 onset must be finite" not in str(exc):
            raise AssertionError(f"unexpected non-finite note error: {exc}") from exc
    else:
        raise AssertionError("non-finite note timing should raise QuantizationBridgeError")

    try:
        quantize_notes_with_backend(
            [],
            120.0,
            60,
            mode="simple",
            backend="rust-json",
            executable=PROJECT_ROOT / "does-not-exist",
        )
    except QuantizationBridgeError as exc:
        if "not found" not in str(exc):
            raise AssertionError(f"unexpected missing-binary error: {exc}") from exc
    else:
        raise AssertionError("missing bridge executable should raise QuantizationBridgeError")

    try:
        quantize_notes_with_backend(
            [],
            120.0,
            60,
            mode="simple",
            backend="rust-json",
            executable=PROJECT_ROOT,
        )
    except QuantizationBridgeError as exc:
        if "bridge process failed to start" not in str(exc):
            raise AssertionError(f"unexpected startup error: {exc}") from exc
    else:
        raise AssertionError("directory bridge executable should raise QuantizationBridgeError")


def assert_wrapper_notes(
    case_id: str,
    actual: list[FixtureNote],
    expected: list[FixtureNote],
) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: length mismatch")
    for index, (actual_note, expected_note) in enumerate(zip(actual, expected)):
        assert_float_close(case_id, f"note[{index}].onset", actual_note.onset, expected_note.onset)
        assert_float_close(case_id, f"note[{index}].offset", actual_note.offset, expected_note.offset)
        if actual_note.pitch != expected_note.pitch:
            raise AssertionError(f"{case_id}: pitch changed at {index}")
        if actual_note.lyric != expected_note.lyric:
            raise AssertionError(f"{case_id}: lyric changed at {index}")


def main() -> None:
    check_payload_fixtures()
    check_error_fixtures()
    check_python_wrapper()


if __name__ == "__main__":
    main()
