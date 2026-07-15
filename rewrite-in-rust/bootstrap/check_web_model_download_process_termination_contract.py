"""Check Web model download process-termination fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
from datetime import datetime as RealDateTime
from typing import Any, Callable

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "web_model_download_process_termination_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import web_model_download_manager as model_module  # noqa: E402
from web_model_download_manager import ModelDownloadManager, ModelDownloadTask  # noqa: E402


class FakeStopEvent:
    def __init__(self, is_set: bool) -> None:
        self._is_set = is_set

    def is_set(self) -> bool:
        return self._is_set

    def set(self) -> None:
        self._is_set = True


class FakeProcess:
    def __init__(self, spec: dict[str, Any]) -> None:
        self.pid = spec["pid"]
        self._poll = spec.get("poll")
        self.calls: list[str] = []

    def poll(self):
        return self._poll

    def terminate(self) -> None:
        self.calls.append("terminate")

    def kill(self) -> None:
        self.calls.append("kill")


class RecordingManager(ModelDownloadManager):
    def __init__(self, raises_oserror: bool = False) -> None:
        super().__init__()
        self.raises_oserror = raises_oserror
        self.termination_calls: list[dict[str, Any]] = []

    def _terminate_process_tree(self, process, force: bool = False) -> None:
        self.termination_calls.append({"pid": process.pid, "force": force})
        if self.raises_oserror:
            raise OSError("termination failed")


def parse_datetime(value: str | None):
    if value is None:
        return None
    return RealDateTime.fromisoformat(value)


def make_task(data: dict[str, Any], process: FakeProcess | None = None) -> ModelDownloadTask:
    task = ModelDownloadTask(
        id=data["task_id"],
        selected_models=list(data["selected_models"]),
        qwen_source=data["qwen_source"],
        force=data["force"],
        proxy_mode=data["proxy_mode"],
        proxy_url=data["proxy_url"],
        status=data["status"],
        progress=data["progress"],
        stage=data["stage"],
        created_at=parse_datetime(data["created_at"]),
        started_at=parse_datetime(data.get("started_at")),
        completed_at=parse_datetime(data.get("completed_at")),
        error=data.get("error"),
        returncode=data.get("returncode"),
        logs=list(data.get("logs", [])),
    )
    task.stop_event = (
        FakeStopEvent(data.get("stop_event_set", False))
        if data.get("stop_event_present", True)
        else None
    )
    task.process = process
    return task


def task_state(task: ModelDownloadTask) -> dict[str, Any]:
    return {
        "status": task.status,
        "stop_event_present": task.stop_event is not None,
        "stop_event_set": bool(task.stop_event and task.stop_event.is_set()),
    }


def assert_subset(case_id: str, actual: Any, expected: Any, path: str = "") -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {path} actual is not object")
        for key, expected_value in expected.items():
            if key not in actual:
                raise AssertionError(f"{case_id}: missing key {path}.{key}")
            assert_subset(case_id, actual[key], expected_value, f"{path}.{key}")
        return
    if isinstance(expected, list):
        if not isinstance(actual, list):
            raise AssertionError(f"{case_id}: {path} actual is not list")
        if len(actual) != len(expected):
            raise AssertionError(
                f"{case_id}: {path} list length {len(actual)} != {len(expected)}"
            )
        for index, (actual_item, expected_item) in enumerate(zip(actual, expected)):
            assert_subset(case_id, actual_item, expected_item, f"{path}[{index}]")
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")


def patch_attr(obj: Any, name: str, value: Any) -> Callable[[], None]:
    had_attr = hasattr(obj, name)
    old_value = getattr(obj, name, None)
    setattr(obj, name, value)

    def restore() -> None:
        if had_attr:
            setattr(obj, name, old_value)
        else:
            delattr(obj, name)

    return restore


def run_terminate(case: dict[str, Any]) -> None:
    process = FakeProcess(case["process"])
    killpg_calls: list[dict[str, Any]] = []
    taskkill_calls: list[dict[str, Any]] = []

    def fake_killpg(pid, sig):
        signal_name = "SIGKILL" if sig == model_module.signal.SIGKILL else "SIGTERM"
        killpg_calls.append({"pid": pid, "signal": signal_name})
        if case.get("killpg_outcome") == "process_lookup_error":
            raise ProcessLookupError("missing process group")
        if case.get("killpg_outcome") == "os_error":
            raise OSError("killpg failed")

    def fake_run(command, stdout, stderr, check):
        taskkill_calls.append(
            {
                "command": list(command),
                "stdout_devnull": stdout is model_module.subprocess.DEVNULL,
                "stderr_devnull": stderr is model_module.subprocess.DEVNULL,
                "check": check,
            }
        )
        if case.get("taskkill_outcome") == "os_error":
            raise OSError("taskkill failed")

    restores = [
        patch_attr(model_module.os, "name", case["os_name"]),
        patch_attr(model_module.os, "killpg", fake_killpg),
        patch_attr(model_module.subprocess, "run", fake_run),
    ]
    try:
        ModelDownloadManager()._terminate_process_tree(process, force=case["force"])
    finally:
        for restore in reversed(restores):
            restore()

    actual = {
        "killpg_calls": killpg_calls,
        "taskkill_calls": taskkill_calls,
        "process_calls": process.calls,
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_stop_task(case: dict[str, Any]) -> None:
    process = FakeProcess(case["process"])
    manager = RecordingManager(case.get("terminate_raises_oserror", False))
    task = make_task(case["task"], process)
    manager.tasks[task.id] = task
    success = manager.stop_task(task.id)
    actual = {
        "success": success,
        "task": task_state(task),
        "termination_calls": manager.termination_calls,
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "terminate":
        run_terminate(case)
    elif operation == "stop_task":
        run_stop_task(case)
    else:
        raise AssertionError(f"unknown operation {operation!r}")


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        try:
            case = json.loads(line)
        except json.JSONDecodeError as exc:
            raise AssertionError(
                f"fixture line {line_number} is invalid JSON: {exc}"
            ) from exc
        run_case(case)


if __name__ == "__main__":
    main()
