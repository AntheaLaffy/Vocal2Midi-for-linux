"""Check Web filesystem picker fixtures against legacy Python."""

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
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_filesystem_picker_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import web_server  # noqa: E402


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


def assert_subset(case_id: str, actual: Any, expected: Any, path: str = "") -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {path} actual is not object")
        for key, expected_value in expected.items():
            if key == "error_contains":
                actual_error = actual.get("error", "")
                if expected_value not in actual_error:
                    raise AssertionError(f"{case_id}: error {actual_error!r}")
                continue
            if key not in actual:
                raise AssertionError(f"{case_id}: missing key {path}.{key}")
            assert_subset(case_id, actual[key], expected_value, f"{path}.{key}")
        return
    if isinstance(expected, list):
        if actual != expected:
            raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")


def assert_exact(case_id: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise AssertionError(f"{case_id}: exact payload {actual!r} != {expected!r}")


def setup_case_dirs(case_dir: pathlib.Path) -> tuple[pathlib.Path, pathlib.Path]:
    project_root = case_dir / "project"
    home_dir = case_dir / "home"
    project_root.mkdir(parents=True, exist_ok=True)
    home_dir.mkdir(parents=True, exist_ok=True)
    return project_root, home_dir


def setup_children(base_dir: pathlib.Path, children: list[dict[str, str]]) -> None:
    base_dir.mkdir(parents=True, exist_ok=True)
    for child in children:
        path = base_dir / child["name"]
        if child["type"] == "directory":
            path.mkdir(parents=True, exist_ok=True)
        elif child["type"] == "file":
            path.write_text("fixture", encoding="utf-8")
        else:
            raise AssertionError(f"unknown child type {child['type']!r}")


class FakeDirEntry:
    def __init__(self, base_dir: pathlib.Path, child: dict[str, str]):
        self.name = child["name"]
        self.path = str(base_dir / child["name"])
        self._type = child["type"]

    def is_dir(self, *, follow_symlinks: bool = True) -> bool:
        return self._type == "directory"

    def is_file(self, *, follow_symlinks: bool = True) -> bool:
        return self._type == "file"


class FakeScandir:
    def __init__(self, entries: list[FakeDirEntry]):
        self._entries = entries

    def __enter__(self):
        return iter(self._entries)

    def __exit__(self, exc_type, exc, traceback):
        return False


def configure_web_server(project_root: pathlib.Path, home_dir: pathlib.Path):
    original_project_root = web_server.PROJECT_ROOT
    original_home = os.environ.get("HOME")
    web_server.PROJECT_ROOT = project_root
    os.environ["HOME"] = str(home_dir)

    def restore() -> None:
        web_server.PROJECT_ROOT = original_project_root
        if original_home is None:
            os.environ.pop("HOME", None)
        else:
            os.environ["HOME"] = original_home

    return restore


def run_resolve(case: dict[str, Any], case_dir: pathlib.Path) -> None:
    project_root, home_dir = setup_case_dirs(case_dir)
    restore = configure_web_server(project_root, home_dir)
    try:
        path_text = replace_case_placeholder(case["path_text"], case_dir)
        resolved = web_server._resolve_picker_path(path_text)
        actual = {
            "resolved": str(resolved),
            "input_path": web_server._input_value_for_path(resolved),
        }
        assert_subset(
            case["case_id"],
            restore_case_placeholder(actual, case_dir),
            case["expect"],
        )
    finally:
        restore()


def run_extensions(case: dict[str, Any]) -> None:
    actual = {"extensions": sorted(web_server._parse_extensions(case["raw"]))}
    assert_subset(case["case_id"], actual, case["expect"])


def run_roots(case: dict[str, Any], case_dir: pathlib.Path) -> None:
    project_root, home_dir = setup_case_dirs(case_dir)
    restore = configure_web_server(project_root, home_dir)
    try:
        actual = {
            "separator": os.sep,
            "roots": web_server._filesystem_roots(),
        }
        assert_subset(
            case["case_id"],
            restore_case_placeholder(actual, case_dir),
            case["expect"],
        )
    finally:
        restore()


def read_response(response) -> dict[str, Any]:
    data = json.loads(response.data.decode("utf-8"))
    data["status_code"] = response.status_code
    return data


def run_list(case: dict[str, Any], case_dir: pathlib.Path) -> None:
    project_root, home_dir = setup_case_dirs(case_dir)
    restore = configure_web_server(project_root, home_dir)
    original_scandir = web_server.os.scandir
    try:
        materialized = replace_case_placeholder(case, case_dir)
        requested_path = web_server._resolve_picker_path(materialized["path_text"])
        if case["path_state"] == "directory":
            list_dir = requested_path
        elif case["path_state"] == "file":
            requested_path.parent.mkdir(parents=True, exist_ok=True)
            requested_path.write_text("fixture", encoding="utf-8")
            list_dir = requested_path.parent
        elif case["path_state"] == "missing":
            list_dir = requested_path
        else:
            raise AssertionError(f"unknown path_state {case['path_state']!r}")

        if case["path_state"] != "missing":
            setup_children(list_dir, materialized.get("children", []))

        error_path = list_dir.resolve()
        fake_entries = [
            FakeDirEntry(list_dir, child)
            for child in materialized.get("children", [])
        ]

        def fake_scandir(path):
            if pathlib.Path(path).resolve() == error_path:
                if materialized.get("scandir_error"):
                    raise OSError(materialized["scandir_error"])
                return FakeScandir(fake_entries)
            return original_scandir(path)

        web_server.os.scandir = fake_scandir

        with web_server.app.test_client() as client:
            response = client.get(
                "/api/filesystem/list",
                query_string={
                    "path": materialized["path_text"],
                    "mode": materialized["mode"],
                    "extensions": materialized.get("extensions", ""),
                },
            )
        actual = restore_case_placeholder(read_response(response), case_dir)
        expected = case["expect"]
        if expected.get("success") is False:
            if "error_contains" in expected:
                exact_expected = dict(expected)
                error_contains = exact_expected.pop("error_contains")
                if error_contains not in actual.get("error", ""):
                    raise AssertionError(f"{case['case_id']}: error {actual.get('error')!r}")
                exact_expected["error"] = actual.get("error")
            else:
                exact_expected = expected
            assert_exact(case["case_id"], actual, exact_expected)
        else:
            assert_subset(case["case_id"], actual, expected)
    finally:
        web_server.os.scandir = original_scandir
        restore()


def run_case(case: dict[str, Any], tmp_root: pathlib.Path) -> None:
    case_dir = tmp_root / case["case_id"]
    operation = case["operation"]
    if operation == "resolve":
        run_resolve(case, case_dir)
    elif operation == "extensions":
        run_extensions(case)
    elif operation == "roots":
        run_roots(case, case_dir)
    elif operation == "list":
        run_list(case, case_dir)
    else:
        raise AssertionError(f"unknown operation {operation!r}")


def main() -> None:
    tmp_root = pathlib.Path(tempfile.mkdtemp(prefix="v2m-web-picker-"))
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
