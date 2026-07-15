"""Check Web model download process-plan fixtures against legacy Python."""

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
    / "web_model_download_process_plan_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import web_model_download_manager as model_module  # noqa: E402
from web_model_download_manager import ModelDownloadManager, ModelDownloadTask  # noqa: E402


class FakeSocket:
    def __init__(self) -> None:
        self.emits: list[dict[str, Any]] = []

    def emit(self, event: str, payload: dict[str, Any], room: str | None = None) -> None:
        self.emits.append(
            {
                "event": event,
                "room": room,
                "payload": normalize_value(payload),
            }
        )


class FakeStdout:
    def __init__(self, output: str) -> None:
        self.output = output
        self.index = 0

    def read(self, size: int) -> str:
        if self.index >= len(self.output):
            return ""
        char = self.output[self.index : self.index + size]
        self.index += len(char)
        return char


class FakeProcess:
    def __init__(self, output: str | None) -> None:
        self.stdout = FakeStdout(output) if output is not None else None

    def poll(self):
        if self.stdout is None:
            return 0
        return 0 if self.stdout.index >= len(self.stdout.output) else None


def parse_datetime(value: str | None):
    if value is None:
        return None
    return RealDateTime.fromisoformat(value)


def make_task(data: dict[str, Any]) -> ModelDownloadTask:
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
    task.completed_models = set(data.get("completed_models", []))
    task.active_model = data.get("active_model")
    return task


def normalize_value(value: Any) -> Any:
    if isinstance(value, dict):
        normalized = {key: normalize_value(item) for key, item in value.items()}
        if "timestamp" in normalized:
            normalized["timestamp"] = "__timestamp__"
        return normalized
    if isinstance(value, list):
        return [normalize_value(item) for item in value]
    return value


def completed_models(task: ModelDownloadTask) -> list[str]:
    ordered = [model_id for model_id in task.selected_models if model_id in task.completed_models]
    extras = sorted(model_id for model_id in task.completed_models if model_id not in ordered)
    return ordered + extras


def task_state(task: ModelDownloadTask) -> dict[str, Any]:
    logs = normalize_value(task.logs)
    state = {
        "task_id": task.id,
        "selected_models": task.selected_models,
        "qwen_source": task.qwen_source,
        "force": task.force,
        "proxy_mode": task.proxy_mode,
        "proxy_url": task.proxy_url,
        "status": task.status,
        "progress": task.progress,
        "stage": task.stage,
        "created_at": task.created_at.isoformat(),
        "started_at": task.started_at.isoformat() if task.started_at else None,
        "completed_at": task.completed_at.isoformat() if task.completed_at else None,
        "error": task.error,
        "returncode": task.returncode,
        "logs": logs,
        "log_count": len(task.logs),
        "completed_models": completed_models(task),
        "active_model": task.active_model,
        "stop_event_present": task.stop_event is not None,
    }
    if logs:
        state["first_log_message"] = logs[0].get("message")
        state["last_log_message"] = logs[-1].get("message")
    return state


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


def assert_expected(case: dict[str, Any], actual: dict[str, Any]) -> None:
    assert_subset(case["case_id"], actual, case["expect"])


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


def run_command(case: dict[str, Any]) -> None:
    restores = [
        patch_attr(model_module.sys, "executable", case["python_executable"]),
        patch_attr(model_module, "ROOT_DIR", pathlib.Path(case["root_dir"])),
    ]
    try:
        task = make_task(case["task"])
        command = ModelDownloadManager()._build_command(task)
        assert_expected(case, {"command": command})
    finally:
        for restore in reversed(restores):
            restore()


def run_env(case: dict[str, Any]) -> None:
    env = model_module.os.environ
    old_env = dict(env)
    try:
        env.clear()
        env.update(case["base_env"])
        task = make_task(case["task"])
        actual_env = ModelDownloadManager()._build_process_env(task)
        expected_keys = case["expect"]["env_subset"].keys()
        actual = {
            "env_subset": {
                key: actual_env[key]
                for key in expected_keys
                if key in actual_env
            },
            "present_proxy_keys": [
                key for key in model_module.PROXY_ENV_KEYS if key in actual_env
            ],
        }
        assert_expected(case, actual)
    finally:
        env.clear()
        env.update(old_env)


def run_popen_kwargs(case: dict[str, Any]) -> None:
    restores = [patch_attr(model_module.os, "name", case["os_name"])]
    if "create_new_process_group" in case:
        restores.append(
            patch_attr(
                model_module.subprocess,
                "CREATE_NEW_PROCESS_GROUP",
                case["create_new_process_group"],
            )
        )
    try:
        kwargs = ModelDownloadManager()._popen_process_group_kwargs()
        assert_expected(case, {"kwargs": kwargs})
    finally:
        for restore in reversed(restores):
            restore()


def process_actual(task: ModelDownloadTask, socket: FakeSocket) -> dict[str, Any]:
    return {
        "task": task_state(task),
        "emits": socket.emits,
    }


def run_handle_lines(case: dict[str, Any]) -> None:
    manager = ModelDownloadManager()
    socket = FakeSocket()
    task = make_task(case["task"])
    for line in case["lines"]:
        manager._handle_output_line(task, socket, line)
    assert_expected(case, process_actual(task, socket))


def run_read_output(case: dict[str, Any]) -> None:
    manager = ModelDownloadManager()
    socket = FakeSocket()
    task = make_task(case["task"])
    process = FakeProcess(case["output"])
    manager._read_process_output(task, socket, process)
    assert_expected(case, process_actual(task, socket))


def run_log_cap(case: dict[str, Any]) -> None:
    manager = ModelDownloadManager()
    socket = FakeSocket()
    task = make_task(case["task"])
    task.logs = [
        {
            "task_id": task.id,
            "task_type": "model_download",
            "message": f"old-{index}",
            "level": "info",
            "timestamp": f"old-ts-{index}",
        }
        for index in range(case["prelog_count"])
    ]
    manager._emit_log(task, socket, case["message"], case["level"])
    assert_expected(case, process_actual(task, socket))


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "command":
        run_command(case)
    elif operation == "env":
        run_env(case)
    elif operation == "popen_kwargs":
        run_popen_kwargs(case)
    elif operation == "handle_lines":
        run_handle_lines(case)
    elif operation == "read_output":
        run_read_output(case)
    elif operation == "log_cap":
        run_log_cap(case)
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
