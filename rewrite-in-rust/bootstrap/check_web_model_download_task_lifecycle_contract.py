"""Check Web model download task lifecycle fixtures against legacy Python."""

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
    / "web_model_download_task_lifecycle_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import web_model_download_manager as model_module  # noqa: E402
from web_model_download_manager import ModelDownloadManager, ModelDownloadTask  # noqa: E402


class FakeEvent:
    def __init__(self, set_initially: bool = False) -> None:
        self.set_called = set_initially

    def set(self) -> None:
        self.set_called = True

    def is_set(self) -> bool:
        return self.set_called


class FakeThread:
    def __init__(self, target, daemon: bool, name: str) -> None:
        self.target = target
        self.daemon = daemon
        self.name = name
        self.started = False
        self.target_called = False

    def start(self) -> None:
        self.started = True


class FakeThreading:
    Event = FakeEvent
    Thread = FakeThread


class FakeDateTime(RealDateTime):
    values: list[RealDateTime] = []

    @classmethod
    def now(cls, tz=None):
        if not cls.values:
            raise AssertionError("FakeDateTime exhausted")
        return cls.values.pop(0)


def parse_datetime(value: str | None):
    if value is None:
        return None
    return RealDateTime.fromisoformat(value)


def make_event(data: dict[str, Any]):
    if not data.get("stop_event_present", False):
        return None
    return FakeEvent(bool(data.get("stop_event_set", False)))


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
        stop_event=make_event(data),
    )
    return task


def thread_state(thread) -> dict[str, Any] | None:
    if thread is None:
        return None
    return {
        "name": thread.name,
        "daemon": thread.daemon,
        "started": bool(getattr(thread, "started", False)),
        "target_called": bool(getattr(thread, "target_called", False)),
    }


def event_present(task: ModelDownloadTask) -> bool:
    return task.stop_event is not None


def event_set(task: ModelDownloadTask) -> bool:
    if task.stop_event is None:
        return False
    if hasattr(task.stop_event, "is_set"):
        return bool(task.stop_event.is_set())
    return bool(getattr(task.stop_event, "set_called", False))


def task_state(task: ModelDownloadTask | None) -> dict[str, Any] | None:
    if task is None:
        return None
    return {
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
        "logs": task.logs,
        "stop_event_present": event_present(task),
        "stop_event_set": event_set(task),
        "thread": thread_state(task.thread),
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


def configure_times(times: list[str]) -> None:
    FakeDateTime.values = [RealDateTime.fromisoformat(value) for value in times]


def manager_with_tasks(case: dict[str, Any]) -> ModelDownloadManager:
    manager = ModelDownloadManager()
    manager.threading = FakeThreading
    for task_data in case.get("tasks", []):
        task = make_task(task_data)
        manager.tasks[task.id] = task
    manager.active_task_id = case.get("active_task_id")
    return manager


def run_create_task(case: dict[str, Any]) -> None:
    configure_times(case["times"])
    restores = [
        patch_attr(model_module.uuid, "uuid4", lambda: case["uuid"]),
        patch_attr(model_module, "datetime", FakeDateTime),
    ]
    try:
        manager = manager_with_tasks(case)
        selected_models = list(case["selected_models"])
        kwargs = {}
        if "proxy_mode" in case:
            kwargs["proxy_mode"] = case["proxy_mode"]
        if "proxy_url" in case:
            kwargs["proxy_url"] = case["proxy_url"]
        task = manager.create_task(
            selected_models,
            case["qwen_source"],
            case["force"],
            **kwargs,
        )
        if "mutate_selected_after" in case:
            selected_models[:] = case["mutate_selected_after"]
        actual = {
            "task": task_state(task),
            "registered": manager.tasks.get(task.id) is task,
            "active_task_id": manager.active_task_id,
            "registry_ids": list(manager.tasks.keys()),
        }
        assert_expected(case, actual)
    finally:
        for restore in reversed(restores):
            restore()


def run_get_task(case: dict[str, Any]) -> None:
    manager = manager_with_tasks(case)
    actual = {
        "results": [
            (
                {"task_id": task.id, "status": task.status}
                if (task := manager.get_task(task_id))
                else {"task_id": None}
            )
            for task_id in case["queries"]
        ]
    }
    assert_expected(case, actual)


def run_active_task(case: dict[str, Any]) -> None:
    manager = manager_with_tasks(case)
    task = manager.active_task()
    assert_expected(case, {"active_task_id": task.id if task else None})


def run_active_task_matrix(case: dict[str, Any]) -> None:
    manager = manager_with_tasks(case)
    results = []
    for check in case["checks"]:
        manager.active_task_id = check["active_task_id"]
        task = manager.active_task()
        results.append({"active_task_id": task.id if task else None})
    assert_expected(case, {"results": results})


def run_start_task(case: dict[str, Any]) -> None:
    configure_times(case["times"])
    restores = [
        patch_attr(model_module.uuid, "uuid4", lambda: case["uuid"]),
        patch_attr(model_module, "datetime", FakeDateTime),
    ]
    try:
        manager = manager_with_tasks(case)
        try:
            task = manager.start_task(
                selected_models=list(case["selected_models"]),
                qwen_source=case["qwen_source"],
                force=case["force"],
                proxy_mode=case["proxy_mode"],
                proxy_url=case["proxy_url"],
                socketio_instance=object(),
            )
        except RuntimeError as exc:
            actual = {
                "error": str(exc),
                "active_task_id": manager.active_task_id,
                "registry_ids": list(manager.tasks.keys()),
            }
        else:
            actual = {
                "task": task_state(task),
                "active_task_id": manager.active_task_id,
                "registry_ids": list(manager.tasks.keys()),
            }
        assert_expected(case, actual)
    finally:
        for restore in reversed(restores):
            restore()


def run_stop_task_matrix(case: dict[str, Any]) -> None:
    manager = manager_with_tasks(case)
    results = []
    for task_id in case["stops"]:
        success = manager.stop_task(task_id)
        task = manager.get_task(task_id)
        task_result = None
        if task is not None:
            task_result = {
                "status": task.status,
                "stop_event_present": event_present(task),
                "stop_event_set": event_set(task),
            }
        results.append(
            {
                "task_id": task_id,
                "success": success,
                "task": task_result,
            }
        )
    assert_expected(case, {"results": results})


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "create_task":
        run_create_task(case)
    elif operation == "get_task":
        run_get_task(case)
    elif operation == "active_task":
        run_active_task(case)
    elif operation == "active_task_matrix":
        run_active_task_matrix(case)
    elif operation == "start_task":
        run_start_task(case)
    elif operation == "stop_task_matrix":
        run_stop_task_matrix(case)
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
