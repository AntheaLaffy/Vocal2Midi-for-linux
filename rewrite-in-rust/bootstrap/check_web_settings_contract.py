"""Check Web settings fixtures against legacy Python."""

from __future__ import annotations

import copy
import json
import pathlib
import shutil
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_settings_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import web_server  # noqa: E402


RESPONSE_EXPECTATION_KEYS = {
    "status_code",
    "success",
    "message",
    "error_contains",
    "saved_file",
    "saved_trailing_newline",
    "raw_contains",
    "json_semantic",
}


def expected_settings_subset(expected: dict[str, Any]) -> dict[str, Any]:
    return {
        key: value
        for key, value in expected.items()
        if key not in RESPONSE_EXPECTATION_KEYS
    }


def assert_subset(case_id: str, actual: Any, expected: Any, path: str = "") -> None:
    if isinstance(expected, dict):
        if not isinstance(actual, dict):
            raise AssertionError(f"{case_id}: {path} actual is not object")
        for key, expected_value in expected.items():
            if key == "unknown_top_present":
                if ("unknown_top" in actual) != expected_value:
                    raise AssertionError(f"{case_id}: unknown_top presence mismatch")
                continue
            if key not in actual:
                raise AssertionError(f"{case_id}: missing key {path}.{key}")
            assert_subset(case_id, actual[key], expected_value, f"{path}.{key}")
        return
    if actual != expected:
        raise AssertionError(f"{case_id}: {path} {actual!r} != {expected!r}")


def read_response(response) -> dict[str, Any]:
    return json.loads(response.data.decode("utf-8"))


def reset_web_server_state(settings_file: pathlib.Path) -> None:
    web_server.SETTINGS_FILE = settings_file
    web_server.current_settings = copy.deepcopy(web_server.DEFAULT_SETTINGS)


def run_merge(case: dict[str, Any]) -> None:
    actual = web_server._merge_settings(web_server.DEFAULT_SETTINGS, case["overrides"])
    assert_subset(case["case_id"], actual, case["expect"])


def run_load(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    settings_file = tmp_dir / f"{case['case_id']}.json"
    reset_web_server_state(settings_file)
    state = case["file_state"]
    if state == "malformed":
        settings_file.write_text(case["file_text"], encoding="utf-8")
    elif state == "json":
        settings_file.write_text(
            json.dumps(case["file_json"], ensure_ascii=False),
            encoding="utf-8",
        )
    elif state != "missing":
        raise AssertionError(f"unknown file_state {state!r}")
    actual = web_server._load_settings_from_disk()
    assert_subset(case["case_id"], actual, case["expect"])


def run_update(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    settings_file = tmp_dir / f"{case['case_id']}.json"
    reset_web_server_state(settings_file)
    with web_server.app.test_client() as client:
        response = client.put("/api/settings", json=case["request_json"])
    actual = read_response(response)
    expected = case["expect"]
    if response.status_code != expected["status_code"]:
        raise AssertionError(
            f"{case['case_id']}: status {response.status_code} != {expected['status_code']}"
        )
    if actual.get("success") != expected["success"]:
        raise AssertionError(f"{case['case_id']}: success mismatch")
    if "message" in expected and actual.get("message") != expected["message"]:
        raise AssertionError(f"{case['case_id']}: message mismatch")
    if "error_contains" in expected and expected["error_contains"] not in actual.get("error", ""):
        raise AssertionError(f"{case['case_id']}: error {actual.get('error')!r}")
    if actual.get("success"):
        assert_subset(case["case_id"], actual["settings"], expected_settings_subset(expected))
    saved = settings_file.is_file()
    if saved != expected.get("saved_file", False):
        raise AssertionError(f"{case['case_id']}: saved_file {saved}")
    if saved and expected.get("saved_trailing_newline") and not settings_file.read_text(encoding="utf-8").endswith("\n"):
        raise AssertionError(f"{case['case_id']}: saved payload lacks trailing newline")


def run_update_raw(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    settings_file = tmp_dir / f"{case['case_id']}.json"
    reset_web_server_state(settings_file)
    with web_server.app.test_client() as client:
        response = client.put(
            "/api/settings",
            data=case["request_raw"],
            content_type="application/json",
        )
    actual = read_response(response)
    expected = case["expect"]
    if response.status_code != expected["status_code"]:
        raise AssertionError(f"{case['case_id']}: status mismatch")
    if actual.get("success") != expected["success"]:
        raise AssertionError(f"{case['case_id']}: success mismatch")
    if "message" in expected and actual.get("message") != expected["message"]:
        raise AssertionError(f"{case['case_id']}: message mismatch")
    if "error_contains" in expected and expected["error_contains"] not in actual.get("error", ""):
        raise AssertionError(f"{case['case_id']}: error {actual.get('error')!r}")
    if actual.get("success"):
        assert_subset(case["case_id"], actual["settings"], expected_settings_subset(expected))
    if settings_file.is_file() != expected.get("saved_file", False):
        raise AssertionError(f"{case['case_id']}: saved_file mismatch")
    if (
        settings_file.is_file()
        and expected.get("saved_trailing_newline")
        and not settings_file.read_text(encoding="utf-8").endswith("\n")
    ):
        raise AssertionError(f"{case['case_id']}: saved payload lacks trailing newline")


def run_reset(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    settings_file = tmp_dir / f"{case['case_id']}.json"
    reset_web_server_state(settings_file)
    web_server.current_settings = web_server._merge_settings(
        web_server.current_settings,
        case["pre_update"],
    )
    with web_server.app.test_client() as client:
        response = client.post("/api/settings/reset")
    actual = read_response(response)
    expected = case["expect"]
    if response.status_code != expected["status_code"]:
        raise AssertionError(f"{case['case_id']}: status mismatch")
    if actual.get("success") != expected["success"]:
        raise AssertionError(f"{case['case_id']}: success mismatch")
    if actual.get("message") != expected["message"]:
        raise AssertionError(f"{case['case_id']}: message mismatch")
    assert_subset(case["case_id"], actual["settings"], expected_settings_subset(expected))
    if settings_file.is_file() != expected.get("saved_file", False):
        raise AssertionError(f"{case['case_id']}: saved_file mismatch")


def run_save(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    settings_file = tmp_dir / f"{case['case_id']}.json"
    reset_web_server_state(settings_file)
    web_server._save_settings_to_disk(case["settings"])
    raw = settings_file.read_text(encoding="utf-8")
    expected = case["expect"]
    if raw.endswith("\n") != expected["saved_trailing_newline"]:
        raise AssertionError(f"{case['case_id']}: trailing newline mismatch")
    for needle in expected.get("raw_contains", []):
        if needle not in raw:
            raise AssertionError(f"{case['case_id']}: raw payload lacks {needle!r}")
    if expected.get("json_semantic") and json.loads(raw) != case["settings"]:
        raise AssertionError(f"{case['case_id']}: JSON semantic mismatch")


def run_case(case: dict[str, Any], tmp_dir: pathlib.Path) -> None:
    operation = case["operation"]
    if operation == "merge":
        run_merge(case)
    elif operation == "load":
        run_load(case, tmp_dir)
    elif operation == "update":
        run_update(case, tmp_dir)
    elif operation == "update_raw":
        run_update_raw(case, tmp_dir)
    elif operation == "reset":
        run_reset(case, tmp_dir)
    elif operation == "save":
        run_save(case, tmp_dir)
    else:
        raise AssertionError(f"unknown operation {operation!r}")


def main() -> None:
    tmp_dir = pathlib.Path(tempfile.mkdtemp(prefix="v2m-web-settings-"))
    original_settings_file = web_server.SETTINGS_FILE
    original_current_settings = web_server.current_settings
    try:
        for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
            if not line or line.startswith("#"):
                continue
            try:
                case = json.loads(line)
            except json.JSONDecodeError as exc:
                raise AssertionError(f"fixture line {line_number} is invalid JSON: {exc}") from exc
            run_case(case, tmp_dir)
    finally:
        web_server.SETTINGS_FILE = original_settings_file
        web_server.current_settings = original_current_settings
        shutil.rmtree(tmp_dir, ignore_errors=True)


if __name__ == "__main__":
    main()
