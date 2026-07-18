"""Check HubertFA HTK label export fixtures against legacy Python."""

from __future__ import annotations

import builtins
import json
import math
import pathlib
import sys
from dataclasses import dataclass
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "hfa_htk_label_export_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from inference.HubertFA.tools.export_tool import Exporter  # noqa: E402


@dataclass
class FakePhoneme:
    start: Any
    end: Any
    text: str


@dataclass
class FakeWord:
    start: Any
    end: Any
    text: str
    phonemes: list[FakePhoneme]


class FakeWriter:
    def __init__(self, path: pathlib.Path, encoding: str | None, writes: list[dict[str, Any]]) -> None:
        self.path = path
        self.encoding = encoding
        self.writes = writes
        self.content = ""

    def __enter__(self) -> FakeWriter:
        return self

    def __exit__(self, exc_type: object, exc: object, traceback: object) -> None:
        if exc_type is None:
            self.writes.append(
                {
                    "path": self.path.as_posix(),
                    "encoding": self.encoding,
                    "content": self.content,
                }
            )

    def write(self, text: str) -> int:
        self.content += text
        return len(text)


def decode_float(value: Any) -> Any:
    if not isinstance(value, dict) or set(value) != {"$float"}:
        return value
    return {
        "nan": float("nan"),
        "+inf": float("inf"),
        "-inf": float("-inf"),
        "-0.0": -0.0,
    }[value["$float"]]


def decode_word(data: dict[str, Any]) -> FakeWord:
    return FakeWord(
        start=decode_float(data["start"]),
        end=decode_float(data["end"]),
        text=data["text"],
        phonemes=[
            FakePhoneme(
                start=decode_float(phoneme["start"]),
                end=decode_float(phoneme["end"]),
                text=phoneme["text"],
            )
            for phoneme in data.get("phonemes", [])
        ],
    )


def decode_predictions(case: dict[str, Any]) -> list[tuple[str, Any, list[FakeWord]]]:
    return [
        (
            prediction["wav_path"],
            decode_float(prediction.get("wav_length", 0.0)),
            [decode_word(word) for word in prediction.get("words", [])],
        )
        for prediction in case["predictions"]
    ]


def plan_with_legacy(case: dict[str, Any]) -> dict[str, Any]:
    mkdirs: list[dict[str, Any]] = []
    writes: list[dict[str, Any]] = []

    original_mkdir = pathlib.Path.mkdir
    original_open = builtins.open

    def fake_mkdir(self: pathlib.Path, mode: int = 0o777, parents: bool = False, exist_ok: bool = False) -> None:
        mkdirs.append(
            {
                "path": self.as_posix(),
                "parents": parents,
                "exist_ok": exist_ok,
            }
        )

    def fake_open(file: object, mode: str = "r", *args: Any, **kwargs: Any) -> FakeWriter:
        if mode != "w":
            return original_open(file, mode, *args, **kwargs)
        return FakeWriter(pathlib.Path(file), kwargs.get("encoding"), writes)

    pathlib.Path.mkdir = fake_mkdir
    builtins.open = fake_open
    try:
        output_folder = case.get("output_folder")
        if output_folder == "":
            output_folder = ""
        exporter = Exporter(decode_predictions(case), output_folder=output_folder)
        try:
            exporter.save_htk()
        except Exception as error:  # noqa: BLE001 - exact legacy surface is fixture data.
            return {
                "error": {
                    "type": type(error).__name__,
                    "message": str(error),
                },
                "partial_plan": {"directories": mkdirs, "files": writes},
            }
        return {"ok": {"directories": mkdirs, "files": writes}}
    finally:
        pathlib.Path.mkdir = original_mkdir
        builtins.open = original_open


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    repeat = case.get("repeat", 1)
    return {"calls": [plan_with_legacy(case) for _ in range(repeat)]}


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
    if not math.isclose(1.0, 1.0):
        raise AssertionError("math sanity check failed")

    cases = [
        json.loads(line)
        for line in FIXTURE_PATH.read_text(encoding="utf-8").splitlines()
        if line and not line.startswith("#")
    ]
    for case in cases:
        assert_equal(case["case_id"], run_case(case), case["expect"])
    print(f"validated {len(cases)} hfa_htk_label_export_core fixtures")


if __name__ == "__main__":
    main()
