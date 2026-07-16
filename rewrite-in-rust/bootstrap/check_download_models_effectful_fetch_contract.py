"""Check download_models effectful-fetch fixtures against legacy Python."""

from __future__ import annotations

import contextlib
import io
import json
import pathlib
import sys
import tempfile
import urllib.error
from typing import Any

REWRITE_ROOT = pathlib.Path(__file__).resolve().parents[1]
PROJECT_ROOT = REWRITE_ROOT.parent
FIXTURE_PATH = (
    REWRITE_ROOT
    / "fixtures"
    / "download_models_effectful_fetch_contract.jsonl"
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


def collect_files(root: pathlib.Path) -> list[str]:
    if not root.exists():
        return []
    return sorted(path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file())


def make_model(root: pathlib.Path, data: dict[str, Any]) -> dm.GithubModel:
    return dm.GithubModel(
        name=data["name"],
        repo=data["repo"],
        tag=data["tag"],
        asset=data["asset"],
        target=root / pathlib.PurePosixPath(data["target"]),
        marker=data["marker"],
        label=data["label"],
    )


class FakeResponse:
    def __init__(
        self,
        body: bytes = b"",
        chunks: list[bytes] | None = None,
        headers: dict[str, str] | None = None,
    ) -> None:
        self.body = body
        self.chunks = list(chunks or [])
        self.headers = headers or {}

    def __enter__(self) -> "FakeResponse":
        return self

    def __exit__(self, *args: object) -> None:
        return None

    def read(self, size: int | None = None) -> bytes:
        if self.chunks:
            return self.chunks.pop(0)
        if size is not None:
            body = self.body[:size]
            self.body = self.body[size:]
            return body
        body = self.body
        self.body = b""
        return body


def request_url(req: Any) -> str:
    return getattr(req, "full_url", str(req))


def run_github_api_asset_sizes(case: dict[str, Any]) -> None:
    calls: list[Any] = []

    def fake_urlopen(req: Any, timeout: int) -> FakeResponse:
        calls.append(req)
        if case["mode"] == "url_error":
            raise urllib.error.URLError("fixture")
        if case["mode"] == "invalid_json":
            return FakeResponse(body=b"{not json")
        return FakeResponse(body=json.dumps(case["payload"]).encode("utf-8"))

    old_cache = dm._ASSET_SIZE_CACHE
    old_urlopen = dm.urllib.request.urlopen
    try:
        dm._ASSET_SIZE_CACHE = {}
        dm.urllib.request.urlopen = fake_urlopen
        first = dm.github_api_asset_sizes(case["repo"], case["tag"])
        actual: dict[str, Any] = {
            "first": first,
            "urlopen_calls": len(calls),
            "request_url": request_url(calls[0]) if calls else None,
        }
        if case.get("repeat"):
            actual["second"] = dm.github_api_asset_sizes(case["repo"], case["tag"])
            actual["urlopen_calls"] = len(calls)
    finally:
        dm._ASSET_SIZE_CACHE = old_cache
        dm.urllib.request.urlopen = old_urlopen

    assert_subset(case["case_id"], actual, case["expect"])


def run_stream_download(case: dict[str, Any]) -> None:
    calls: list[Any] = []
    chunks = [chunk.encode("utf-8") for chunk in case["chunks"]]
    headers = {}
    if case.get("content_length") is not None:
        headers["Content-Length"] = str(case["content_length"])

    def fake_urlopen(req: Any, timeout: int) -> FakeResponse:
        calls.append(req)
        return FakeResponse(chunks=chunks, headers=headers)

    old_urlopen = dm.urllib.request.urlopen
    try:
        dm.urllib.request.urlopen = fake_urlopen
        with tempfile.TemporaryDirectory(prefix="v2m_effect_stream_") as tmp:
            dest = pathlib.Path(tmp) / "download.bin"
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                downloaded = dm.stream_download(case["url"], dest)
            req = calls[0]
            actual = {
                "downloaded": downloaded,
                "file_content": dest.read_text(encoding="utf-8"),
                "stdout": stdout.getvalue(),
                "request_url": request_url(req),
                "user_agent": req.get_header("User-agent"),
            }
    finally:
        dm.urllib.request.urlopen = old_urlopen

    assert_subset(case["case_id"], actual, case["expect"])


def run_download_github_model(case: dict[str, Any]) -> None:
    stream_calls: list[str] = []
    extract_calls: list[dict[str, Any]] = []
    target_present_sequence = list(case.get("target_present_sequence", []))

    with tempfile.TemporaryDirectory(prefix="v2m_effect_github_") as tmp:
        root = pathlib.Path(tmp)
        experiments_dir = root / "experiments"
        experiments_dir.mkdir(parents=True, exist_ok=True)
        model = make_model(root, case["model"])

        def fake_target_has_model(_model: dm.GithubModel) -> bool:
            if not target_present_sequence:
                return False
            return bool(target_present_sequence.pop(0))

        def fake_asset_sizes(repo: str, tag: str) -> dict[str, int]:
            return {model.asset: int(case.get("expected_size", 0))}

        def fake_stream_download(url: str, dest: pathlib.Path) -> int:
            stream_calls.append(url)
            result = case.get("stream_result", {})
            if "http_error" in result:
                raise urllib.error.HTTPError(url, result["http_error"], "fixture", None, None)
            if "url_error" in result:
                raise urllib.error.URLError(result["url_error"])
            if "timeout_error" in result:
                raise TimeoutError(result["timeout_error"])
            return int(result.get("bytes", 0))

        def fake_extract_zip(zip_path: pathlib.Path, target: pathlib.Path) -> None:
            extract_calls.append({"target": target.relative_to(root).as_posix()})
            result = case.get("extract_result", "ok")
            if result == "bad_zip":
                raise dm.zipfile.BadZipFile("fixture")
            if result == "unsafe_layout":
                raise ValueError("Unsafe zip member path: '../evil'")

        old_root = dm.ROOT_DIR
        old_experiments = dm.EXPERIMENTS_DIR
        old_target_has_model = dm.target_has_model
        old_asset_sizes = dm.github_api_asset_sizes
        old_stream_download = dm.stream_download
        old_extract_zip = dm.extract_zip
        old_use_color = dm._USE_COLOR
        try:
            dm.ROOT_DIR = root
            dm.EXPERIMENTS_DIR = experiments_dir
            dm.target_has_model = fake_target_has_model
            dm.github_api_asset_sizes = fake_asset_sizes
            dm.stream_download = fake_stream_download
            dm.extract_zip = fake_extract_zip
            dm._USE_COLOR = False
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                try:
                    result = dm.download_github_model(model, case["force"])
                except Exception as exc:  # noqa: BLE001 - fixture records legacy escape shape.
                    actual = {
                        "status": "error",
                        "error_type": type(exc).__name__,
                        "error": str(exc),
                        "stream_calls": stream_calls,
                        "extract_calls": extract_calls,
                        "tmp_zip_leftovers": sorted(
                            path.name for path in experiments_dir.glob("v2m_dl_*.zip")
                        ),
                        "stdout_lines": stdout.getvalue().splitlines(),
                        "stderr_lines": stderr.getvalue().splitlines(),
                    }
                else:
                    actual = {
                        "return": result,
                        "stream_calls": stream_calls,
                        "extract_calls": extract_calls,
                        "tmp_zip_leftovers": sorted(
                            path.name for path in experiments_dir.glob("v2m_dl_*.zip")
                        ),
                        "stdout_lines": stdout.getvalue().splitlines(),
                        "stderr_lines": stderr.getvalue().splitlines(),
                    }
        finally:
            dm.ROOT_DIR = old_root
            dm.EXPERIMENTS_DIR = old_experiments
            dm.target_has_model = old_target_has_model
            dm.github_api_asset_sizes = old_asset_sizes
            dm.stream_download = old_stream_download
            dm.extract_zip = old_extract_zip
            dm._USE_COLOR = old_use_color

    assert_subset(case["case_id"], actual, case["expect"])


def run_run_cli(case: dict[str, Any]) -> None:
    old_run = dm.subprocess.run
    results: list[dict[str, Any]] = []

    class Completed:
        def __init__(self, returncode: int) -> None:
            self.returncode = returncode

    try:
        for item in case["cases"]:
            def fake_run(args: list[str], check: bool) -> Completed:
                if item["mode"] == "missing":
                    raise FileNotFoundError(args[0])
                return Completed(item["returncode"])

            dm.subprocess.run = fake_run
            results.append({"name": item["name"], "return": dm._run_cli(item["args"])})
    finally:
        dm.subprocess.run = old_run

    assert_subset(case["case_id"], {"results": results}, case["expect"])


def run_pip_install(case: dict[str, Any]) -> None:
    old_have_uv = dm._have_uv
    old_run_cli = dm._run_cli
    old_executable = dm.sys.executable
    results: list[dict[str, Any]] = []
    try:
        dm.sys.executable = "__python__"
        for item in case["cases"]:
            run_args: list[str] | None = None

            def fake_have_uv() -> bool:
                return bool(item["have_uv"])

            def fake_run_cli(args: list[str]) -> int:
                nonlocal run_args
                run_args = list(args)
                return int(item["run_return"])

            dm._have_uv = fake_have_uv
            dm._run_cli = fake_run_cli
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                result = dm._pip_install(item["pkgs"])
            results.append(
                {
                    "name": item["name"],
                    "return": result,
                    "run_args": run_args,
                    "stdout_lines": stdout.getvalue().splitlines(),
                }
            )
    finally:
        dm._have_uv = old_have_uv
        dm._run_cli = old_run_cli
        dm.sys.executable = old_executable

    assert_subset(case["case_id"], {"results": results}, case["expect"])


def run_resolve_cli(case: dict[str, Any]) -> None:
    old_venv_bin = dm._venv_bin
    old_which = dm.shutil.which
    results: list[dict[str, Any]] = []
    try:
        for item in case["cases"]:
            with tempfile.TemporaryDirectory(prefix="v2m_effect_resolve_") as tmp:
                venv = pathlib.Path(tmp) / "venv"
                venv.mkdir(parents=True, exist_ok=True)
                venv_cli = venv / item["cli"]
                if item["venv_exists"]:
                    venv_cli.write_text("", encoding="utf-8")

                def fake_venv_bin(name: str) -> pathlib.Path:
                    return venv / name

                def fake_which(name: str) -> str | None:
                    return item.get("which")

                dm._venv_bin = fake_venv_bin
                dm.shutil.which = fake_which
                resolved = dm._resolve_cli(item["cli"])
                if resolved == str(venv_cli):
                    resolved = f"__venv__/{item['cli']}"
                results.append({"name": item["name"], "resolved": resolved})
    finally:
        dm._venv_bin = old_venv_bin
        dm.shutil.which = old_which

    assert_subset(case["case_id"], {"results": results}, case["expect"])


def run_cleanup_qwen_artifacts(case: dict[str, Any]) -> None:
    if any(entry.get("unlink_error") for entry in case["entries"]):
        files = {
            entry["path"]: bool(entry.get("unlink_error"))
            for entry in case["entries"]
            if entry.get("kind") == "file"
        }

        class FakePath:
            def __init__(self, path: str = "") -> None:
                self.path = path

            @property
            def name(self) -> str:
                return pathlib.PurePosixPath(self.path).name

            def __truediv__(self, child: str) -> "FakePath":
                if not self.path:
                    return FakePath(child)
                return FakePath(f"{self.path}/{child}")

            def exists(self) -> bool:
                return self.path in files

            def unlink(self) -> None:
                if files[self.path]:
                    raise OSError("fixture unlink failure")
                del files[self.path]

            def glob(self, pattern: str) -> list["FakePath"]:
                if pattern != "*.incomplete":
                    return []
                return [
                    FakePath(path)
                    for path in sorted(files)
                    if "/" not in path and path.endswith(".incomplete")
                ]

        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            dm._cleanup_qwen_artifacts(FakePath())
        actual = {
            "remaining": sorted(files),
            "stdout_lines": stdout.getvalue().splitlines(),
        }
        assert_subset(case["case_id"], actual, case["expect"])
        return

    with tempfile.TemporaryDirectory(prefix="v2m_effect_cleanup_") as tmp:
        dest = pathlib.Path(tmp) / "qwen"
        dest.mkdir(parents=True, exist_ok=True)
        for entry in case["entries"]:
            make_entry(dest, entry)
        stdout = io.StringIO()
        with contextlib.redirect_stdout(stdout):
            dm._cleanup_qwen_artifacts(dest)
        actual = {
            "remaining": collect_files(dest),
            "stdout_lines": stdout.getvalue().splitlines(),
        }
    assert_subset(case["case_id"], actual, case["expect"])


def normalize_arg(arg: Any, dest: pathlib.Path) -> str:
    text = str(arg)
    if text == str(dest):
        return "__dest__"
    return text


def run_qwen_cli_download(case: dict[str, Any]) -> None:
    provider = case["provider"]
    resolve_calls: list[str] = []
    resolve_sequence = list(case.get("resolve_sequence", []))
    pip_calls: list[list[str]] = []
    run_calls: list[list[str]] = []
    cleanup_called = False

    with tempfile.TemporaryDirectory(prefix="v2m_effect_qwen_cli_") as tmp:
        root = pathlib.Path(tmp)
        dest = root / "experiments" / "Qwen3-ASR-1.7B"
        dest.mkdir(parents=True, exist_ok=True)

        def fake_resolve_cli(name: str) -> str | None:
            resolve_calls.append(name)
            if not resolve_sequence:
                return None
            return resolve_sequence.pop(0)

        def fake_pip_install(pkgs: list[str]) -> int:
            pip_calls.append(list(pkgs))
            return int(case.get("pip_rc", 0))

        def fake_venv_bin(name: str) -> pathlib.Path:
            return pathlib.Path(case.get("venv_bin", f"/venv/{name}"))

        def fake_run_cli(args: list[str]) -> int:
            run_calls.append([normalize_arg(arg, dest) for arg in args])
            return int(case.get("run_rc", 0))

        def fake_cleanup_qwen_artifacts(_dest: pathlib.Path) -> None:
            nonlocal cleanup_called
            cleanup_called = True

        def fake_qwen_has_weights(_dest: pathlib.Path) -> bool:
            return bool(case.get("weights_present", False))

        old_root = dm.ROOT_DIR
        old_resolve_cli = dm._resolve_cli
        old_pip_install = dm._pip_install
        old_venv_bin = dm._venv_bin
        old_run_cli = dm._run_cli
        old_cleanup = dm._cleanup_qwen_artifacts
        old_qwen_has_weights = dm.qwen_has_weights
        old_use_color = dm._USE_COLOR
        try:
            dm.ROOT_DIR = root
            dm._resolve_cli = fake_resolve_cli
            dm._pip_install = fake_pip_install
            dm._venv_bin = fake_venv_bin
            dm._run_cli = fake_run_cli
            dm._cleanup_qwen_artifacts = fake_cleanup_qwen_artifacts
            dm.qwen_has_weights = fake_qwen_has_weights
            dm._USE_COLOR = False
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                if provider == "modelscope":
                    result = dm.download_qwen_modelscope(dest)
                elif provider == "huggingface":
                    result = dm.download_qwen_huggingface(dest)
                else:
                    raise AssertionError(f"unknown provider {provider!r}")
        finally:
            dm.ROOT_DIR = old_root
            dm._resolve_cli = old_resolve_cli
            dm._pip_install = old_pip_install
            dm._venv_bin = old_venv_bin
            dm._run_cli = old_run_cli
            dm._cleanup_qwen_artifacts = old_cleanup
            dm.qwen_has_weights = old_qwen_has_weights
            dm._USE_COLOR = old_use_color

    actual = {
        "return": result,
        "resolve_calls": resolve_calls,
        "pip_calls": pip_calls,
        "run_calls": run_calls,
        "cleanup_called": cleanup_called,
        "stdout_lines": stdout.getvalue().splitlines(),
        "stderr_lines": stderr.getvalue().splitlines(),
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_download_qwen_strategy(case: dict[str, Any]) -> None:
    modelscope_calls: list[str] = []
    huggingface_calls: list[str] = []

    with tempfile.TemporaryDirectory(prefix="v2m_effect_qwen_strategy_") as tmp:
        root = pathlib.Path(tmp)
        qwen_dir = root / "experiments" / "Qwen3-ASR-1.7B"

        def fake_qwen_has_weights(_dest: pathlib.Path) -> bool:
            return bool(case["already_has_weights"])

        def fake_modelscope(dest: pathlib.Path) -> bool:
            modelscope_calls.append(dest.relative_to(root).as_posix())
            return bool(case.get("modelscope_result", False))

        def fake_huggingface(dest: pathlib.Path) -> bool:
            huggingface_calls.append(dest.relative_to(root).as_posix())
            return bool(case.get("huggingface_result", False))

        old_root = dm.ROOT_DIR
        old_qwen_local_dir = dm.QWEN_LOCAL_DIR
        old_qwen_has_weights = dm.qwen_has_weights
        old_modelscope = dm.download_qwen_modelscope
        old_huggingface = dm.download_qwen_huggingface
        old_use_color = dm._USE_COLOR
        try:
            dm.ROOT_DIR = root
            dm.QWEN_LOCAL_DIR = qwen_dir
            dm.qwen_has_weights = fake_qwen_has_weights
            dm.download_qwen_modelscope = fake_modelscope
            dm.download_qwen_huggingface = fake_huggingface
            dm._USE_COLOR = False
            stdout = io.StringIO()
            stderr = io.StringIO()
            with contextlib.redirect_stdout(stdout), contextlib.redirect_stderr(stderr):
                result = dm.download_qwen(case["source"], case["force"])
        finally:
            dm.ROOT_DIR = old_root
            dm.QWEN_LOCAL_DIR = old_qwen_local_dir
            dm.qwen_has_weights = old_qwen_has_weights
            dm.download_qwen_modelscope = old_modelscope
            dm.download_qwen_huggingface = old_huggingface
            dm._USE_COLOR = old_use_color

    actual = {
        "return": result,
        "modelscope_calls": modelscope_calls,
        "huggingface_calls": huggingface_calls,
        "stdout_lines": stdout.getvalue().splitlines(),
        "stderr_lines": stderr.getvalue().splitlines(),
    }
    assert_subset(case["case_id"], actual, case["expect"])


def run_case(case: dict[str, Any]) -> None:
    operation = case["operation"]
    if operation == "github_api_asset_sizes":
        run_github_api_asset_sizes(case)
    elif operation == "stream_download":
        run_stream_download(case)
    elif operation == "download_github_model":
        run_download_github_model(case)
    elif operation == "run_cli":
        run_run_cli(case)
    elif operation == "pip_install":
        run_pip_install(case)
    elif operation == "resolve_cli":
        run_resolve_cli(case)
    elif operation == "cleanup_qwen_artifacts":
        run_cleanup_qwen_artifacts(case)
    elif operation == "qwen_cli_download":
        run_qwen_cli_download(case)
    elif operation == "download_qwen_strategy":
        run_download_qwen_strategy(case)
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
