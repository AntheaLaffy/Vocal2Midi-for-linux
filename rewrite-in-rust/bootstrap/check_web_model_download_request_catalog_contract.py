"""Check Web model download request/catalog fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
from datetime import datetime
from types import SimpleNamespace
from typing import Any, Callable

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "web_model_download_request_catalog_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import web_model_download_manager as model_module  # noqa: E402
import web_server  # noqa: E402
from web_model_download_manager import ModelDownloadTask  # noqa: E402


def parse_datetime(value: str | None) -> datetime | None:
    if value is None:
        return None
    return datetime.fromisoformat(value)


def make_task(data: dict[str, Any]) -> ModelDownloadTask:
    return ModelDownloadTask(
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


def read_response(response) -> dict[str, Any]:
    return json.loads(response.data.decode("utf-8"))


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


def patch_instance_attr(obj: Any, name: str, value: Any) -> Callable[[], None]:
    had_attr = name in obj.__dict__
    old_value = obj.__dict__.get(name)
    setattr(obj, name, value)

    def restore() -> None:
        if had_attr:
            setattr(obj, name, old_value)
        else:
            delattr(obj, name)

    return restore


def patch_install_state(installed: list[str]) -> Callable[[], None]:
    installed_set = set(installed)
    old_target_has_model = model_module.target_has_model
    old_qwen_has_weights = model_module.qwen_has_weights

    def fake_target_has_model(model) -> bool:
        return model.name in installed_set

    def fake_qwen_has_weights(_dest) -> bool:
        return "qwen" in installed_set

    model_module.target_has_model = fake_target_has_model
    model_module.qwen_has_weights = fake_qwen_has_weights

    def restore() -> None:
        model_module.target_has_model = old_target_has_model
        model_module.qwen_has_weights = old_qwen_has_weights

    return restore


def reset_manager_state() -> Callable[[], None]:
    manager = web_server.model_download_manager
    old_tasks = dict(manager.tasks)
    old_active_task_id = manager.active_task_id
    manager.tasks.clear()
    manager.active_task_id = None

    def restore() -> None:
        manager.tasks.clear()
        manager.tasks.update(old_tasks)
        manager.active_task_id = old_active_task_id

    return restore


def run_status_route(case: dict[str, Any]) -> None:
    manager = web_server.model_download_manager
    restore_install = patch_install_state(case.get("installed", []))
    restore_state = reset_manager_state()
    try:
        if case.get("active_task"):
            task = make_task(case["active_task"])
            manager.tasks[task.id] = task
            manager.active_task_id = task.id
        with web_server.app.test_client() as client:
            response = client.get("/api/models/status")
        actual = {
            "status_code": response.status_code,
            **read_response(response),
        }
        assert_expected(case, actual)
    finally:
        restore_state()
        restore_install()


def run_start_route(case: dict[str, Any]) -> None:
    manager = web_server.model_download_manager
    captured: dict[str, Any] | None = None
    restores: list[Callable[[], None]] = []
    try:
        if "model_statuses" in case:
            restores.append(
                patch_instance_attr(
                    manager,
                    "model_statuses",
                    lambda: case["model_statuses"],
                )
            )

        def fake_start_task(
            selected_models,
            qwen_source,
            force,
            proxy_mode,
            proxy_url,
            socketio_instance,
        ):
            nonlocal captured
            captured = {
                "selected_models": selected_models,
                "qwen_source": qwen_source,
                "force": force,
                "proxy_mode": proxy_mode,
                "proxy_url": proxy_url,
            }
            outcome = case.get("start_outcome", {"kind": "success"})
            if outcome["kind"] == "conflict":
                raise RuntimeError(outcome["error"])
            return SimpleNamespace(
                id=outcome.get("task_id", "fake-download-task"),
                status=outcome.get("status", "running"),
            )

        restores.append(patch_instance_attr(manager, "start_task", fake_start_task))

        with web_server.app.test_client() as client:
            if "request_raw" in case:
                response = client.post(
                    "/api/models/download",
                    data=case["request_raw"],
                    content_type="application/json",
                )
            else:
                response = client.post("/api/models/download", json=case["request"])
        actual = {
            "status_code": response.status_code,
            "json": read_response(response),
            "captured": captured,
        }
        assert_expected(case, actual)
    finally:
        for restore in reversed(restores):
            restore()


def run_status_lookup_route(case: dict[str, Any]) -> None:
    manager = web_server.model_download_manager
    task = make_task(case["task"]) if case.get("task") else None
    restores = [
        patch_instance_attr(manager, "get_task", lambda task_id: task),
    ]
    try:
        with web_server.app.test_client() as client:
            response = client.get(f"/api/models/download/status/{case['task_id']}")
        actual = {
            "status_code": response.status_code,
            "json": read_response(response),
        }
        assert_expected(case, actual)
    finally:
        for restore in reversed(restores):
            restore()


def run_stop_route(case: dict[str, Any]) -> None:
    manager = web_server.model_download_manager
    task = make_task(case["task"]) if case.get("task") else None
    restores = [
        patch_instance_attr(manager, "get_task", lambda task_id: task),
        patch_instance_attr(
            manager,
            "stop_task",
            lambda task_id: bool(case.get("stop_success", False)),
        ),
    ]
    try:
        with web_server.app.test_client() as client:
            response = client.post("/api/models/download/stop", json=case["request"])
        actual = {
            "status_code": response.status_code,
            "json": read_response(response),
        }
        assert_expected(case, actual)
    finally:
        for restore in reversed(restores):
            restore()


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "status_route":
        run_status_route(case)
    elif operation == "start_route":
        run_start_route(case)
    elif operation == "status_lookup_route":
        run_status_lookup_route(case)
    elif operation == "stop_route":
        run_stop_route(case)
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
