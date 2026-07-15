"""Check Web stream redirector fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "web_stream_redirector_contract.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

from web_stream_redirector import WebStreamRedirector  # noqa: E402


class FakeStream:
    encoding = "utf-8"

    def __init__(self):
        self.writes: list[str] = []
        self.flush_count = 0

    def write(self, text: str) -> None:
        self.writes.append(text)

    def flush(self) -> None:
        self.flush_count += 1


def make_callback(mode: str, calls: list[list[str]]):
    if mode == "none":
        return None

    def callback(message: str, level: str = "info") -> None:
        calls.append([message, level])
        if mode == "raise":
            raise RuntimeError("callback failed")

    if mode == "record" or mode == "raise":
        return callback
    raise AssertionError(f"unknown callback mode {mode!r}")


def check_case(case: dict[str, Any]) -> None:
    case_id = case["case_id"]
    stream = FakeStream()
    callbacks: list[list[str]] = []
    redirector = WebStreamRedirector(stream, make_callback(case["callback"], callbacks))

    operation = case["operation"]
    if operation == "write":
        redirector.write(case["text"])
    elif operation == "flush":
        redirector.flush()
    elif operation == "getattr":
        actual = getattr(redirector, case["attribute"])
        expected = case["expect"]["attribute_value"]
        if actual != expected:
            raise AssertionError(f"{case_id}: attribute {actual!r} != {expected!r}")
    else:
        raise AssertionError(f"unknown operation {operation!r}")

    expected = case["expect"]
    if stream.writes != expected["stream_writes"]:
        raise AssertionError(f"{case_id}: writes {stream.writes!r} != {expected['stream_writes']!r}")
    if callbacks != expected["callbacks"]:
        raise AssertionError(f"{case_id}: callbacks {callbacks!r} != {expected['callbacks']!r}")
    if stream.flush_count != expected["flush_count"]:
        raise AssertionError(
            f"{case_id}: flush_count {stream.flush_count!r} != {expected['flush_count']!r}"
        )


def main() -> None:
    for line_number, line in enumerate(FIXTURE_PATH.read_text().splitlines(), start=1):
        if not line or line.startswith("#"):
            continue
        try:
            case = json.loads(line)
        except json.JSONDecodeError as exc:
            raise AssertionError(f"fixture line {line_number} is invalid JSON: {exc}") from exc
        check_case(case)


if __name__ == "__main__":
    main()
