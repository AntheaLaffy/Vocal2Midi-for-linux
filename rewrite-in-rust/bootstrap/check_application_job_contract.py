"""Check application job contract fixtures against legacy Python."""

from __future__ import annotations

import pathlib
import shutil
import sys
import tempfile

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "application_job_contract.tsv"

sys.path.insert(0, str(PROJECT_ROOT))

import application.pipeline as pipeline  # noqa: E402
from application.config import PipelineConfig  # noqa: E402
from application.exceptions import (  # noqa: E402
    CancellationError,
    ModelNotFoundError,
    Vocal2MidiError,
)


def parse_bool(value: str) -> bool:
    if value == "true":
        return True
    if value == "false":
        return False
    raise AssertionError(f"unknown bool value {value!r}")


def cancel_checker_from_marker(marker: str):
    if marker == "none":
        return None
    if marker == "false":
        return lambda: False
    if marker == "true":
        return lambda: True
    raise AssertionError(f"unknown cancel checker marker {marker!r}")


def decode_empty(value: str) -> str:
    if value == "__empty__":
        return ""
    return value


def labels_from_fixture(value: str) -> list[str]:
    if value == "__empty__":
        return []
    return value.split("|")


def resolve_fixture_path(base_dir: pathlib.Path, marker: str, name: str) -> str:
    if marker == "empty":
        return ""
    if marker == "missing":
        return str(base_dir / f"missing-{name}")
    if marker == "exists_dir":
        path = base_dir / f"{name}-dir"
        path.mkdir(parents=True, exist_ok=True)
        return str(path)
    if marker == "exists_file":
        path = base_dir / f"{name}.bin"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("fixture", encoding="utf-8")
        return str(path)
    raise AssertionError(f"unknown path marker {marker!r}")


def make_config(
    case_dir: pathlib.Path,
    *,
    output_lyrics: bool,
    game_path: str,
    hfa_path: str,
    asr_path: str,
    cancel_checker_marker: str,
) -> PipelineConfig:
    return PipelineConfig(
        audio_path="input.wav",
        output_filename="input.wav",
        output_dir=case_dir / "output",
        game_model_dir=game_path,
        hfa_model_dir=hfa_path,
        asr_model_path=asr_path,
        device="cpu",
        language="zh",
        ts=[],
        output_lyrics=output_lyrics,
        cancel_checker=cancel_checker_from_marker(cancel_checker_marker),
    )


def classify_exception(exc: BaseException) -> tuple[str, str, str]:
    if isinstance(exc, ModelNotFoundError):
        return "model_not_found", str(exc), exc.details
    if isinstance(exc, CancellationError):
        return "cancelled", str(exc), exc.details
    if isinstance(exc, Vocal2MidiError):
        return "vocal2midi_error", str(exc), exc.details
    raise AssertionError(f"unexpected exception type {type(exc).__name__}: {exc}")


def assert_model_details(case_id: str, details: str, labels: list[str]) -> None:
    parts = details.split("; ") if details else []
    if len(parts) != len(labels):
        raise AssertionError(
            f"{case_id}: detail count mismatch: {parts!r} for labels {labels!r}"
        )

    for part, label in zip(parts, labels):
        expected_prefix = f"{label}不存在或无效: "
        if not part.startswith(expected_prefix):
            raise AssertionError(
                f"{case_id}: detail {part!r} does not start with {expected_prefix!r}"
            )


def run_fixture_case(
    tmp_root: pathlib.Path,
    fields: list[str],
    original_pipeline,
) -> None:
    (
        case_id,
        output_lyrics_raw,
        game_path_marker,
        hfa_path_marker,
        asr_path_marker,
        cancel_checker_marker,
        pipeline_result,
        expected_kind,
        expected_message_raw,
        expected_details_raw,
        expected_detail_labels_raw,
        expected_pipeline_called_raw,
    ) = fields

    case_dir = tmp_root / case_id
    case_dir.mkdir(parents=True, exist_ok=True)
    game_path = resolve_fixture_path(case_dir, game_path_marker, "game")
    hfa_path = resolve_fixture_path(case_dir, hfa_path_marker, "hfa")
    asr_path = resolve_fixture_path(case_dir, asr_path_marker, "asr")
    expected_message = decode_empty(expected_message_raw)
    expected_details = decode_empty(expected_details_raw)
    expected_detail_labels = labels_from_fixture(expected_detail_labels_raw)
    expected_pipeline_called = parse_bool(expected_pipeline_called_raw)

    called = {"value": False, "kwargs": None}

    def fake_pipeline(**kwargs):
        called["value"] = True
        called["kwargs"] = kwargs
        if pipeline_result == "ok":
            return None
        if pipeline_result == "interrupted":
            raise InterruptedError("stop")
        if pipeline_result == "v2m_error":
            raise Vocal2MidiError("legacy failure", details="legacy details")
        if pipeline_result == "generic":
            raise RuntimeError("boom")
        raise AssertionError(f"unknown pipeline result {pipeline_result!r}")

    pipeline.auto_lyric_hybrid_pipeline = fake_pipeline
    config = make_config(
        case_dir,
        output_lyrics=parse_bool(output_lyrics_raw),
        game_path=game_path,
        hfa_path=hfa_path,
        asr_path=asr_path,
        cancel_checker_marker=cancel_checker_marker,
    )

    try:
        try:
            pipeline.run_auto_lyric_job(config)
            actual_kind = "ok"
            actual_message = ""
            actual_details = ""
        except Exception as exc:
            actual_kind, actual_message, actual_details = classify_exception(exc)
    finally:
        pipeline.auto_lyric_hybrid_pipeline = original_pipeline

    if actual_kind != expected_kind:
        raise AssertionError(f"{case_id}: kind {actual_kind!r} != {expected_kind!r}")
    if actual_message != expected_message:
        raise AssertionError(
            f"{case_id}: message {actual_message!r} != {expected_message!r}"
        )
    if expected_details == "__model_path_details__":
        assert_model_details(case_id, actual_details, expected_detail_labels)
    elif actual_details != expected_details:
        raise AssertionError(
            f"{case_id}: details {actual_details!r} != {expected_details!r}"
        )
    if called["value"] != expected_pipeline_called:
        raise AssertionError(
            f"{case_id}: pipeline called {called['value']!r} != "
            f"{expected_pipeline_called!r}"
        )
    if expected_pipeline_called and called["kwargs"] != config.to_kwargs():
        raise AssertionError(
            f"{case_id}: pipeline kwargs {called['kwargs']!r} != "
            f"{config.to_kwargs()!r}"
        )


def main() -> None:
    original_pipeline = pipeline.auto_lyric_hybrid_pipeline
    tmp_root = pathlib.Path(tempfile.mkdtemp(prefix="v2m-app-contract-"))
    try:
        for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
            if not line or line.startswith("#"):
                continue
            fields = line.split("\t")
            if len(fields) != 12:
                raise AssertionError(
                    f"fixture line {line_number} has {len(fields)} fields, expected 12"
                )
            run_fixture_case(tmp_root, fields, original_pipeline)
    finally:
        pipeline.auto_lyric_hybrid_pipeline = original_pipeline
        shutil.rmtree(tmp_root, ignore_errors=True)


if __name__ == "__main__":
    main()
