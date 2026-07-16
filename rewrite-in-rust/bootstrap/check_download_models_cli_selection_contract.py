"""Check download_models CLI-selection fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import pathlib
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "download_models_cli_selection_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

import download_models as dm  # noqa: E402


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


def parse_result(ns: Any) -> dict[str, Any]:
    return {
        "status": "ok",
        "only": ns.only,
        "force": ns.force,
        "qwen_source": ns.qwen_source,
        "no_qwen": ns.no_qwen,
        "list": ns.list,
    }


def run_parse_args(case: dict[str, Any]) -> None:
    stderr = io.StringIO()
    try:
        with contextlib.redirect_stderr(stderr):
            ns = dm.parse_args(case["argv"])
    except SystemExit as exc:
        err = stderr.getvalue()
        fragments = case.get("stderr_must_contain", [])
        missing = [fragment for fragment in fragments if fragment not in err]
        if missing:
            raise AssertionError(
                f"{case['case_id']}: stderr missing fragments {missing!r}: {err!r}"
            )
        actual = {
            "status": "error",
            "exit_code": exc.code,
            "stderr_contains": fragments,
        }
    else:
        actual = parse_result(ns)
    assert_subset(case["case_id"], actual, case["expect"])


def run_main_plan(case: dict[str, Any]) -> None:
    github_outcomes = case.get("github_outcomes", {})
    qwen_outcome = case.get("qwen_outcome", True)
    github_calls: list[dict[str, Any]] = []
    qwen_calls: list[dict[str, Any]] = []
    list_calls: list[str] = []

    old_experiments_dir = dm.EXPERIMENTS_DIR
    old_download_github_model = dm.download_github_model
    old_download_qwen = dm.download_qwen
    old_list_planned = dm.list_planned
    old_use_color = dm._USE_COLOR

    with tempfile.TemporaryDirectory(prefix="v2m_cli_selection_") as tmp:
        experiments_dir = pathlib.Path(tmp) / "experiments"

        def fake_download_github_model(model: dm.GithubModel, force: bool) -> bool:
            github_calls.append({"name": model.name, "force": force})
            return github_outcomes.get(model.name, True)

        def fake_download_qwen(source: str, force: bool) -> bool:
            qwen_calls.append({"source": source, "force": force})
            return qwen_outcome

        def fake_list_planned(qwen_source: str) -> None:
            list_calls.append(qwen_source)
            print(f"LIST {qwen_source}")

        stdout = io.StringIO()
        stderr = io.StringIO()
        try:
            dm.EXPERIMENTS_DIR = experiments_dir
            dm.download_github_model = fake_download_github_model
            dm.download_qwen = fake_download_qwen
            dm.list_planned = fake_list_planned
            dm._USE_COLOR = False

            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                exit_code = dm.main(case["argv"])
            mkdir_called = experiments_dir.exists()
        finally:
            dm.EXPERIMENTS_DIR = old_experiments_dir
            dm.download_github_model = old_download_github_model
            dm.download_qwen = old_download_qwen
            dm.list_planned = old_list_planned
            dm._USE_COLOR = old_use_color

    actual = {
        "exit_code": exit_code,
        "mkdir_called": mkdir_called,
        "github_calls": github_calls,
        "qwen_calls": qwen_calls,
        "list_calls": list_calls,
        "stdout_lines": stdout.getvalue().splitlines(),
        "stderr_lines": stderr.getvalue().splitlines(),
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "parse_args":
        run_parse_args(case)
    elif operation == "main_plan":
        run_main_plan(case)
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
