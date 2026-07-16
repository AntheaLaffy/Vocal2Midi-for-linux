"""Check batch CLI planning/index fixtures against legacy Python."""

from __future__ import annotations

import json
import pathlib
import sys
import tempfile
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = REWRITE_ROOT / "fixtures" / "batch_cli_planning_and_index_core.jsonl"

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


def make_entry(root: pathlib.Path, entry: dict[str, Any]) -> None:
    path = root / pathlib.PurePosixPath(entry["path"])
    if entry.get("kind") == "dir":
        path.mkdir(parents=True, exist_ok=True)
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(entry.get("content", "").encode("utf-8"))


def normalize_for_fixture(root: pathlib.Path, value: Any) -> Any:
    if isinstance(value, dict):
        return {
            key: normalize_for_fixture(root, item)
            for key, item in value.items()
        }
    if isinstance(value, list):
        return [normalize_for_fixture(root, item) for item in value]
    if isinstance(value, str):
        return value.replace(root.resolve().as_posix(), "__case__")
    return value


def relative_to_root(root: pathlib.Path, path: pathlib.Path) -> str:
    return path.relative_to(root).as_posix()


def relative_to_input(input_dir: pathlib.Path, path: pathlib.Path) -> str:
    return path.relative_to(input_dir).as_posix()


def run_batch_iter(case: dict[str, Any]) -> dict[str, Any]:
    try:
        batches = [
            [str(item) for item in batch]
            for batch in cli.batch_iter(
                [pathlib.Path(item) for item in case["items"]],
                int(case["batch_size"]),
            )
        ]
    except ValueError as exc:
        return {"status": "error", "error": str(exc)}
    return {"status": "ok", "batches": batches}


def run_normalize_method(case: dict[str, Any]) -> dict[str, Any]:
    try:
        method = cli.normalize_slicing_method(case.get("method"))
    except ValueError as exc:
        return {"status": "error", "error": str(exc)}
    return {"status": "ok", "method": method}


def run_slice_bounds(case: dict[str, Any]) -> dict[str, Any]:
    try:
        bounds = cli.resolve_slice_bounds(
            case.get("min_seconds"),
            case.get("max_seconds"),
        )
    except ValueError as exc:
        return {"status": "error", "error": str(exc)}
    return {"status": "ok", "bounds": list(bounds) if bounds is not None else None}


def run_scan(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    for entry in case.get("entries", []):
        make_entry(root, entry)
    input_dir = root / pathlib.PurePosixPath(case["input_dir"])
    files = cli.collect_audio_files(input_dir, recursive=case.get("recursive", True))
    return {"files": [relative_to_input(input_dir, path) for path in files]}


def run_source_identity(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    audio_path = root / pathlib.PurePosixPath(case["audio_path"])
    audio_path.parent.mkdir(parents=True, exist_ok=True)
    audio_path.write_bytes(case.get("content", "").encode("utf-8"))
    provided_md5 = case.get("provided_md5")
    return {
        "safe_stem": cli.safe_stem(audio_path),
        "md5": cli.file_md5(audio_path),
        "source_key": cli.source_key(audio_path, provided_md5),
    }


def run_source_index(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    output_dir = root / "output"
    index_path = cli.source_index_path(output_dir)
    if "initial_content" in case:
        index_path.parent.mkdir(parents=True, exist_ok=True)
        index_path.write_text(case["initial_content"], encoding="utf-8")
    elif "initial_index" in case:
        index_path.parent.mkdir(parents=True, exist_ok=True)
        index_path.write_text(
            json.dumps(case["initial_index"], ensure_ascii=False, indent=2),
            encoding="utf-8",
        )

    loaded = cli.load_source_index(output_dir)
    actual: dict[str, Any] = {
        "source_index_path": cli.source_index_path(output_dir)
        .relative_to(output_dir)
        .as_posix(),
        "loaded": normalize_for_fixture(root, loaded),
    }

    if update := case.get("update"):
        audio_path = root / pathlib.PurePosixPath(update["audio_path"])
        audio_path.parent.mkdir(parents=True, exist_ok=True)
        audio_path.write_bytes(b"")
        cli.update_source_index(
            loaded,
            audio_path,
            update["output_key"],
            update["source_md5"],
            update["chunks"],
            update["labs"],
        )
        cli.save_source_index(output_dir, loaded)
        actual["updated"] = normalize_for_fixture(root, loaded)
        actual["saved_json"] = normalize_for_fixture(
            root,
            index_path.read_text(encoding="utf-8"),
        )

    return actual


def run_existing_outputs(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    audio_path = root / pathlib.PurePosixPath(case["audio_path"])
    audio_path.parent.mkdir(parents=True, exist_ok=True)
    audio_path.write_bytes(b"")
    for entry in case.get("entries", []):
        make_entry(root, entry)

    output_dir = root / "output"
    source_index = case.get("source_index", {})
    source_md5 = case["source_md5"]
    actual: dict[str, Any] = {
        "existing": cli.has_existing_outputs(
            audio_path,
            output_dir,
            source_md5,
            recursive_output=case.get("recursive_output", True),
        )
    }
    try:
        actual["indexed_key"] = cli.index_has_completed_output(
            source_index,
            source_md5,
            output_dir,
        )
        actual["indexed_status"] = "ok"
    except Exception as exc:
        actual["indexed_status"] = "error"
        actual["indexed_error"] = str(exc)
    return actual


def run_batch_plan(root: pathlib.Path, case: dict[str, Any]) -> dict[str, Any]:
    for entry in case.get("source_entries", []):
        make_entry(root, entry)
    for entry in case.get("output_entries", []):
        make_entry(root, entry)

    input_dir = root / pathlib.PurePosixPath(case.get("input_dir", "input"))
    output_dir = root / pathlib.PurePosixPath(case.get("output_dir", "output"))
    source_index = json.loads(json.dumps(case.get("source_index", {})))
    md5_errors = set(case.get("md5_errors", []))
    process_outcomes = case.get("process_outcomes", {})

    audio_files = cli.collect_audio_files(input_dir, recursive=case.get("recursive", True))
    batches = [
        [relative_to_input(input_dir, path) for path in batch]
        for batch in cli.batch_iter(audio_files, int(case["file_batch_size"]))
    ]

    total_chunks = 0
    total_labs = 0
    skipped_existing = 0
    skipped_failed = 0
    processed: list[dict[str, Any]] = []
    skipped: list[dict[str, Any]] = []

    for file_batch in cli.batch_iter(audio_files, int(case["file_batch_size"])):
        for audio_path in file_batch:
            rel_path = relative_to_root(root, audio_path)
            if rel_path in md5_errors:
                skipped_failed += 1
                skipped.append({"path": rel_path, "reason": "md5_failed"})
                continue

            source_md5 = cli.file_md5(audio_path)
            output_key = cli.source_key(audio_path, source_md5)
            if not case.get("no_skip_existing", False):
                indexed_key = cli.index_has_completed_output(
                    source_index,
                    source_md5,
                    output_dir,
                )
                if indexed_key is not None:
                    skipped_existing += 1
                    skipped.append(
                        {
                            "path": rel_path,
                            "reason": "indexed",
                            "output_key": indexed_key,
                        }
                    )
                    continue
                if cli.has_existing_outputs(audio_path, output_dir, source_md5):
                    skipped_existing += 1
                    skipped.append(
                        {
                            "path": rel_path,
                            "reason": "existing",
                            "output_key": output_key,
                        }
                    )
                    continue

            outcome = process_outcomes.get(rel_path, {"chunks": 0, "labs": 0})
            if outcome.get("error") is not None:
                skipped_failed += 1
                skipped.append({"path": rel_path, "reason": "process_failed"})
                continue

            chunks = int(outcome.get("chunks", 0))
            labs = int(outcome.get("labs", 0))
            cli.update_source_index(
                source_index,
                audio_path,
                output_key,
                source_md5,
                chunks,
                labs,
            )
            cli.save_source_index(output_dir, source_index)
            total_chunks += chunks
            total_labs += labs
            processed.append(
                {
                    "path": rel_path,
                    "output_key": output_key,
                    "chunks": chunks,
                    "labs": labs,
                }
            )

    return normalize_for_fixture(
        root,
        {
            "audio_files": [relative_to_input(input_dir, path) for path in audio_files],
            "batches": batches,
            "total_chunks": total_chunks,
            "total_labs": total_labs,
            "skipped_existing": skipped_existing,
            "skipped_failed": skipped_failed,
            "processed": processed,
            "skipped": skipped,
            "source_index": source_index,
        },
    )


def run_case(case: dict[str, Any]) -> dict[str, Any]:
    operation = case["operation"]
    if operation == "batch_iter":
        return run_batch_iter(case)
    if operation == "normalize_method":
        return run_normalize_method(case)
    if operation == "slice_bounds":
        return run_slice_bounds(case)

    with tempfile.TemporaryDirectory(prefix="v2m_batch_cli_") as tmp:
        root = pathlib.Path(tmp)
        if operation == "scan":
            return run_scan(root, case)
        if operation == "source_identity":
            return run_source_identity(root, case)
        if operation == "source_index":
            return run_source_index(root, case)
        if operation == "existing_outputs":
            return run_existing_outputs(root, case)
        if operation == "batch_plan":
            return run_batch_plan(root, case)

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
        assert_subset(case["case_id"], actual, case["expect"])


if __name__ == "__main__":
    main()
