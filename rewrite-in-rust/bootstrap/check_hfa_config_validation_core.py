"""Check injected-loader HubertFA config validation fixtures against legacy Python."""

from __future__ import annotations

import builtins
import json
import sys
import tempfile
from copy import deepcopy
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = REPO_ROOT / "rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl"
sys.path.insert(0, str(REPO_ROOT))

from inference.HubertFA.tools import config_utils  # noqa: E402


def load_cases() -> list[dict[str, Any]]:
    return [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]


def replace_temp(value: Any, temp_root: Path, replacement: str) -> Any:
    if isinstance(value, str):
        return value.replace(replacement, str(temp_root))
    if isinstance(value, list):
        return [replace_temp(item, temp_root, replacement) for item in value]
    if isinstance(value, dict):
        return {
            key: replace_temp(item, temp_root, replacement)
            for key, item in value.items()
        }
    return value


def normalize_error(error: BaseException, temp_root: Path) -> dict[str, str]:
    return {
        "type": type(error).__name__,
        "message": str(error).replace(str(temp_root), "<TMP>"),
    }


def create_files(temp_root: Path, files: list[str | dict[str, Any]]) -> None:
    for file_spec in files:
        if isinstance(file_spec, str):
            relative_path = file_spec
            kind = "file"
            content = ""
        else:
            relative_path = file_spec["path"]
            kind = file_spec.get("kind", "file")
            content = file_spec.get("content", "")

        rendered_path = relative_path.replace("<TMP>", str(temp_root))
        path = Path(rendered_path)
        if not path.is_absolute():
            path = temp_root / path
        path.parent.mkdir(parents=True, exist_ok=True)
        if kind == "file":
            path.write_text(content, encoding="utf-8")
        elif kind == "directory":
            path.mkdir()
        else:
            raise AssertionError(f"unsupported file kind: {kind!r}")


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    with tempfile.TemporaryDirectory() as directory:
        temp_root = Path(directory)
        create_files(temp_root, case.get("files", []))
        loader_paths: list[str] = []
        loader_spec = case["loader"]

        def injected_loader(path: str | Path) -> Any:
            loader_paths.append(str(path).replace(str(temp_root), "<TMP>"))
            if "error" in loader_spec:
                error_spec = loader_spec["error"]
                error_type = getattr(builtins, error_spec["type"])
                raise error_type(error_spec["message"])
            return replace_temp(deepcopy(loader_spec["value"]), temp_root, "<TMP>")

        original_loader = config_utils.load_yaml
        config_utils.load_yaml = injected_loader
        try:
            calls = []
            for _ in range(case.get("repeat", 1)):
                try:
                    value = config_utils.check_configs(
                        temp_root,
                        suffix=case.get("suffix", "yaml"),
                    )
                    calls.append({"value": value})
                except BaseException as error:  # noqa: BLE001 - legacy errors are fixture data.
                    calls.append({"error": normalize_error(error, temp_root)})
        finally:
            config_utils.load_yaml = original_loader

        return {"calls": calls, "loader_paths": loader_paths}


def main() -> None:
    if sys.version_info[:2] != (3, 12):
        raise AssertionError(f"fixtures require Python 3.12, got {sys.version.split()[0]}")

    cases = load_cases()
    for case in cases:
        actual = run_case(case)
        if actual != case["expect"]:
            raise AssertionError(
                f"{case['case_id']}:\n"
                f"actual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}\n"
                f"expect={json.dumps(case['expect'], ensure_ascii=False, sort_keys=True)}"
            )

    print(f"validated {len(cases)} hfa_config_validation_core fixtures")


if __name__ == "__main__":
    main()
