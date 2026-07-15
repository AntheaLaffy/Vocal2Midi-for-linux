"""Check Web output download security fixtures against legacy Python."""

from __future__ import annotations

import json
import os
import pathlib
import shutil
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_output_download_security.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import web_server  # noqa: E402
from web_task_manager import Task  # noqa: E402


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


def restore_case_placeholder(value: Any, case_dir: pathlib.Path) -> Any:
    if isinstance(value, str):
        return value.replace(str(case_dir), "__case__")
    if isinstance(value, list):
        return [restore_case_placeholder(item, case_dir) for item in value]
    if isinstance(value, dict):
        return {
            key: restore_case_placeholder(item, case_dir)
            for key, item in value.items()
        }
    return value


def assert_equal(case_id: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise AssertionError(f"{case_id}: {actual!r} != {expected!r}")


def configure_web_server(project_root: pathlib.Path):
    original_project_root = web_server.PROJECT_ROOT
    original_tasks = dict(web_server.task_manager.tasks)
    web_server.PROJECT_ROOT = project_root
    web_server.task_manager.tasks.clear()

    def restore() -> None:
        web_server.PROJECT_ROOT = original_project_root
        web_server.task_manager.tasks.clear()
        web_server.task_manager.tasks.update(original_tasks)

    return restore


def write_files(project_root: pathlib.Path, files: list[dict[str, str]]) -> None:
    for file in files:
        path = pathlib.Path(file["path"])
        if not path.is_absolute():
            path = project_root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(file["body"].encode("utf-8"))


def write_symlinks(project_root: pathlib.Path, symlinks: list[dict[str, str]]) -> None:
    for symlink in symlinks:
        link_path = pathlib.Path(symlink["path"])
        if not link_path.is_absolute():
            link_path = project_root / link_path
        target_path = pathlib.Path(symlink["target"])
        if not target_path.is_absolute():
            target_path = project_root / target_path
        link_path.parent.mkdir(parents=True, exist_ok=True)
        link_path.symlink_to(target_path)


def register_outputs(project_root: pathlib.Path, outputs: list[str]) -> None:
    task = Task(
        id="download-task",
        status="completed",
        progress=100,
        stage="done",
        config={},
        audio_file_path="audio.wav",
        created_at=web_server.datetime(2026, 7, 16, 12, 0, 0),
        output_files=[
            output if pathlib.Path(output).is_absolute() else str(pathlib.Path(output))
            for output in outputs
        ],
    )
    with web_server.task_manager._lock:
        web_server.task_manager.tasks[task.id] = task


def response_payload(response) -> dict[str, Any]:
    payload: dict[str, Any] = {"status_code": response.status_code}
    if response.status_code == 200:
        disposition = response.headers.get("Content-Disposition", "")
        download_name = disposition.rsplit("filename=", 1)[-1] if "filename=" in disposition else None
        payload.update(
            {
                "body": response.data.decode("utf-8"),
                "download_name": download_name,
            }
        )
    else:
        payload["json"] = json.loads(response.data.decode("utf-8"))
    return payload


def run_helper(case: dict[str, Any], case_dir: pathlib.Path) -> None:
    project_root = case_dir / "project"
    project_root.mkdir(parents=True, exist_ok=True)
    restore = configure_web_server(project_root)
    try:
        filepath = replace_case_placeholder(case["filepath"], case_dir)
        safe_path = web_server._safe_requested_download_path(filepath)
        authorized = web_server._authorized_output_file(filepath)
        actual = {
            "safe_path": str(safe_path) if safe_path else None,
            "authorized": bool(authorized),
        }
        assert_equal(
            case["case_id"],
            restore_case_placeholder(actual, case_dir),
            case["expect"],
        )
    finally:
        restore()


def run_route(case: dict[str, Any], case_dir: pathlib.Path) -> None:
    project_root = case_dir / "project"
    project_root.mkdir(parents=True, exist_ok=True)
    restore = configure_web_server(project_root)
    try:
        materialized = replace_case_placeholder(case, case_dir)
        write_files(project_root, materialized.get("files", []))
        write_symlinks(project_root, materialized.get("symlinks", []))
        register_outputs(project_root, materialized.get("registered_outputs", []))
        url = materialized.get("url") or f"/api/download/{materialized['filepath']}"
        with web_server.app.test_client() as client:
            response = client.get(url)
        actual = restore_case_placeholder(response_payload(response), case_dir)
        assert_equal(case["case_id"], actual, case["expect"])
    finally:
        restore()


def run_case(case: dict[str, Any], tmp_root: pathlib.Path) -> None:
    case_dir = tmp_root / case["case_id"]
    operation = case["operation"]
    if operation == "helper":
        run_helper(case, case_dir)
    elif operation in {"route", "route_raw"}:
        run_route(case, case_dir)
    else:
        raise AssertionError(f"unknown operation {operation!r}")


def main() -> None:
    tmp_root = pathlib.Path(tempfile.mkdtemp(prefix="v2m-web-download-"))
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
