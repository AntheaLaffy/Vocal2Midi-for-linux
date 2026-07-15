"""Check download_models archive-layout fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
import tempfile
import zipfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "download_models_archive_layout_contract.jsonl"
)

sys.path.insert(0, str(PROJECT_ROOT))

from download_models import _validated_zip_member_path, extract_zip  # noqa: E402


def write_file(root: pathlib.Path, rel_path: str, content: str) -> None:
    path = root / pathlib.PurePosixPath(rel_path)
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")


def collect_files(root: pathlib.Path) -> list[dict[str, str]]:
    files: list[dict[str, str]] = []
    if not root.exists():
        return files
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        files.append(
            {
                "path": path.relative_to(root).as_posix(),
                "content": path.read_text(encoding="utf-8"),
            }
        )
    return files


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


def run_extract_zip(case: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="v2m_archive_fixture_") as tmp:
        root = pathlib.Path(tmp)
        archive = root / "fixture.zip"
        target = root / "target"

        for item in case.get("preexisting", []):
            write_file(target, item["path"], item["content"])

        with zipfile.ZipFile(archive, "w") as zf:
            for member in case["members"]:
                zf.writestr(member["name"], member["content"])

        try:
            extract_zip(archive, target)
        except ValueError as exc:
            actual = {
                "status": "error",
                "error": str(exc),
            }
        else:
            actual = {
                "status": "ok",
                "files": collect_files(target),
            }

    assert_subset(case["case_id"], actual, case["expect"])


def run_validate_member(case: dict[str, Any]) -> None:
    try:
        path = _validated_zip_member_path(case["member_name"])
    except ValueError as exc:
        actual = {
            "status": "error",
            "error": str(exc),
        }
    else:
        actual = {
            "status": "ok",
            "parts": list(path.parts),
        }
    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "extract_zip":
        run_extract_zip(case)
    elif operation == "validate_member":
        run_validate_member(case)
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
