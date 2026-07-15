"""Promotion-seam tests for quantization pipeline routing."""

from __future__ import annotations

import os
import sys
import time
from dataclasses import dataclass
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from application.config import PipelineConfig
from inference.quant.rust_bridge import QuantizationBridgeError, quantize_notes_with_backend


@dataclass
class FakeNote:
    onset: float
    offset: float
    pitch: float = 60.0
    lyric: str = "la"


def _snapshot(notes: list[FakeNote]) -> tuple[tuple[float, float, str], ...]:
    return tuple((note.onset, note.offset, note.lyric) for note in notes)


def _required_config_kwargs(tmp_path: Path) -> dict:
    return {
        "audio_path": "input.wav",
        "output_filename": "input",
        "output_dir": tmp_path,
        "game_model_dir": "game",
        "hfa_model_dir": "hfa",
        "asr_model_path": "asr",
        "device": "cpu",
        "language": "zh",
        "ts": [0.0, 0.5],
    }


def _install_fake_pipeline(monkeypatch, notes: list[FakeNote], exports: list) -> None:
    import inference.pipeline.auto_lyric_hybrid as pipeline

    monkeypatch.setattr(pipeline.librosa, "load", lambda *args, **kwargs: ([0.0], 44100))
    monkeypatch.setattr(pipeline, "slice_audio", lambda *args, **kwargs: [{"waveform": [0.0]}])
    monkeypatch.setattr(pipeline, "load_game_model", lambda *args, **kwargs: object())
    monkeypatch.setattr(pipeline, "extract_pitches_only_torch", lambda *args, **kwargs: notes)
    monkeypatch.setattr(pipeline, "free_memory", lambda: None)
    monkeypatch.setattr(
        pipeline,
        "_save_midi",
        lambda export_notes, *args, **kwargs: exports.append(("mid", _snapshot(export_notes))),
    )
    monkeypatch.setattr(
        pipeline,
        "_save_text",
        lambda export_notes, *args, **kwargs: exports.append((args[1], _snapshot(export_notes))),
    )
    monkeypatch.setattr(
        pipeline,
        "save_ustx",
        lambda export_notes, *args, **kwargs: exports.append(("ustx", _snapshot(export_notes))),
    )


def _run_fake_pipeline(tmp_path: Path, **overrides) -> None:
    import inference.pipeline.auto_lyric_hybrid as pipeline

    kwargs = {
        "audio_path": "input.wav",
        "output_filename": "input",
        "game_model_dir": "game",
        "device": "cpu",
        "hfa_model_dir": "hfa",
        "asr_model_path": "asr",
        "ts": [0.0, 0.5],
        "language": "zh",
        "lyric_output_mode": "auto",
        "original_lyrics": "",
        "output_dir": tmp_path,
        "output_formats": ["mid", "txt", "csv", "ustx"],
        "slicing_method": "auto",
        "tempo": 120.0,
        "quantization_step": 60,
        "pitch_format": "midi",
        "round_pitch": True,
        "quantization_mode": "simple",
        "seg_threshold": 0.2,
        "seg_radius": 0.02,
        "est_threshold": 0.2,
        "batch_size": 1,
        "asr_batch_size": 1,
        "output_lyrics": False,
        "output_pitch_curve": False,
    }
    kwargs.update(overrides)
    pipeline.auto_lyric_hybrid_pipeline(**kwargs)


def _write_executable(path: Path, body: str) -> Path:
    path.write_text(f"#!{sys.executable}\n{body}", encoding="utf-8")
    path.chmod(0o755)
    return path


def _assert_process_dead(pid: int) -> None:
    deadline = time.time() + 2.0
    while time.time() < deadline:
        try:
            os.kill(pid, 0)
        except ProcessLookupError:
            return
        time.sleep(0.05)
    raise AssertionError(f"process {pid} is still alive")


def test_pipeline_routes_quantization_through_legacy_wrapper_by_default(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5, lyric="before")]
    exports = []
    calls = []
    _install_fake_pipeline(monkeypatch, notes, exports)

    def fake_wrapper(export_notes, tempo, quantization_step, *, mode, backend, executable, timeout_sec, cancel_checker):
        calls.append((tempo, quantization_step, mode, backend, executable, timeout_sec, cancel_checker is None))
        export_notes[0].onset = 0.0
        export_notes[0].offset = 0.75
        export_notes[0].lyric = "legacy"

    monkeypatch.setattr(pipeline, "quantize_notes_with_backend", fake_wrapper)

    _run_fake_pipeline(tmp_path)

    assert calls == [(120.0, 60, "simple", None, None, 30.0, True)]
    assert exports == [
        ("mid", ((0.0, 0.75, "legacy"),)),
        ("txt", ((0.0, 0.75, "legacy"),)),
        ("csv", ((0.0, 0.75, "legacy"),)),
        ("ustx", ((0.0, 0.75, "legacy"),)),
    ]


def test_pipeline_passes_explicit_rust_json_backend_configuration(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5)]
    exports = []
    calls = []
    bridge_path = tmp_path / "v2m-quant-bridge"
    _install_fake_pipeline(monkeypatch, notes, exports)

    def fake_wrapper(export_notes, tempo, quantization_step, *, mode, backend, executable, timeout_sec, cancel_checker):
        calls.append((backend, executable, timeout_sec, cancel_checker is not None))
        export_notes[0].onset = 0.125
        export_notes[0].offset = 0.625

    monkeypatch.setattr(pipeline, "quantize_notes_with_backend", fake_wrapper)

    _run_fake_pipeline(
        tmp_path,
        quantization_backend="rust-json",
        quantization_bridge_bin=str(bridge_path),
        quantization_timeout_sec=1.25,
        cancel_checker=lambda: False,
    )

    assert calls == [("rust-json", str(bridge_path), 1.25, True)]
    assert exports[0] == ("mid", ((0.125, 0.625, "la"),))


def test_pipeline_explicit_legacy_backend_rolls_back_with_bridge_config_present(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5)]
    exports = []
    calls = []
    _install_fake_pipeline(monkeypatch, notes, exports)

    def fake_wrapper(export_notes, tempo, quantization_step, *, mode, backend, executable, timeout_sec, cancel_checker):
        calls.append((backend, executable))
        export_notes[0].lyric = "rollback"

    monkeypatch.setattr(pipeline, "quantize_notes_with_backend", fake_wrapper)

    _run_fake_pipeline(
        tmp_path,
        quantization_backend="legacy",
        quantization_bridge_bin=str(tmp_path / "configured-bridge"),
    )

    assert calls == [("legacy", str(tmp_path / "configured-bridge"))]
    assert exports[0] == ("mid", ((0.25, 0.5, "rollback"),))


def test_pipeline_skips_wrapper_when_activation_policy_is_disabled(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.3, 0.4, lyric="second"), FakeNote(0.1, 0.2, lyric="first")]
    exports = []
    _install_fake_pipeline(monkeypatch, notes, exports)
    monkeypatch.setattr(
        pipeline,
        "quantize_notes_with_backend",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("wrapper should not run")),
    )

    _run_fake_pipeline(
        tmp_path,
        output_formats=["mid"],
        quantization_step=0,
        quantization_mode="simple",
    )

    assert exports == [("mid", ((0.1, 0.2, "first"), (0.3, 0.4, "second")))]


def test_pipeline_checks_cancellation_after_quantization_before_export(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5)]
    exports = []
    state = {"cancelled": False}
    _install_fake_pipeline(monkeypatch, notes, exports)

    def fake_wrapper(export_notes, *args, **kwargs):
        export_notes[0].onset = 0.0
        state["cancelled"] = True

    monkeypatch.setattr(pipeline, "quantize_notes_with_backend", fake_wrapper)

    with pytest.raises(InterruptedError):
        _run_fake_pipeline(tmp_path, cancel_checker=lambda: state["cancelled"])

    assert exports == []


def test_pipeline_config_passes_quantization_backend_fields(tmp_path):
    cfg = PipelineConfig(
        **_required_config_kwargs(tmp_path),
        quantization_backend="rust-json",
        quantization_bridge_bin="/opt/v2m-quant-bridge",
        quantization_timeout_sec=2.5,
    )

    kwargs = cfg.to_kwargs()
    assert kwargs["quantization_backend"] == "rust-json"
    assert kwargs["quantization_bridge_bin"] == "/opt/v2m-quant-bridge"
    assert kwargs["quantization_timeout_sec"] == 2.5


def test_rust_bridge_rejects_unsupported_backend():
    with pytest.raises(QuantizationBridgeError, match="unsupported quantization backend"):
        quantize_notes_with_backend([], 120.0, 60, backend="unsupported")


def test_rust_bridge_uses_environment_backend_when_backend_is_not_explicit(monkeypatch):
    import inference.quant.rust_bridge as rust_bridge

    calls = []

    def fake_legacy(notes, tempo, quantization_step, mode):
        calls.append((tempo, quantization_step, mode, len(notes)))

    monkeypatch.setenv("V2M_QUANT_BACKEND", "legacy")
    monkeypatch.setattr(rust_bridge, "legacy_quantize_notes", fake_legacy)

    quantize_notes_with_backend([FakeNote(0.1, 0.2)], 120.0, 60, mode="simple", backend=None)

    assert calls == [(120.0, 60, "simple", 1)]


def test_rust_bridge_rejects_missing_and_non_executable_paths(tmp_path):
    with pytest.raises(QuantizationBridgeError, match="not found"):
        quantize_notes_with_backend([], 120.0, 60, backend="rust-json", executable=tmp_path / "missing")

    not_executable = tmp_path / "not-executable"
    not_executable.write_text("#!/bin/sh\n", encoding="utf-8")
    not_executable.chmod(0o644)

    with pytest.raises(QuantizationBridgeError, match="bridge process failed to start"):
        quantize_notes_with_backend([], 120.0, 60, backend="rust-json", executable=not_executable)


def test_rust_bridge_timeout_kills_child_process(tmp_path):
    pid_file = tmp_path / "timeout.pid"
    script = _write_executable(
        tmp_path / "sleep_bridge.py",
        f"""
import os
import pathlib
import sys
import time
pathlib.Path({str(pid_file)!r}).write_text(str(os.getpid()), encoding='utf-8')
sys.stdin.read()
time.sleep(30)
""",
    )

    with pytest.raises(QuantizationBridgeError, match="bridge timed out"):
        quantize_notes_with_backend([], 120.0, 60, backend="rust-json", executable=script, timeout_sec=0.1)

    _assert_process_dead(int(pid_file.read_text(encoding="utf-8")))


def test_rust_bridge_timeout_covers_non_reading_child_stdin_delivery(tmp_path):
    pid_file = tmp_path / "non_reading_timeout.pid"
    script = _write_executable(
        tmp_path / "non_reading_timeout_bridge.py",
        f"""
import os
import pathlib
import time
pathlib.Path({str(pid_file)!r}).write_text(str(os.getpid()), encoding='utf-8')
time.sleep(30)
""",
    )
    notes = [FakeNote(index / 1000.0, index / 1000.0 + 0.1) for index in range(4000)]

    started = time.monotonic()
    with pytest.raises(QuantizationBridgeError, match="bridge timed out"):
        quantize_notes_with_backend(notes, 120.0, 60, backend="rust-json", executable=script, timeout_sec=0.1)

    assert time.monotonic() - started < 1.0
    _assert_process_dead(int(pid_file.read_text(encoding="utf-8")))


def test_rust_bridge_cancel_checker_kills_in_flight_child_process(tmp_path):
    marker_file = tmp_path / "started"
    pid_file = tmp_path / "cancel.pid"
    script = _write_executable(
        tmp_path / "cancel_bridge.py",
        f"""
import os
import pathlib
import sys
import time
pathlib.Path({str(pid_file)!r}).write_text(str(os.getpid()), encoding='utf-8')
pathlib.Path({str(marker_file)!r}).write_text('started', encoding='utf-8')
sys.stdin.read()
time.sleep(30)
""",
    )

    with pytest.raises(InterruptedError):
        quantize_notes_with_backend(
            [],
            120.0,
            60,
            backend="rust-json",
            executable=script,
            timeout_sec=5.0,
            cancel_checker=lambda: marker_file.exists(),
        )

    _assert_process_dead(int(pid_file.read_text(encoding="utf-8")))


def test_rust_bridge_cancel_checker_covers_non_reading_child_stdin_delivery(tmp_path):
    marker_file = tmp_path / "non_reading_started"
    pid_file = tmp_path / "non_reading_cancel.pid"
    script = _write_executable(
        tmp_path / "non_reading_cancel_bridge.py",
        f"""
import os
import pathlib
import time
pathlib.Path({str(pid_file)!r}).write_text(str(os.getpid()), encoding='utf-8')
pathlib.Path({str(marker_file)!r}).write_text('started', encoding='utf-8')
time.sleep(30)
""",
    )
    notes = [FakeNote(index / 1000.0, index / 1000.0 + 0.1) for index in range(4000)]

    with pytest.raises(InterruptedError):
        quantize_notes_with_backend(
            notes,
            120.0,
            60,
            backend="rust-json",
            executable=script,
            timeout_sec=5.0,
            cancel_checker=lambda: marker_file.exists(),
        )

    _assert_process_dead(int(pid_file.read_text(encoding="utf-8")))
