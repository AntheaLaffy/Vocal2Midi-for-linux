"""Check PyYAML safe_load fixtures for the HubertFA config loader contract."""

from __future__ import annotations

import base64
import datetime as dt
import errno
import json
import math
import sys
import tempfile
from pathlib import Path
from typing import Any


REPO_ROOT = Path(__file__).resolve().parents[2]
FIXTURE_PATH = REPO_ROOT / "rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl"
sys.path.insert(0, str(REPO_ROOT))

from inference.HubertFA.tools import config_utils  # noqa: E402
import yaml  # noqa: E402


class Projector:
    """Project Python values into tagged JSON without losing PyYAML semantics."""

    def __init__(self) -> None:
        self.ids: dict[int, int] = {}
        self.next_id = 1

    def _object_id(self, value: Any) -> tuple[int, bool]:
        object_id = id(value)
        if object_id in self.ids:
            return self.ids[object_id], False

        label = self.next_id
        self.next_id += 1
        self.ids[object_id] = label
        return label, True

    def project(self, value: Any) -> dict[str, Any]:
        if value is None:
            return {"tag": "null"}
        if isinstance(value, bool):
            return {"tag": "bool", "value": value}
        if isinstance(value, int) and not isinstance(value, bool):
            return {"tag": "int", "value": str(value)}
        if isinstance(value, float):
            sign = -1 if math.copysign(1.0, value) < 0 else 1
            if math.isnan(value):
                return {"tag": "float", "kind": "nan", "repr": repr(value), "hex": value.hex(), "sign": sign}
            if math.isinf(value):
                return {"tag": "float", "kind": "inf", "repr": repr(value), "hex": value.hex(), "sign": sign}
            return {"tag": "float", "kind": "finite", "repr": repr(value), "hex": value.hex(), "sign": sign}
        if isinstance(value, str):
            return {"tag": "str", "value": value}
        if isinstance(value, bytes):
            return {
                "tag": "bytes",
                "hex": value.hex(),
                "base64": base64.b64encode(value).decode("ascii"),
            }
        if isinstance(value, dt.datetime):
            offset = value.utcoffset()
            return {
                "tag": "datetime",
                "iso": value.isoformat(),
                "tz_offset_seconds": None if offset is None else int(offset.total_seconds()),
            }
        if isinstance(value, dt.date):
            return {"tag": "date", "iso": value.isoformat()}
        if isinstance(value, list):
            label, is_first = self._object_id(value)
            if not is_first:
                return {"tag": "ref", "id": label}
            return {
                "tag": "list",
                "id": label,
                "items": [self.project(item) for item in value],
            }
        if isinstance(value, tuple):
            return {
                "tag": "tuple",
                "items": [self.project(item) for item in value],
            }
        if isinstance(value, set):
            label, is_first = self._object_id(value)
            if not is_first:
                return {"tag": "ref", "id": label}
            items = [self.project(item) for item in value]
            items.sort(key=lambda item: json.dumps(item, ensure_ascii=False, sort_keys=True))
            return {"tag": "set", "id": label, "items": items}
        if isinstance(value, dict):
            label, is_first = self._object_id(value)
            if not is_first:
                return {"tag": "ref", "id": label}
            return {
                "tag": "dict",
                "id": label,
                "entries": [
                    {"key": self.project(key), "value": self.project(item)}
                    for key, item in value.items()
                ],
            }

        raise TypeError(f"unsupported PyYAML projection type: {type(value)!r}")


def load_cases() -> list[dict[str, Any]]:
    return [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]


def normalize_text(text: object, temp_root: Path) -> str:
    return str(text).replace(str(temp_root), "<TMP>")


def project_mark(mark: object, temp_root: Path) -> dict[str, object] | None:
    if mark is None:
        return None

    snippet = mark.get_snippet() if hasattr(mark, "get_snippet") else None
    return {
        "name": normalize_text(getattr(mark, "name"), temp_root),
        "index": getattr(mark, "index"),
        "line": getattr(mark, "line"),
        "column": getattr(mark, "column"),
        "line_1": getattr(mark, "line") + 1,
        "column_1": getattr(mark, "column") + 1,
        "pointer": getattr(mark, "pointer", None),
        "snippet": snippet,
    }


def error_phase(error: BaseException) -> str:
    if isinstance(error, OSError):
        return "file_open"
    if isinstance(error, UnicodeDecodeError):
        return "utf8_decode"

    module = type(error).__module__
    if module == "yaml.scanner":
        return "scanner"
    if module == "yaml.parser":
        return "parser"
    if module == "yaml.composer":
        return "composer"
    if module == "yaml.constructor":
        return "constructor"
    return "python_value"


def project_error(error: BaseException, temp_root: Path) -> dict[str, Any]:
    projected: dict[str, Any] = {
        "ok": False,
        "error": {
            "phase": error_phase(error),
            "class": f"{type(error).__module__}.{type(error).__name__}",
            "message": normalize_text(error, temp_root),
            "context": getattr(error, "context", None),
            "problem": getattr(error, "problem", None),
            "note": getattr(error, "note", None),
            "context_mark": project_mark(getattr(error, "context_mark", None), temp_root),
            "problem_mark": project_mark(getattr(error, "problem_mark", None), temp_root),
        },
    }

    if isinstance(error, OSError):
        projected["error"].update(
            {
                "errno": error.errno,
                "strerror": error.strerror,
                "filename": None if error.filename is None else normalize_text(error.filename, temp_root),
                "filename2": None if error.filename2 is None else normalize_text(error.filename2, temp_root),
            }
        )
    if isinstance(error, UnicodeDecodeError):
        projected["error"].update(
            {
                "encoding": error.encoding,
                "reason": error.reason,
                "start": error.start,
                "end": error.end,
                "object_len": len(error.object),
                "object_hex": error.object[:32].hex(),
            }
        )

    return projected


def prepare_case_path(case: dict[str, Any], temp_root: Path) -> Path:
    path = temp_root / "case.yaml"
    kind = case.get("kind", "file")
    if kind == "file":
        path.write_text(case["content"], encoding="utf-8")
    elif kind == "bytes":
        path.write_bytes(bytes.fromhex(case["bytes_hex"]))
    elif kind == "directory":
        path.mkdir()
    elif kind == "missing":
        pass
    else:
        raise AssertionError(f"unsupported fixture kind: {kind!r}")
    return path


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    with tempfile.TemporaryDirectory() as directory:
        temp_root = Path(directory)
        path = prepare_case_path(case, temp_root)
        calls = []
        for _ in range(case.get("repeat", 1)):
            try:
                value = config_utils.load_yaml(path)
                calls.append({"ok": True, "value": Projector().project(value)})
            except BaseException as error:  # noqa: BLE001 - legacy errors are fixture data.
                calls.append(project_error(error, temp_root))
        return {"calls": calls}


def main() -> None:
    if sys.version_info[:2] != (3, 12):
        raise AssertionError(f"fixtures require Python 3.12, got {sys.version.split()[0]}")
    if yaml.__version__ != "6.0.3":
        raise AssertionError(f"fixtures require PyYAML 6.0.3, got {yaml.__version__}")
    if errno.ENOENT != 2:
        raise AssertionError("fixtures assume POSIX-style errno constants")

    cases = load_cases()
    for case in cases:
        actual = run_case(case)
        if actual != case["expect"]:
            raise AssertionError(
                f"{case['case_id']}:\n"
                f"actual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}\n"
                f"expect={json.dumps(case['expect'], ensure_ascii=False, sort_keys=True)}"
            )

    print(f"validated {len(cases)} hfa_pyyaml_safe_load_contract fixtures")


if __name__ == "__main__":
    main()
