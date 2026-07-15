"""Check Web pipeline execution event fixtures against legacy Python."""

from __future__ import annotations

import datetime as datetime_module
import json
import pathlib
import shutil
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_pipeline_execution_events.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import application.pipeline as application_pipeline  # noqa: E402
import web_task_manager  # noqa: E402
from web_task_manager import Task, TaskManager  # noqa: E402


class FakeDateTime:
    counter = 0
    base = datetime_module.datetime(2026, 7, 16, 12, 0, 0)

    @classmethod
    def now(cls):
        value = cls.base + datetime_module.timedelta(seconds=cls.counter)
        cls.counter += 1
        return value


class FakeStopEvent:
    def __init__(self):
        self._set = False

    def set(self):
        self._set = True

    def is_set(self):
        return self._set


class FakeStream:
    def __init__(self):
        self.writes: list[str] = []

    def write(self, text: str) -> None:
        self.writes.append(text)

    def flush(self) -> None:
        pass


class FakeSocketIO:
    def __init__(self):
        self.emits: list[dict[str, Any]] = []

    def emit(self, event: str, payload: dict[str, Any], room=None):
        self.emits.append({"event": event, "payload": payload, "room": room})


class FakeConfig:
    def __init__(self, *, output_dir: pathlib.Path, language: str, device: str):
        self.output_dir = output_dir
        self.language = language
        self.device = device
        self.cancel_checker = None


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


def make_task(case: dict[str, Any]) -> Task:
    return Task(
        id=case["task_id"],
        status="running",
        progress=0,
        stage="idle",
        config={"case_id": case["case_id"]},
        audio_file_path=case["audio_path"],
        created_at=datetime_module.datetime(2026, 7, 16, 11, 59, 0),
        stop_event=FakeStopEvent(),
    )


def setup_output_files(output_dir: pathlib.Path, names: list[str]) -> None:
    output_dir.mkdir(parents=True, exist_ok=True)
    for name in names:
        (output_dir / name).write_text("fixture", encoding="utf-8")


def normalized_logs(task: Task) -> list[list[str]]:
    return [[entry["message"], entry["level"]] for entry in task.logs]


def normalized_emit_events(socketio: FakeSocketIO) -> list[dict[str, Any]]:
    events = []
    for emit in socketio.emits:
        payload = emit["payload"]
        normalized_payload: dict[str, Any] = {}
        for key in (
            "task_id",
            "message",
            "level",
            "timestamp",
            "progress",
            "stage",
            "status",
            "error",
        ):
            if key in payload:
                normalized_payload[key] = payload[key]
        if "result" in payload:
            result = payload["result"]
            normalized_payload["result"] = {
                "output_dir": result.get("output_dir"),
                "files": result.get("files", []),
            }
        events.append(
            {
                "event": emit["event"],
                "room": emit["room"],
                "payload": normalized_payload,
            }
        )
    return events


def normalized_progress_events(socketio: FakeSocketIO) -> list[dict[str, Any]]:
    return [
        {
            "progress": emit["payload"]["progress"],
            "stage": emit["payload"]["stage"],
        }
        for emit in socketio.emits
        if emit["event"] == "progress"
    ]


def normalized_status_changes(socketio: FakeSocketIO) -> list[dict[str, Any]]:
    changes = []
    for emit in socketio.emits:
        if emit["event"] != "status_change":
            continue
        payload = emit["payload"]
        result = payload.get("result") or {}
        changes.append(
            {
                "status": payload["status"],
                "error": payload.get("error"),
                "output_dir": result.get("output_dir"),
                "files": result.get("files", []),
            }
        )
    return changes


def assert_log_matches(case_id: str, actual: list[list[str]], expected: list[list[str]]) -> None:
    if len(actual) != len(expected):
        raise AssertionError(f"{case_id}: log length {len(actual)} != {len(expected)}")
    for index, (actual_item, expected_item) in enumerate(zip(actual, expected)):
        actual_message, actual_level = actual_item
        expected_message, expected_level = expected_item
        if actual_level != expected_level:
            raise AssertionError(
                f"{case_id}: log {index} level {actual_level!r} != {expected_level!r}"
            )
        if expected_message.startswith("__contains__:"):
            needle = expected_message.removeprefix("__contains__:")
            if needle not in actual_message:
                raise AssertionError(
                    f"{case_id}: log {index} {actual_message!r} does not contain {needle!r}"
                )
        elif actual_message != expected_message:
            raise AssertionError(
                f"{case_id}: log {index} message {actual_message!r} != {expected_message!r}"
            )


def assert_payload_value(case_id: str, path: str, actual: Any, expected: Any) -> None:
    if isinstance(expected, str) and expected.startswith("__contains__:"):
        needle = expected.removeprefix("__contains__:")
        if not isinstance(actual, str) or needle not in actual:
            raise AssertionError(f"{case_id}: {path} {actual!r} does not contain {needle!r}")
        return

    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {path} expected dict, got {type(actual).__name__}")
        if set(actual) != set(expected):
            raise AssertionError(
                f"{case_id}: {path} keys {sorted(actual)!r} != {sorted(expected)!r}"
            )
        for key, expected_value in expected.items():
            assert_payload_value(case_id, f"{path}.{key}", actual[key], expected_value)
        return

    if isinstance(expected, list):
        if not isinstance(actual, list):
            raise AssertionError(f"{case_id}: {path} expected list, got {type(actual).__name__}")
        if len(actual) != len(expected):
            raise AssertionError(f"{case_id}: {path} length {len(actual)} != {len(expected)}")
        for index, (actual_value, expected_value) in enumerate(zip(actual, expected)):
            assert_payload_value(case_id, f"{path}[{index}]", actual_value, expected_value)
        return

    if actual != expected:
        raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")


def assert_emit_events(case_id: str, actual: list[dict[str, Any]], expected: list[dict[str, Any]]) -> None:
    assert_payload_value(case_id, "emit_events", actual, expected)


def assert_rooms(case_id: str, socketio: FakeSocketIO, task_id: str) -> None:
    for emit in socketio.emits:
        if emit["room"] != task_id:
            raise AssertionError(f"{case_id}: emit room {emit['room']!r} != {task_id!r}")


def run_case(case: dict[str, Any], tmp_root: pathlib.Path) -> None:
    case_id = case["case_id"]
    case_dir = tmp_root / case_id
    case_dir.mkdir(parents=True, exist_ok=True)
    case = replace_case_placeholder(case, case_dir)
    expected = case["expect"]

    output_dir = pathlib.Path(case["output_dir"])
    if "config_error" not in case:
        setup_output_files(output_dir, case.get("output_files", []))

    task = make_task(case)
    socketio = FakeSocketIO()
    manager = TaskManager()
    fake_config = FakeConfig(
        output_dir=output_dir,
        language=case["language"],
        device=case["device"],
    )

    def fake_build_config(frontend_config, audio_path):
        if "config_error" in case:
            raise RuntimeError(case["config_error"])
        if frontend_config != task.config:
            raise AssertionError("unexpected frontend_config")
        if audio_path != task.audio_file_path:
            raise AssertionError("unexpected audio_path")
        return fake_config

    def fake_run_auto_lyric_job(config):
        if config is not fake_config:
            raise AssertionError("unexpected config object")
        for line in case.get("stdout_lines", []):
            print(line)
        for line in case.get("stderr_lines", []):
            print(line, file=sys.stderr)

        run_result = case["run_result"]
        if run_result == "completed":
            return None
        if run_result == "stop_after_run":
            task.stop_event.set()
            return None
        if run_result == "keyboard_interrupt":
            raise KeyboardInterrupt()
        if run_result == "stopped_error":
            task.stop_event.set()
            raise RuntimeError(case["error_message"])
        if run_result == "generic_error":
            raise RuntimeError(case["error_message"])
        raise AssertionError(f"unknown run_result {run_result!r}")

    original_build_config = manager._build_config
    original_run = application_pipeline.run_auto_lyric_job
    original_datetime = web_task_manager.datetime
    original_stdout = sys.stdout
    original_stderr = sys.stderr
    fake_stdout = FakeStream()
    fake_stderr = FakeStream()
    sys.stdout = fake_stdout
    sys.stderr = fake_stderr
    try:
        FakeDateTime.counter = 0
        web_task_manager.datetime = FakeDateTime
        manager._build_config = fake_build_config
        application_pipeline.run_auto_lyric_job = fake_run_auto_lyric_job
        manager._execute_pipeline(task, socketio)

        stdout_restored = sys.stdout is fake_stdout
        stderr_restored = sys.stderr is fake_stderr
        if stdout_restored != expected["stdout_restored"]:
            raise AssertionError(
                f"{case_id}: stdout_restored {stdout_restored!r} != {expected['stdout_restored']!r}"
            )
        if stderr_restored != expected["stderr_restored"]:
            raise AssertionError(
                f"{case_id}: stderr_restored {stderr_restored!r} != {expected['stderr_restored']!r}"
            )
    finally:
        manager._build_config = original_build_config
        application_pipeline.run_auto_lyric_job = original_run
        web_task_manager.datetime = original_datetime
        sys.stdout = original_stdout
        sys.stderr = original_stderr

    if task.status != expected["status"]:
        raise AssertionError(f"{case_id}: status {task.status!r} != {expected['status']!r}")
    if task.progress != expected["progress"]:
        raise AssertionError(f"{case_id}: progress {task.progress!r} != {expected['progress']!r}")
    if task.stage != expected["stage"]:
        raise AssertionError(f"{case_id}: stage {task.stage!r} != {expected['stage']!r}")
    if task.error != expected["error"]:
        raise AssertionError(f"{case_id}: error {task.error!r} != {expected['error']!r}")
    if bool(task.completed_at) != expected["completed_at_present"]:
        raise AssertionError(f"{case_id}: completed_at presence mismatch")
    if bool(fake_config.cancel_checker) != expected["cancel_checker_set"]:
        raise AssertionError(f"{case_id}: cancel_checker presence mismatch")
    if task.output_files != expected["output_files"]:
        raise AssertionError(
            f"{case_id}: output_files {task.output_files!r} != {expected['output_files']!r}"
        )

    assert_log_matches(case_id, normalized_logs(task), expected["logs"])
    progress_events = normalized_progress_events(socketio)
    if progress_events != expected["progress_events"]:
        raise AssertionError(
            f"{case_id}: progress_events {progress_events!r} != {expected['progress_events']!r}"
        )
    status_changes = normalized_status_changes(socketio)
    if status_changes != expected["status_changes"]:
        raise AssertionError(
            f"{case_id}: status_changes {status_changes!r} != {expected['status_changes']!r}"
        )
    assert_emit_events(case_id, normalized_emit_events(socketio), expected["emit_events"])
    assert_rooms(case_id, socketio, task.id)


def main() -> None:
    tmp_root = pathlib.Path(tempfile.mkdtemp(prefix="v2m-web-pipeline-events-"))
    try:
        for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
            if not line or line.startswith("#"):
                continue
            try:
                case = json.loads(line)
            except json.JSONDecodeError as exc:
                raise AssertionError(f"fixture line {line_number} is invalid JSON: {exc}") from exc
            run_case(case, tmp_root)
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)


if __name__ == "__main__":
    main()
