"""Check batch CLI JSON re-slicing fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import pathlib
import sys
import tempfile
from typing import Any

import numpy as np

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "batch_cli_reslice_json_core.jsonl"

sys.path.insert(0, str(PROJECT_ROOT))

import scripts.slice_asr_cli as cli  # noqa: E402


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


class TextObject:
    def __init__(self, text: str) -> None:
        self.text = text


class TextNoneObject:
    text = None

    def __init__(self, repr_text: str) -> None:
        self.repr_text = repr_text

    def __str__(self) -> str:
        return self.repr_text


class ErrorTextObject:
    def __init__(self, error: str) -> None:
        self.error = error

    @property
    def text(self) -> str:
        raise RuntimeError(self.error)


def result_from_fixture(data: dict[str, Any]) -> Any:
    kind = data["kind"]
    if kind == "none":
        return None
    if kind == "object":
        return TextObject(data["text"])
    if kind == "object_text_none":
        return TextNoneObject(data["repr"])
    if kind == "object_text_error":
        return ErrorTextObject(data["error"])
    if kind == "dict":
        return data["value"]
    if kind == "scalar":
        return data["value"]
    raise AssertionError(f"unknown result kind {kind!r}")


def normalize_for_fixture(root: pathlib.Path, value: Any) -> Any:
    if isinstance(value, dict):
        return {key: normalize_for_fixture(root, item) for key, item in value.items()}
    if isinstance(value, list):
        return [normalize_for_fixture(root, item) for item in value]
    if isinstance(value, str):
        return value.replace(root.resolve().as_posix(), "__case__")
    return value


def rel(root: pathlib.Path, path: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()


def chunk_from_fixture(data: dict[str, Any]) -> dict[str, Any]:
    chunk: dict[str, Any] = {}
    if "offset" in data:
        chunk["offset"] = data["offset"]
    if "waveform" in data:
        chunk["waveform"] = np.asarray(data["waveform"], dtype=float)
    return chunk


def error_result(exc: Exception) -> dict[str, Any]:
    return {
        "status": "error",
        "error_type": type(exc).__name__,
        "error": str(exc),
    }


def run_extract_text_case(case: dict[str, Any]) -> dict[str, Any]:
    try:
        return {"status": "ok", "text": cli.extract_text(result_from_fixture(case["result"]))}
    except Exception as exc:
        return error_result(exc)


def run_save_timestamps_json(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    json_dir = root / pathlib.PurePosixPath(case["json_dir"])
    chunks = [chunk_from_fixture(chunk) for chunk in case["chunks"]]
    results = [result_from_fixture(result) for result in case["results"]]
    source_audio = case.get("source_audio")
    source_path = (
        root / pathlib.PurePosixPath(source_audio)
        if source_audio is not None
        else None
    )

    stdout = io.StringIO()
    try:
        with contextlib.redirect_stdout(stdout):
            json_path = cli.save_timestamps_json(
                json_dir=json_dir,
                source_stem=case["source_stem"],
                chunks=chunks,
                results=results,
                chunk_indices=case["chunk_indices"],
                sr=int(case["sr"]),
                source_audio=source_path,
                source_md5=case.get("source_md5"),
            )
    except Exception as exc:
        return error_result(exc)

    return normalize_for_fixture(
        root,
        {
            "status": "ok",
            "json_path": rel(root, json_path),
            "stdout_lines": stdout.getvalue().splitlines(),
            "payload_json": json_path.read_text(encoding="utf-8"),
        },
    )


def write_json_fixture(root: pathlib.Path, case: dict[str, Any]) -> pathlib.Path:
    json_path = root / pathlib.PurePosixPath(case["json_path"])
    if case.get("json_exists", True):
        json_path.parent.mkdir(parents=True, exist_ok=True)
        if "raw_json" in case:
            json_path.write_text(case["raw_json"], encoding="utf-8")
        else:
            json_path.write_text(
                json.dumps(case.get("payload"), ensure_ascii=False),
                encoding="utf-8",
            )
    return json_path


def run_slice_audio_from_json(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    json_path = write_json_fixture(root, case)
    source_audio = root / pathlib.PurePosixPath(case["source_audio"])
    if case.get("source_exists", True):
        source_audio.parent.mkdir(parents=True, exist_ok=True)
        source_audio.write_bytes(b"audio")
    output_dir = root / pathlib.PurePosixPath(case["output_dir"])

    writes: list[dict[str, Any]] = []
    loaded_audio = False
    old_load_audio = cli.load_audio
    old_write = cli.sf.write

    def fake_load_audio(path: pathlib.Path, sr: int = cli.DEFAULT_SAMPLE_RATE) -> tuple[Any, int]:
        nonlocal loaded_audio
        loaded_audio = True
        return (
            np.asarray(case.get("waveform", []), dtype=float),
            int(case.get("actual_sr", sr)),
        )

    def fake_write(path: pathlib.Path, data: Any, sr: int) -> None:
        array = np.asarray(data, dtype=float)
        writes.append(
            {
                "path": rel(root, pathlib.Path(path)),
                "sr": int(sr),
                "len": int(len(array)),
                "data": [float(value) for value in array.tolist()],
            }
        )

    stdout = io.StringIO()
    try:
        cli.load_audio = fake_load_audio
        cli.sf.write = fake_write
        with contextlib.redirect_stdout(stdout):
            written = cli.slice_audio_from_json(json_path, source_audio, output_dir)
    except Exception as exc:
        return normalize_for_fixture(root, error_result(exc))
    finally:
        cli.load_audio = old_load_audio
        cli.sf.write = old_write

    labs = [
        {"path": rel(root, path), "content": path.read_text(encoding="utf-8")}
        for path in sorted(output_dir.rglob("*.lab"))
    ]
    return normalize_for_fixture(
        root,
        {
            "status": "ok",
            "written": written,
            "loaded_audio": loaded_audio,
            "output_dir_exists": output_dir.exists(),
            "stdout_lines": stdout.getvalue().splitlines(),
            "writes": writes,
            "labs": labs,
        },
    )


def run_save_chunks(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    chunk_dir = root / pathlib.PurePosixPath(case["chunk_dir"])
    chunks = [chunk_from_fixture(chunk) for chunk in case["chunks"]]
    writes: list[dict[str, Any]] = []
    old_write = cli.sf.write

    def fake_write(path: pathlib.Path, data: Any, sr: int) -> None:
        array = np.asarray(data, dtype=float)
        writes.append(
            {
                "path": rel(root, pathlib.Path(path)),
                "sr": int(sr),
                "len": int(len(array)),
                "data": [float(value) for value in array.tolist()],
            }
        )

    try:
        cli.sf.write = fake_write
        saved_paths = cli.save_chunks(
            chunk_dir,
            case["source_stem"],
            chunks,
            int(case["sr"]),
        )
    except Exception as exc:
        return error_result(exc)
    finally:
        cli.sf.write = old_write

    return {
        "status": "ok",
        "created_dir": chunk_dir.is_dir(),
        "saved_paths": [rel(root, path) for path in saved_paths],
        "writes": writes,
    }


def run_grouped(
    case: dict[str, Any],
    runner: Any,
    root: pathlib.Path | None = None,
) -> None:
    for index, subcase in enumerate(case["cases"]):
        actual = runner(root, subcase) if root is not None else runner(subcase)
        assert_subset(f"{case['case_id']}[{index}]", actual, subcase["expect"])


def run_case(case: dict[str, Any]) -> dict[str, Any] | None:
    operation = case["operation"]
    if operation == "extract_text":
        run_grouped(case, run_extract_text_case)
        return None

    with tempfile.TemporaryDirectory(prefix="v2m_reslice_json_") as tmp:
        root = pathlib.Path(tmp)
        if operation == "save_timestamps_json":
            return run_save_timestamps_json(root, case)
        if operation == "save_timestamps_json_cases":
            run_grouped(case, run_save_timestamps_json, root)
            return None
        if operation == "slice_audio_from_json":
            return run_slice_audio_from_json(root, case)
        if operation == "slice_audio_from_json_cases":
            run_grouped(case, run_slice_audio_from_json, root)
            return None
        if operation == "save_chunks_cases":
            run_grouped(case, run_save_chunks, root)
            return None

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
        actual = run_case(case)
        if actual is not None:
            assert_subset(case["case_id"], actual, case["expect"])


if __name__ == "__main__":
    main()
