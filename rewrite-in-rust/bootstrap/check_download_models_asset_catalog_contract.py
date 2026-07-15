"""Check download_models asset-catalog fixtures against legacy Python."""

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
    / "download_models_asset_catalog_contract.jsonl"
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


def make_entry(root: pathlib.Path, entry: dict[str, Any]) -> None:
    path = root / pathlib.PurePosixPath(entry["path"])
    if entry.get("kind") == "dir":
        path.mkdir(parents=True, exist_ok=True)
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(entry.get("content", ""), encoding="utf-8")


def make_model(root: pathlib.Path, data: dict[str, Any]) -> dm.GithubModel:
    name = data.get("name", "fixture")
    return dm.GithubModel(
        name=name,
        repo=data.get("repo", "fixture/repo"),
        tag=data.get("tag", "v0"),
        asset=data.get("asset", f"{name}.zip"),
        target=root / pathlib.PurePosixPath(data.get("target", f"models/{name}")),
        marker=data.get("marker", "model.onnx"),
        label=data.get("label", "fixture label"),
    )


def model_metadata(model: dm.GithubModel) -> dict[str, str]:
    return {
        "name": model.name,
        "repo": model.repo,
        "tag": model.tag,
        "asset": model.asset,
        "target": model.target.relative_to(dm.ROOT_DIR).as_posix(),
        "marker": model.marker,
        "label": model.label,
        "asset_url": dm.asset_url(model),
    }


def run_catalog_metadata(case: dict[str, Any]) -> None:
    actual = {
        "models": [model_metadata(model) for model in dm.GITHUB_MODELS],
        "lookup_keys": list(dm.GITHUB_MODEL_BY_NAME.keys()),
        "qwen": {
            "model_id": dm.QWEN_MODEL_ID,
            "local_dir": dm.QWEN_LOCAL_DIR.relative_to(dm.ROOT_DIR).as_posix(),
        },
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_human_size(case: dict[str, Any]) -> None:
    actual = {
        "results": [
            {"input": value, "display": dm.human_size(value)}
            for value in case["values"]
        ]
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_asset_url(case: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="v2m_catalog_url_") as tmp:
        model = make_model(pathlib.Path(tmp), case["model"])
        actual = {"url": dm.asset_url(model)}
    assert_subset(case["case_id"], actual, case["expect"])


def run_target_markers(case: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="v2m_catalog_markers_") as tmp:
        root = pathlib.Path(tmp)
        for entry in case.get("existing", []):
            make_entry(root, entry)

        actual = {
            "results": [
                {
                    "name": item["name"],
                    "present": dm.target_has_model(make_model(root, item)),
                }
                for item in case["models"]
            ]
        }

    assert_subset(case["case_id"], actual, case["expect"])


def run_qwen_has_weights(case: dict[str, Any]) -> None:
    actual_results: list[dict[str, Any]] = []
    for item in case["cases"]:
        with tempfile.TemporaryDirectory(prefix="v2m_catalog_qwen_") as tmp:
            dest = pathlib.Path(tmp) / "qwen"
            if item.get("dest_exists", False):
                dest.mkdir(parents=True, exist_ok=True)
                for entry in item.get("entries", []):
                    make_entry(dest, entry)
            actual_results.append(
                {"name": item["name"], "present": dm.qwen_has_weights(dest)}
            )

    assert_subset(case["case_id"], {"results": actual_results}, case["expect"])


def run_list_planned(case: dict[str, Any]) -> None:
    with tempfile.TemporaryDirectory(prefix="v2m_catalog_list_") as tmp:
        root = pathlib.Path(tmp)
        qwen_local_dir = root / "experiments" / "Qwen3-ASR-1.7B"

        for entry in case.get("existing", []):
            make_entry(root, entry)
        if case.get("qwen_dest_exists", False):
            qwen_local_dir.mkdir(parents=True, exist_ok=True)
            for entry in case.get("qwen_entries", []):
                make_entry(qwen_local_dir, entry)

        models = [make_model(root, item) for item in case["models"]]
        asset_sizes = {
            (item["repo"], item["tag"]): item["sizes"]
            for item in case.get("asset_sizes", [])
        }

        old_models = dm.GITHUB_MODELS
        old_lookup = dm.GITHUB_MODEL_BY_NAME
        old_root = dm.ROOT_DIR
        old_qwen_local_dir = dm.QWEN_LOCAL_DIR
        old_use_color = dm._USE_COLOR
        old_asset_sizes = dm.github_api_asset_sizes

        def fake_asset_sizes(repo: str, tag: str) -> dict[str, int]:
            return asset_sizes.get((repo, tag), {})

        try:
            dm.GITHUB_MODELS = models
            dm.GITHUB_MODEL_BY_NAME = {model.name: model for model in models}
            dm.ROOT_DIR = root
            dm.QWEN_LOCAL_DIR = qwen_local_dir
            dm._USE_COLOR = False
            dm.github_api_asset_sizes = fake_asset_sizes

            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                dm.list_planned(case["qwen_source"])
            actual = {"lines": stdout.getvalue().splitlines()}
        finally:
            dm.GITHUB_MODELS = old_models
            dm.GITHUB_MODEL_BY_NAME = old_lookup
            dm.ROOT_DIR = old_root
            dm.QWEN_LOCAL_DIR = old_qwen_local_dir
            dm._USE_COLOR = old_use_color
            dm.github_api_asset_sizes = old_asset_sizes

    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "catalog_metadata":
        run_catalog_metadata(case)
    elif operation == "human_size":
        run_human_size(case)
    elif operation == "asset_url":
        run_asset_url(case)
    elif operation == "target_markers":
        run_target_markers(case)
    elif operation == "qwen_has_weights":
        run_qwen_has_weights(case)
    elif operation == "list_planned":
        run_list_planned(case)
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
