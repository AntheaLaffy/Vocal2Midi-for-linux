"""Check Web PipelineConfig mapping fixtures against legacy Python."""

from __future__ import annotations

import json
import math
import os
import pathlib
import shutil
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_pipeline_config_mapping.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from web_task_manager import TaskManager  # noqa: E402


EXPECTED_KWARG_KEYS = {
    "audio_path",
    "output_filename",
    "output_dir",
    "game_model_dir",
    "hfa_model_dir",
    "asr_model_path",
    "device",
    "language",
    "ts",
    "lyric_output_mode",
    "original_lyrics",
    "output_formats",
    "slicing_method",
    "slice_min_sec",
    "slice_max_sec",
    "tempo",
    "quantization_step",
    "quantization_mode",
    "quantization_backend",
    "quantization_bridge_bin",
    "quantization_timeout_sec",
    "pitch_format",
    "round_pitch",
    "seg_threshold",
    "seg_radius",
    "est_threshold",
    "batch_size",
    "asr_batch_size",
    "output_lyrics",
    "rmvpe_model_path",
    "phoneme_asr_model_path",
    "output_pitch_curve",
    "debug_mode",
    "cancel_checker",
}


def replace_case_placeholder(value: Any, case_dir: pathlib.Path) -> Any:
    if isinstance(value, str):
        return value.replace("__case__", str(case_dir))
    if isinstance(value, list):
        return [replace_case_placeholder(item, case_dir) for item in value]
    if isinstance(value, dict):
        return {
            key: replace_case_placeholder(item, case_dir)
            for key, item in value.items()
        }
    return value


def assert_equal(case_id: str, key: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, float):
        if not math.isclose(float(actual), expected, rel_tol=0.0, abs_tol=1e-9):
            raise AssertionError(f"{case_id}: {key} {actual!r} != {expected!r}")
        return
    if isinstance(expected, list) and expected and all(
        isinstance(item, float) for item in expected
    ):
        if len(actual) != len(expected):
            raise AssertionError(f"{case_id}: {key} length {len(actual)} != {len(expected)}")
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected)):
            if not math.isclose(float(actual_item), expected_item, rel_tol=0.0, abs_tol=1e-9):
                raise AssertionError(
                    f"{case_id}: {key}[{index}] {actual_item!r} != {expected_item!r}"
                )
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {key} {actual!r} != {expected!r}")


def check_case(case: dict[str, Any], tmp_root: pathlib.Path) -> None:
    case_id = case["case_id"]
    case_dir = tmp_root / case_id
    case_dir.mkdir(parents=True, exist_ok=True)

    audio_path = replace_case_placeholder(case["audio_path"], case_dir)
    frontend_config = replace_case_placeholder(case["config"], case_dir)
    for setup_file in replace_case_placeholder(case.get("setup_files", []), case_dir):
        setup_path = pathlib.Path(setup_file)
        setup_path.parent.mkdir(parents=True, exist_ok=True)
        setup_path.write_text("fixture", encoding="utf-8")
    expected = replace_case_placeholder(case.get("expected"), case_dir)
    expected_error = case.get("expected_error")

    original_cwd = pathlib.Path.cwd()
    try:
        os.chdir(case_dir)
        try:
            config = TaskManager()._build_config(frontend_config, audio_path)
        except Exception as exc:
            if not expected_error:
                raise
            if type(exc).__name__ != expected_error["python_type"]:
                raise AssertionError(
                    f"{case_id}: error type {type(exc).__name__!r} != "
                    f"{expected_error['python_type']!r}"
                ) from exc
            return
        if expected_error:
            raise AssertionError(f"{case_id}: expected error {expected_error!r}")
        kwargs = config.to_kwargs()
        if set(kwargs) != EXPECTED_KWARG_KEYS:
            raise AssertionError(
                f"{case_id}: kwargs keys mismatch: "
                f"missing={sorted(EXPECTED_KWARG_KEYS - set(kwargs))}, "
                f"extra={sorted(set(kwargs) - EXPECTED_KWARG_KEYS)}"
            )

        actual = {
            key: kwargs[key]
            for key in expected
            if key not in {"output_dir"}
        }
        actual["output_dir"] = str(config.output_dir)

        for key, expected_value in expected.items():
            assert_equal(case_id, key, actual[key], expected_value)

        if config.cancel_checker is not None:
            raise AssertionError(f"{case_id}: cancel_checker should be None")
        if config.debug_mode is not False:
            raise AssertionError(f"{case_id}: debug_mode should default to False")
        if not config.output_dir.exists():
            raise AssertionError(f"{case_id}: output_dir was not created")
    finally:
        os.chdir(original_cwd)


def main() -> None:
    tmp_root = pathlib.Path(tempfile.mkdtemp(prefix="v2m-web-config-"))
    try:
        for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
            if not line or line.startswith("#"):
                continue
            try:
                case = json.loads(line)
            except json.JSONDecodeError as exc:
                raise AssertionError(f"fixture line {line_number} is invalid JSON: {exc}") from exc
            check_case(case, tmp_root)
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)


if __name__ == "__main__":
    main()
