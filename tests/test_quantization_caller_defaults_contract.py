"""Behavior contract tests for quantization callers and defaults."""

from __future__ import annotations

import ast
import csv
import sys
from dataclasses import dataclass
from pathlib import Path

import pytest

PROJECT_ROOT = Path(__file__).resolve().parents[1]
if str(PROJECT_ROOT) not in sys.path:
    sys.path.insert(0, str(PROJECT_ROOT))

from application.config import PipelineConfig
from gui.fluent_utils import parse_quantization, parse_quantization_mode
from inference.quant import quantization
from web_task_manager import TaskManager


FIXTURE_PATH = PROJECT_ROOT / "rewrite-in-rust" / "fixtures" / "quantization_caller_defaults_contract.tsv"


@dataclass
class FakeNote:
    onset: float
    offset: float
    pitch: float = 60.0
    lyric: str = "la"


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


def _fixture_rows() -> list[dict]:
    with FIXTURE_PATH.open(newline="", encoding="utf-8") as handle:
        return list(csv.DictReader(handle, delimiter="\t"))


def _mode_from_fixture(value: str):
    if value == "null":
        return None
    return value


def _snapshot(notes: list[FakeNote]) -> tuple[tuple[float, float, str], ...]:
    return tuple((note.onset, note.offset, note.lyric) for note in notes)


@pytest.mark.parametrize("row", _fixture_rows(), ids=lambda row: row["case_id"])
def test_activation_policy_matches_contract_fixture(row):
    mode = _mode_from_fixture(row["mode_value"])
    step = int(row["quantization_step"])

    assert quantization.should_apply_quantization(mode, step) is (row["expected_should_apply"] == "true")


@pytest.mark.parametrize("row", _fixture_rows(), ids=lambda row: row["case_id"])
def test_quantize_notes_dispatch_matches_contract_fixture(row, monkeypatch):
    calls = []

    def record(name):
        def _inner(notes, tempo, quantization_step):
            calls.append((name, tempo, quantization_step, len(notes)))

        return _inner

    monkeypatch.setattr(quantization, "_quantize_notes_simple", record("simple"))
    monkeypatch.setattr(quantization, "_quantize_notes_smart", record("smart"))
    monkeypatch.setattr(quantization, "_quantize_notes_bayesian", record("bayes"))
    monkeypatch.setattr(quantization, "_quantize_notes_dp_asym", record("dp"))

    mode = _mode_from_fixture(row["mode_value"])
    step = int(row["quantization_step"])
    quantization.quantize_notes([FakeNote(0.1, 0.2)], 120.0, step, mode=mode)

    assert calls == [(row["expected_dispatch"], 120.0, step, 1)]


def test_pipeline_no_quantize_path_sorts_without_mutating_timings(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [
        FakeNote(0.3, 0.4, lyric="second"),
        FakeNote(0.1, 0.2, lyric="first"),
    ]
    exports = []

    monkeypatch.setattr(pipeline.librosa, "load", lambda *args, **kwargs: ([0.0], 44100))
    monkeypatch.setattr(pipeline, "slice_audio", lambda *args, **kwargs: [{"waveform": [0.0]}])
    monkeypatch.setattr(pipeline, "load_game_model", lambda *args, **kwargs: object())
    monkeypatch.setattr(pipeline, "extract_pitches_only_torch", lambda *args, **kwargs: notes)
    monkeypatch.setattr(pipeline, "free_memory", lambda: None)
    monkeypatch.setattr(
        pipeline,
        "quantize_notes_with_backend",
        lambda *args, **kwargs: (_ for _ in ()).throw(AssertionError("quantize_notes should not run")),
    )
    monkeypatch.setattr(pipeline, "_save_midi", lambda export_notes, *args, **kwargs: exports.append(("mid", _snapshot(export_notes))))
    monkeypatch.setattr(pipeline, "_save_text", lambda export_notes, *args, **kwargs: exports.append((args[1], _snapshot(export_notes))))

    pipeline.auto_lyric_hybrid_pipeline(
        audio_path="input.wav",
        output_filename="input",
        game_model_dir="game",
        device="cpu",
        hfa_model_dir="hfa",
        asr_model_path="asr",
        ts=[0.0, 0.5],
        language="zh",
        lyric_output_mode="auto",
        original_lyrics="",
        output_dir=tmp_path,
        output_formats=["mid", "txt", "csv"],
        slicing_method="auto",
        tempo=120.0,
        quantization_step=0,
        pitch_format="midi",
        round_pitch=True,
        quantization_mode="simple",
        seg_threshold=0.2,
        seg_radius=0.02,
        est_threshold=0.2,
        batch_size=1,
        asr_batch_size=1,
        output_lyrics=False,
    )

    sorted_unmutated = ((0.1, 0.2, "first"), (0.3, 0.4, "second"))
    assert exports == [
        ("mid", sorted_unmutated),
        ("txt", sorted_unmutated),
        ("csv", sorted_unmutated),
    ]


def test_pipeline_unknown_positive_mode_uses_simple_fallback_before_export(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5, lyric="before")]
    exports = []
    simple_calls = []

    def fake_simple(export_notes, tempo, quantization_step):
        simple_calls.append((tempo, quantization_step))
        export_notes[0].onset = 0.0
        export_notes[0].offset = 0.5
        export_notes[0].lyric = "simple"

    def fail_dispatch(name):
        def _inner(*args, **kwargs):
            raise AssertionError(f"{name} should not handle unknown mode")

        return _inner

    monkeypatch.setattr(pipeline.librosa, "load", lambda *args, **kwargs: ([0.0], 44100))
    monkeypatch.setattr(pipeline, "slice_audio", lambda *args, **kwargs: [{"waveform": [0.0]}])
    monkeypatch.setattr(pipeline, "load_game_model", lambda *args, **kwargs: object())
    monkeypatch.setattr(pipeline, "extract_pitches_only_torch", lambda *args, **kwargs: notes)
    monkeypatch.setattr(pipeline, "free_memory", lambda: None)
    monkeypatch.setattr(quantization, "_quantize_notes_simple", fake_simple)
    monkeypatch.setattr(quantization, "_quantize_notes_smart", fail_dispatch("smart"))
    monkeypatch.setattr(quantization, "_quantize_notes_bayesian", fail_dispatch("bayes"))
    monkeypatch.setattr(quantization, "_quantize_notes_dp_asym", fail_dispatch("dp"))
    monkeypatch.setattr(pipeline, "_save_midi", lambda export_notes, *args, **kwargs: exports.append(("mid", _snapshot(export_notes))))

    pipeline.auto_lyric_hybrid_pipeline(
        audio_path="input.wav",
        output_filename="input",
        game_model_dir="game",
        device="cpu",
        hfa_model_dir="hfa",
        asr_model_path="asr",
        ts=[0.0, 0.5],
        language="zh",
        lyric_output_mode="auto",
        original_lyrics="",
        output_dir=tmp_path,
        output_formats=["mid"],
        slicing_method="auto",
        tempo=120.0,
        quantization_step=60,
        pitch_format="midi",
        round_pitch=True,
        quantization_mode="unknown",
        seg_threshold=0.2,
        seg_radius=0.02,
        est_threshold=0.2,
        batch_size=1,
        asr_batch_size=1,
        output_lyrics=False,
    )

    assert simple_calls == [(120.0, 60)]
    assert exports == [("mid", ((0.0, 0.5, "simple"),))]


def test_pipeline_dp_step_zero_quantizes_before_all_exports(monkeypatch, tmp_path):
    import inference.pipeline.auto_lyric_hybrid as pipeline

    notes = [FakeNote(0.25, 0.5, lyric="before")]
    exports = []
    quantize_calls = []

    def fake_quantize(export_notes, tempo, quantization_step, *, mode, **kwargs):
        quantize_calls.append((tempo, quantization_step, mode))
        export_notes[0].onset = 0.0
        export_notes[0].offset = 0.75
        export_notes[0].lyric = "after"

    monkeypatch.setattr(pipeline.librosa, "load", lambda *args, **kwargs: ([0.0], 44100))
    monkeypatch.setattr(pipeline, "slice_audio", lambda *args, **kwargs: [{"waveform": [0.0]}])
    monkeypatch.setattr(pipeline, "load_game_model", lambda *args, **kwargs: object())
    monkeypatch.setattr(pipeline, "extract_pitches_only_torch", lambda *args, **kwargs: notes)
    monkeypatch.setattr(pipeline, "free_memory", lambda: None)
    monkeypatch.setattr(pipeline, "quantize_notes_with_backend", fake_quantize)
    monkeypatch.setattr(pipeline, "_save_midi", lambda export_notes, *args, **kwargs: exports.append(("mid", _snapshot(export_notes))))
    monkeypatch.setattr(pipeline, "_save_text", lambda export_notes, *args, **kwargs: exports.append((args[1], _snapshot(export_notes))))
    monkeypatch.setattr(pipeline, "save_ustx", lambda export_notes, *args, **kwargs: exports.append(("ustx", _snapshot(export_notes))))

    pipeline.auto_lyric_hybrid_pipeline(
        audio_path="input.wav",
        output_filename="input",
        game_model_dir="game",
        device="cpu",
        hfa_model_dir="hfa",
        asr_model_path="asr",
        ts=[0.0, 0.5],
        language="zh",
        lyric_output_mode="auto",
        original_lyrics="",
        output_dir=tmp_path,
        output_formats=["mid", "txt", "csv", "ustx"],
        slicing_method="auto",
        tempo=120.0,
        quantization_step=0,
        pitch_format="midi",
        round_pitch=True,
        quantization_mode="dp",
        seg_threshold=0.2,
        seg_radius=0.02,
        est_threshold=0.2,
        batch_size=1,
        asr_batch_size=1,
        output_lyrics=False,
        output_pitch_curve=False,
    )

    assert quantize_calls == [(120.0, 0, "dp")]
    mutated = ((0.0, 0.75, "after"),)
    assert exports == [
        ("mid", mutated),
        ("txt", mutated),
        ("csv", mutated),
        ("ustx", mutated),
    ]


def test_pipeline_config_defaults_and_explicit_quantization_pass_through(tmp_path):
    default_cfg = PipelineConfig(**_required_config_kwargs(tmp_path))
    assert default_cfg.quantization_step == 16
    assert default_cfg.quantization_mode == "bayes"
    assert default_cfg.to_kwargs()["quantization_step"] == 16
    assert default_cfg.to_kwargs()["quantization_mode"] == "bayes"

    explicit_cfg = PipelineConfig(
        **_required_config_kwargs(tmp_path),
        quantization_step=120,
        quantization_mode="dp",
    )
    kwargs = explicit_cfg.to_kwargs()
    assert kwargs["quantization_step"] == 120
    assert kwargs["quantization_mode"] == "dp"


def test_gui_quantization_parsers_lock_current_label_mapping():
    assert parse_quantization("不量化") == 0
    assert parse_quantization("1/4 音符 (1拍)") == 480
    assert parse_quantization("1/8 音符 (1/2拍)") == 240
    assert parse_quantization("1/16 音符 (1/4拍)") == 120
    assert parse_quantization("1/32 音符 (1/8拍)") == 60
    assert parse_quantization("1/64 音符 (1/16拍)") == 30

    assert parse_quantization_mode("开发中") == "bayes"
    assert parse_quantization_mode("SynthV 风格") == "bayes"
    assert parse_quantization_mode("贝叶斯") == "bayes"
    assert parse_quantization_mode("动态规划") == "dp"
    assert parse_quantization_mode("智能") == "smart"
    assert parse_quantization_mode("") == "simple"


def test_auto_lyric_view_keeps_quantization_controls_disabled_and_config_simple_zero():
    source_path = PROJECT_ROOT / "gui" / "auto_lyric_view.py"
    source = source_path.read_text(encoding="utf-8")
    tree = ast.parse(source)

    disabled_controls = {
        node.func.value.attr
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and node.func.attr == "setEnabled"
        and isinstance(node.func.value, ast.Attribute)
        and node.func.value.attr in {"quantize_combo", "quantize_mode_combo"}
        and len(node.args) == 1
        and isinstance(node.args[0], ast.Constant)
        and node.args[0].value is False
    }
    assert disabled_controls == {"quantize_combo", "quantize_mode_combo"}

    pipeline_config_calls = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "PipelineConfig"
    ]
    assert pipeline_config_calls, "AutoLyric view should build PipelineConfig"
    quantization_kwargs = {
        keyword.arg: keyword.value
        for call in pipeline_config_calls
        for keyword in call.keywords
        if keyword.arg in {"quantization_step", "quantization_mode"}
    }

    assert isinstance(quantization_kwargs["quantization_step"], ast.Constant)
    assert quantization_kwargs["quantization_step"].value == 0
    assert isinstance(quantization_kwargs["quantization_mode"], ast.Constant)
    assert quantization_kwargs["quantization_mode"].value == "simple"


def test_web_quantize_settings_remain_visible_but_ignored_by_pipeline_config(tmp_path):
    import web_server

    assert web_server.DEFAULT_SETTINGS["pipeline"]["quantize_precision"] == "none"
    assert web_server.DEFAULT_SETTINGS["pipeline"]["quantize_algorithm"] == "dev"

    config = TaskManager()._build_config(
        {
            "save_dir": str(tmp_path),
            "device": "cpu",
            "language": "zh",
            "tempo": 120,
            "quantize_precision": "1/64",
            "quantize_algorithm": "dp",
        },
        "input.wav",
    )

    assert config.quantization_step == 16
    assert config.quantization_mode == "bayes"
