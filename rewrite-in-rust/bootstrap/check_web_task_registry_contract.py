"""Check Web task registry fixtures against legacy Python."""

from __future__ import annotations

import datetime as datetime_module
import json
import pathlib
import sys
import uuid as uuid_module
from collections import deque
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_task_registry_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import web_task_manager  # noqa: E402
from web_task_manager import TaskManager  # noqa: E402


class FakeEvent:
    def __init__(self):
        self._set = False

    def set(self):
        self._set = True

    def is_set(self):
        return self._set


class FakeThread:
    def __init__(self, target, daemon, name):
        self.target = target
        self.daemon = daemon
        self.name = name
        self.started = False

    def start(self):
        self.started = True


class FakeThreading:
    Event = FakeEvent
    Thread = FakeThread


class FakeDateTime:
    values: deque[datetime_module.datetime] = deque()

    @classmethod
    def now(cls):
        if not cls.values:
            raise AssertionError("FakeDateTime.now() called too many times")
        return cls.values.popleft()


def iso_to_datetime(value: str) -> datetime_module.datetime:
    return datetime_module.datetime.fromisoformat(value)


def patch_environment(task_ids: str | list[str], times: list[str]):
    original_uuid4 = web_task_manager.uuid.uuid4
    original_datetime = web_task_manager.datetime
    if isinstance(task_ids, str):
        task_id_values = [task_ids]
    else:
        task_id_values = task_ids
    uuid_values = deque(task_id_values)

    def fake_uuid4():
        if not uuid_values:
            raise AssertionError("uuid.uuid4() called too many times")
        return uuid_module.UUID(uuid_values.popleft())

    web_task_manager.uuid.uuid4 = fake_uuid4
    FakeDateTime.values = deque(iso_to_datetime(value) for value in times)
    web_task_manager.datetime = FakeDateTime
    return original_uuid4, original_datetime


def restore_environment(original_uuid4, original_datetime) -> None:
    web_task_manager.uuid.uuid4 = original_uuid4
    web_task_manager.datetime = original_datetime


def make_manager() -> TaskManager:
    manager = TaskManager()
    manager.threading = FakeThreading
    return manager


def task_snapshot(task) -> dict[str, Any]:
    return {
        "id": task.id,
        "status": task.status,
        "progress": task.progress,
        "stage": task.stage,
        "config": task.config,
        "audio_file_path": task.audio_file_path,
        "created_at": task.created_at.isoformat(),
        "started_at": task.started_at.isoformat() if task.started_at else None,
        "completed_at": task.completed_at.isoformat() if task.completed_at else None,
        "error": task.error,
        "output_files": task.output_files,
        "thread": None if task.thread is None else {
            "name": task.thread.name,
            "daemon": task.thread.daemon,
            "started": task.thread.started,
        },
        "stop_event_set": task.stop_event.is_set(),
        "logs": task.logs,
    }


def assert_partial(case_id: str, actual: dict[str, Any], expected: dict[str, Any]) -> None:
    for key, expected_value in expected.items():
        if key == "thread_name":
            actual_value = actual["thread"]["name"]
        elif key == "thread_daemon":
            actual_value = actual["thread"]["daemon"]
        elif key == "thread_started":
            actual_value = actual["thread"]["started"]
        else:
            actual_value = actual[key]
        if actual_value != expected_value:
            raise AssertionError(f"{case_id}: {key} {actual_value!r} != {expected_value!r}")


def check_case(case: dict[str, Any]) -> None:
    case_id = case["case_id"]
    task_ids = case.get("task_ids", case.get("task_id", ""))
    original_uuid4, original_datetime = patch_environment(task_ids, case.get("times", []))
    try:
        manager = make_manager()
        expected = case["expect"]

        if "tasks" in case:
            for task_case in case["tasks"]:
                manager.create_task(task_case.get("config", {}), task_case["audio_path"])
            for expected_task_id in expected.get("get_task_ids", []):
                task = manager.get_task(expected_task_id)
                if not task or task.id != expected_task_id:
                    raise AssertionError(f"{case_id}: missing get_task hit {expected_task_id!r}")
            missing_lookup = expected.get("missing_lookup")
            if missing_lookup and manager.get_task(missing_lookup) is not None:
                raise AssertionError(f"{case_id}: get_task unexpectedly found {missing_lookup!r}")
            if "list" in expected:
                listed = manager.list_tasks()
                if listed != expected["list"]:
                    raise AssertionError(f"{case_id}: list {listed!r} != {expected['list']!r}")
            return

        task_id = case["task_id"]

        if case_id == "start_missing_task":
            assert manager.start_task(task_id, object()) is expected["start_result"]
            return
        if case_id == "stop_missing_task":
            assert manager.stop_task(task_id) is expected["stop_result"]
            return

        returned_id = manager.create_task(case.get("config", {}), case["audio_path"])
        if returned_id != task_id:
            raise AssertionError(f"{case_id}: task id {returned_id!r} != {task_id!r}")
        task = manager.get_task(task_id)

        if "initial_status" in case:
            task.status = case["initial_status"]

        if case.get("start_before_stop"):
            start_result = manager.start_task(task_id, object())
            if start_result is not True:
                raise AssertionError(f"{case_id}: setup start_task failed")

        if "start_result" in expected:
            actual = manager.start_task(task_id, object())
            if actual is not expected["start_result"]:
                raise AssertionError(f"{case_id}: start_result {actual!r}")

        if "stop_result" in expected:
            actual = manager.stop_task(task_id)
            if actual is not expected["stop_result"]:
                raise AssertionError(f"{case_id}: stop_result {actual!r}")

        snapshot = task_snapshot(manager.get_task(task_id))
        if "task" in expected:
            assert_partial(case_id, snapshot, expected["task"])
        if "list" in expected:
            listed = manager.list_tasks()
            if listed != expected["list"]:
                raise AssertionError(f"{case_id}: list {listed!r} != {expected['list']!r}")
    finally:
        restore_environment(original_uuid4, original_datetime)


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        try:
            case = json.loads(line)
        except json.JSONDecodeError as exc:
            raise AssertionError(f"fixture line {line_number} is invalid JSON: {exc}") from exc
        check_case(case)


if __name__ == "__main__":
    main()
