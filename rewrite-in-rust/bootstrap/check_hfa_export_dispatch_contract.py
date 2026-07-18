"""Check HubertFA export dispatch fixtures against legacy Python."""

from __future__ import annotations

import builtins
import json
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "hfa_export_dispatch_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.HubertFA.tools import export_tool  # noqa: E402
from inference.HubertFA.tools.infer_base import InferenceBase  # noqa: E402


class RecordingExporter(export_tool.Exporter):
    def __init__(self, predictions: object, output_folder: object = None, events: list[dict[str, Any]] | None = None, errors: dict[str, dict[str, str]] | None = None) -> None:
        self.events = events if events is not None else []
        self.errors = errors or {}
        super().__init__(predictions, output_folder=output_folder)
        self.events.append(
            {
                "event": "init",
                "predictions": repr(self.predictions),
                "output_folder": None if self.output_folder is None else self.output_folder.as_posix(),
            }
        )

    def save_textgrids(self) -> None:
        self.events.append({"event": "textgrid"})
        if "textgrid" in self.errors:
            error = self.errors["textgrid"]
            raise ERROR_TYPES[error["type"]](error["message"])

    def save_htk(self) -> None:
        self.events.append({"event": "htk"})
        if "htk" in self.errors:
            error = self.errors["htk"]
            raise ERROR_TYPES[error["type"]](error["message"])


ERROR_TYPES = {
    "RuntimeError": RuntimeError,
    "ValueError": ValueError,
    "TypeError": TypeError,
}


def decode_format(value: Any) -> Any:
    if isinstance(value, dict):
        kind = value.get("$kind")
        if kind == "tuple":
            return tuple(value.get("items", []))
        if kind == "mapping":
            return dict(value.get("items", []))
        if kind == "none":
            return None
    return value


def project_result(callable_obj: Any) -> dict[str, Any]:
    try:
        result = callable_obj()
    except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
        return {
            "error": {
                "type": type(error).__name__,
                "message": str(error),
            }
        }
    return {"ok": None if result is None else repr(result)}


def run_exporter_case(case: dict[str, Any]) -> dict[str, Any]:
    events: list[dict[str, Any]] = []
    exporter = RecordingExporter(
        case.get("predictions", "PRED"),
        output_folder=case.get("output_folder"),
        events=events,
        errors=case.get("errors"),
    )
    out_formats = decode_format(case.get("out_formats"))
    result = project_result(lambda: exporter.export(out_formats))
    result["events"] = events
    return result


def run_inference_case(case: dict[str, Any]) -> dict[str, Any]:
    events: list[dict[str, Any]] = []
    prints: list[str] = []
    original_exporter = export_tool.Exporter
    original_infer_base_exporter = InferenceBase.export.__globals__["Exporter"]
    original_print = builtins.print

    class InjectedExporter(RecordingExporter):
        def __init__(self, predictions: object, output_folder: object = None) -> None:
            super().__init__(
                predictions,
                output_folder=output_folder,
                events=events,
                errors=case.get("errors"),
            )

    def fake_print(*args: object, sep: str = " ", end: str = "\n", file: object | None = None, flush: bool = False) -> None:
        if file is not None:
            original_print(*args, sep=sep, end=end, file=file, flush=flush)
            return
        prints.append(sep.join(str(arg) for arg in args) + end)

    export_tool.Exporter = InjectedExporter
    InferenceBase.export.__globals__["Exporter"] = InjectedExporter
    builtins.print = fake_print
    try:
        inference = InferenceBase.__new__(InferenceBase)
        inference.predictions = case.get("predictions", "PRED")
        output_format = decode_format(case.get("output_format"))
        result = project_result(lambda: inference.export(case.get("output_folder"), output_format=output_format))
    finally:
        export_tool.Exporter = original_exporter
        InferenceBase.export.__globals__["Exporter"] = original_infer_base_exporter
        builtins.print = original_print

    result["events"] = events
    result["prints"] = prints
    return result


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    repeat = case.get("repeat", 1)
    kind = case["kind"]
    runner = run_exporter_case if kind == "exporter" else run_inference_case
    return {"calls": [runner(case) for _ in range(repeat)]}


def assert_equal(case_id: str, actual: Any, expected: Any) -> None:
    if actual != expected:
        raise AssertionError(
            f"{case_id}:\n"
            f"actual={json.dumps(actual, ensure_ascii=False, sort_keys=True)}\n"
            f"expect={json.dumps(expected, ensure_ascii=False, sort_keys=True)}"
        )


def main() -> None:
    if sys.version_info[:2] != (3, 12):
        raise AssertionError(f"fixtures require Python 3.12, got {sys.version.split()[0]}")
    cases = [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]
    for case in cases:
        assert_equal(case["case_id"], run_case(case), case["expect"])
    print(f"validated {len(cases)} hfa_export_dispatch_contract fixtures")


if __name__ == "__main__":
    main()
