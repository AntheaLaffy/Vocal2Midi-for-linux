#!/usr/bin/env python3
"""Download model assets for Vocal2Midi.

All ONNX models are published as zip assets on the project's own GitHub
release (AntheaLaffy/Vocal2Midi-for-linux v0.1.0) and extracted under
``experiments/``:

    game    GAME note/pitch extraction        GAME-1.0.3-medium-onnx.zip
    hfa     HubertFA forced alignment         1218_hfa_model_new_dict.zip
    rmvpe   RMVPE pitch estimation            RMVPE.zip
    romaji  romajiASR Japanese mora ASR       romajiASR.zip

The Qwen3-ASR-1.7B model is too large for a GitHub release asset, so it is
fetched from ModelScope (preferred for Mainland China) or Hugging Face, using
the official ``modelscope`` / ``huggingface_hub`` CLIs when available.

Examples
--------
    # download everything that is missing
    python download_models.py

    # only fetch a single model
    python download_models.py --only rmvpe
    python download_models.py --only game --only hfa

    # re-download even if the target looks complete
    python download_models.py --force

    # choose the Qwen source explicitly
    python download_models.py --qwen-source modelscope
    python download_models.py --qwen-source huggingface
    python download_models.py --qwen-source skip

    # show what would be done without downloading
    python download_models.py --list
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath, PureWindowsPath
from typing import Iterable, Optional

ROOT_DIR = Path(__file__).resolve().parent
EXPERIMENTS_DIR = ROOT_DIR / "experiments"

# All ONNX models are published as assets on this self-hosted release.
GITHUB_REPO = "AntheaLaffy/Vocal2Midi-for-linux"
RELEASE_TAG = "v0.1.0"

# Qwen model identifiers on ModelScope / Hugging Face
QWEN_MODEL_ID = "Qwen/Qwen3-ASR-1.7B"
QWEN_LOCAL_DIR = EXPERIMENTS_DIR / "Qwen3-ASR-1.7B"


@dataclass(frozen=True)
class GithubModel:
    """A model distributed as a zip asset on a GitHub release."""

    name: str        # short id for --only
    repo: str        # e.g. "openvpi/GAME"
    tag: str         # e.g. "v1.0.3"
    asset: str       # e.g. "GAME-1.0.3-medium-onnx.zip"
    target: Path     # extraction destination under experiments/
    marker: str      # file whose presence means "already downloaded"
    label: str       # human-readable source label


GITHUB_MODELS: list[GithubModel] = [
    GithubModel(
        name="game",
        repo=GITHUB_REPO,
        tag=RELEASE_TAG,
        asset="GAME-1.0.3-medium-onnx.zip",
        target=EXPERIMENTS_DIR / "GAME-1.0.3-medium-onnx",
        marker="encoder.onnx",
        label=f"{GITHUB_REPO} {RELEASE_TAG}",
    ),
    GithubModel(
        name="hfa",
        repo=GITHUB_REPO,
        tag=RELEASE_TAG,
        asset="1218_hfa_model_new_dict.zip",
        target=EXPERIMENTS_DIR / "1218_hfa_model_new_dict",
        marker="model.onnx",
        label=f"{GITHUB_REPO} {RELEASE_TAG}",
    ),
    GithubModel(
        name="rmvpe",
        repo=GITHUB_REPO,
        tag=RELEASE_TAG,
        asset="RMVPE.zip",
        target=EXPERIMENTS_DIR / "RMVPE",
        marker="rmvpe.onnx",
        label=f"{GITHUB_REPO} {RELEASE_TAG}",
    ),
    GithubModel(
        name="romaji",
        repo=GITHUB_REPO,
        tag=RELEASE_TAG,
        asset="romajiASR.zip",
        target=EXPERIMENTS_DIR / "romajiASR",
        marker="model.onnx",
        label=f"{GITHUB_REPO} {RELEASE_TAG}",
    ),
]

GITHUB_MODEL_BY_NAME: dict[str, GithubModel] = {m.name: m for m in GITHUB_MODELS}

CHUNK_SIZE = 1024 * 256  # 256 KiB chunks for streaming downloads


class Color:
    RESET = "\033[0m"
    BOLD = "\033[1m"
    GREEN = "\033[32m"
    YELLOW = "\033[33m"
    RED = "\033[31m"
    CYAN = "\033[36m"
    DIM = "\033[2m"


def supports_color() -> bool:
    if not sys.stdout.isatty():
        return False
    term = os.environ.get("TERM", "")
    return term != "dumb"


_USE_COLOR = supports_color()


def paint(text: str, color: str) -> str:
    if not _USE_COLOR:
        return text
    return f"{color}{text}{Color.RESET}"


def info(msg: str) -> None:
    print(paint("•", Color.CYAN), msg)


def ok(msg: str) -> None:
    print(paint("✓", Color.GREEN), msg)


def warn(msg: str) -> None:
    print(paint("!", Color.YELLOW), msg)


def fail(msg: str) -> None:
    print(paint("✗", Color.RED), msg, file=sys.stderr)


def human_size(num_bytes: int) -> str:
    size = float(num_bytes)
    for unit in ("B", "KiB", "MiB", "GiB", "TiB"):
        if size < 1024.0 or unit == "TiB":
            if unit == "B":
                return f"{int(size)} {unit}"
            return f"{size:.1f} {unit}"
        size /= 1024.0
    return f"{num_bytes} B"


def asset_url(model: GithubModel) -> str:
    return (
        f"https://github.com/{model.repo}/releases/download/"
        f"{model.tag}/{model.asset}"
    )


_ASSET_SIZE_CACHE: dict[tuple[str, str], dict[str, int]] = {}


def github_api_asset_sizes(repo: str, tag: str) -> dict[str, int]:
    """Return a mapping of asset name -> size in bytes for a repo/tag.

    Cached per (repo, tag). Falls back to an empty dict on any failure.
    """
    key = (repo, tag)
    if key in _ASSET_SIZE_CACHE:
        return _ASSET_SIZE_CACHE[key]
    api_url = f"https://api.github.com/repos/{repo}/releases/tags/{tag}"
    try:
        req = urllib.request.Request(
            api_url, headers={"Accept": "application/vnd.github+json"}
        )
        with urllib.request.urlopen(req, timeout=20) as resp:
            payload = json.loads(resp.read().decode("utf-8"))
    except (urllib.error.URLError, TimeoutError, ValueError):
        _ASSET_SIZE_CACHE[key] = {}
        return {}
    sizes: dict[str, int] = {}
    for asset in payload.get("assets", []):
        name = asset.get("name")
        size = asset.get("size")
        if name and isinstance(size, int):
            sizes[name] = size
    _ASSET_SIZE_CACHE[key] = sizes
    return sizes


def stream_download(url: str, dest: Path) -> int:
    """Stream ``url`` to ``dest`` with a simple progress line. Returns bytes."""
    req = urllib.request.Request(url, headers={"User-Agent": "vocal2midi-model-fetch/1.0"})
    with urllib.request.urlopen(req, timeout=60) as resp:
        total = resp.headers.get("Content-Length")
        total_int = int(total) if total and total.isdigit() else 0

        downloaded = 0
        last_pct = -1
        with open(dest, "wb") as fh:
            while True:
                chunk = resp.read(CHUNK_SIZE)
                if not chunk:
                    break
                fh.write(chunk)
                downloaded += len(chunk)
                if total_int:
                    pct = int(downloaded * 100 / total_int)
                    if pct != last_pct:
                        last_pct = pct
                        sys.stdout.write(
                            f"\r  {pct:3d}%  {human_size(downloaded)}"
                            + (f" / {human_size(total_int)}" if total_int else "")
                        )
                        sys.stdout.flush()
                else:
                    sys.stdout.write(f"\r  {human_size(downloaded)}")
                    sys.stdout.flush()
        sys.stdout.write("\n")
        sys.stdout.flush()
    return downloaded


def extract_zip(zip_path: Path, target_dir: Path) -> None:
    """Extract ``zip_path`` into ``target_dir``.

    Handles zips that either include a single top-level folder or contain
    files directly. Existing files are overwritten.
    """
    target_dir.mkdir(parents=True, exist_ok=True)

    with zipfile.ZipFile(zip_path) as zf:
        names = zf.namelist()

    top_levels = {n.split("/", 1)[0] for n in names if n}
    single_top = (
        len(top_levels) == 1
        and not next(iter(top_levels)).endswith(".onnx")
        and not next(iter(top_levels)).endswith(".zip")
        and next(iter(top_levels)) != ""
    )

    with tempfile.TemporaryDirectory(prefix="v2m_extract_") as tmp:
        tmp_path = Path(tmp)
        with zipfile.ZipFile(zip_path) as zf:
            _safe_extractall(zf, tmp_path)

        if single_top:
            root = tmp_path / next(iter(top_levels))
            if root.is_dir():
                _merge_tree(root, target_dir)
                return

        _merge_tree(tmp_path, target_dir)


def _safe_extractall(zf: zipfile.ZipFile, destination: Path) -> None:
    """Extract zip members after rejecting paths that escape destination."""
    destination = destination.resolve()
    for member in zf.infolist():
        member_path = _validated_zip_member_path(member.filename)
        target = (destination / Path(*member_path.parts)).resolve()
        if not target.is_relative_to(destination):
            raise ValueError(f"Unsafe zip member path: {member.filename!r}")
    zf.extractall(destination)


def _validated_zip_member_path(name: str) -> PurePosixPath:
    if not name or "\x00" in name or "\\" in name:
        raise ValueError(f"Unsafe zip member path: {name!r}")

    posix_path = PurePosixPath(name)
    windows_path = PureWindowsPath(name)
    if (
        posix_path.is_absolute()
        or windows_path.is_absolute()
        or windows_path.drive
        or ".." in posix_path.parts
    ):
        raise ValueError(f"Unsafe zip member path: {name!r}")
    return posix_path


def _merge_tree(src: Path, dst: Path) -> None:
    """Copy ``src`` into ``dst``, merging directories and overwriting files."""
    dst.mkdir(parents=True, exist_ok=True)
    for item in src.iterdir():
        s = src / item.name
        d = dst / item.name
        if s.is_dir():
            _merge_tree(s, d)
        else:
            shutil.copy2(s, d)


def target_has_model(model: GithubModel) -> bool:
    return (model.target / model.marker).exists()


def download_github_model(model: GithubModel, force: bool) -> bool:
    if not force and target_has_model(model):
        ok(f"{model.target.relative_to(ROOT_DIR)} already present, skipping "
           f"(use --force to re-download)")
        return True

    url = asset_url(model)
    expected_size = github_api_asset_sizes(model.repo, model.tag).get(model.asset, 0)
    size_hint = f"  ({human_size(expected_size)})" if expected_size else ""
    info(f"Downloading {model.asset} from {model.label}{size_hint}")
    print(f"  {paint(url, Color.DIM)}")

    EXPERIMENTS_DIR.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(
        prefix="v2m_dl_", suffix=".zip", dir=EXPERIMENTS_DIR, delete=False
    ) as tf:
        tmp_zip = Path(tf.name)

    try:
        downloaded = stream_download(url, tmp_zip)

        # Layer 1: verify download size against GitHub API asset size.
        if expected_size and downloaded != expected_size:
            fail(f"Size mismatch for {model.asset}: got {human_size(downloaded)}, "
                 f"expected {human_size(expected_size)} (download may be truncated)")
            return False
        if expected_size:
            ok(f"Size verified: {human_size(downloaded)}")

        info(f"Extracting {model.asset} -> {model.target.relative_to(ROOT_DIR)}")
        try:
            extract_zip(tmp_zip, model.target)
        except zipfile.BadZipFile:
            fail(f"Corrupted zip archive: {model.asset} (re-run with --force to retry)")
            return False
        except ValueError as e:
            fail(f"Unsafe zip archive layout in {model.asset}: {e}")
            return False

        # Layer 2: verify the marker file appeared after extraction.
        if not target_has_model(model):
            fail(f"Extraction finished but marker '{model.marker}' not found in "
                 f"{model.target.relative_to(ROOT_DIR)} — the zip may have an "
                 f"unexpected layout")
            return False
    except urllib.error.HTTPError as e:
        fail(f"HTTP {e.code} downloading {model.asset}")
        if e.code == 404:
            print(
                f"  {paint('note:', Color.YELLOW)} asset not found on "
                f"{model.label}. Check "
                f"https://github.com/{model.repo}/releases/tag/{model.tag}"
            )
        return False
    finally:
        try:
            tmp_zip.unlink(missing_ok=True)
        except OSError:
            pass

    ok(f"{model.target.relative_to(ROOT_DIR)} ready")
    return True


def _run_cli(args: list[str]) -> int:
    try:
        return subprocess.run(args, check=False).returncode
    except FileNotFoundError:
        return 127


def _have_uv() -> bool:
    """True when ``uv`` is available on PATH."""
    return shutil.which("uv") is not None


def _pip_install(pkgs: list[str]) -> int:
    """Install packages into the running interpreter's environment.

    Prefers ``uv pip install`` (targeting ``sys.executable``) when uv is
    available; falls back to ``python -m pip install`` otherwise.
    """
    if _have_uv():
        info(f"Installing with uv: {' '.join(pkgs)}")
        return _run_cli(
            ["uv", "pip", "install", "--python", sys.executable, *pkgs]
        )
    info(f"Installing with pip: {' '.join(pkgs)}")
    return _run_cli([sys.executable, "-m", "pip", "install", *pkgs])


def _venv_bin(name: str) -> Path:
    """Resolve a CLI binary shipped in the same environment as sys.executable."""
    bin_dir = Path(sys.executable).parent
    if os.name == "nt":
        return bin_dir / f"{name}.exe"
    return bin_dir / name


def _resolve_cli(name: str) -> Optional[str]:
    """Find a CLI binary: prefer the venv bin, then PATH. None if missing."""
    bin_path = _venv_bin(name)
    if bin_path.exists():
        return str(bin_path)
    found = shutil.which(name)
    return found


def _cleanup_qwen_artifacts(dest: Path) -> None:
    """Remove files that ModelScope/HF drop into the model dir but that
    conflict with this repo's non-LFS setup or are leftover partial downloads.

    - ``.gitattributes``: ModelScope ships one that forces ``filter=lfs`` on
      tokenizer/weight files. Without git-lfs installed it breaks every git
      operation touching the directory, so delete it.
    - ``*.incomplete``: aborted/partial weight downloads.
    """
    gitattr = dest / ".gitattributes"
    if gitattr.exists():
        try:
            gitattr.unlink()
            info("Removed modelscope .gitattributes (would force LFS filters)")
        except OSError:
            pass
    for entry in dest.glob("*.incomplete"):
        try:
            entry.unlink()
            info(f"Removed partial download {entry.name}")
        except OSError:
            pass


def download_qwen_modelscope(dest: Path) -> bool:
    cli = _resolve_cli("modelscope")
    if not cli:
        warn("modelscope CLI not found. Installing modelscope...")
        rc = _pip_install(["-U", "modelscope"])
        if rc != 0:
            fail("Failed to install modelscope. Run manually: "
                 "uv pip install -U modelscope  (or: pip install -U modelscope)")
            return False
        cli = _resolve_cli("modelscope")
        if not cli:
            cli = str(_venv_bin("modelscope"))
    info(f"Downloading {QWEN_MODEL_ID} from ModelScope -> {dest.relative_to(ROOT_DIR)}")
    rc = _run_cli([
        cli, "download",
        "--model", QWEN_MODEL_ID,
        "--local_dir", str(dest),
    ])
    if rc != 0:
        fail("modelscope download failed")
        return False
    _cleanup_qwen_artifacts(dest)
    if not qwen_has_weights(dest):
        fail(f"No model weights (.safetensors/.bin) found in "
             f"{dest.relative_to(ROOT_DIR)} after download")
        return False
    ok(f"{dest.relative_to(ROOT_DIR)} ready")
    return True


def download_qwen_huggingface(dest: Path) -> bool:
    cli = _resolve_cli("huggingface-cli")
    if not cli:
        warn("huggingface-cli not found. Installing huggingface_hub[cli]...")
        rc = _pip_install(["-U", "huggingface_hub[cli]"])
        if rc != 0:
            fail("Failed to install huggingface_hub. Run manually: "
                 "uv pip install -U \"huggingface_hub[cli]\"  "
                 "(or: pip install -U \"huggingface_hub[cli]\")")
            return False
        cli = _resolve_cli("huggingface-cli")
        if not cli:
            cli = str(_venv_bin("huggingface-cli"))
    info(f"Downloading {QWEN_MODEL_ID} from Hugging Face -> {dest.relative_to(ROOT_DIR)}")
    rc = _run_cli([
        cli, "download", QWEN_MODEL_ID, "--local-dir", str(dest),
    ])
    if rc != 0:
        fail("huggingface-cli download failed")
        return False
    _cleanup_qwen_artifacts(dest)
    if not qwen_has_weights(dest):
        fail(f"No model weights (.safetensors/.bin) found in "
             f"{dest.relative_to(ROOT_DIR)} after download")
        return False
    ok(f"{dest.relative_to(ROOT_DIR)} ready")
    return True


def qwen_has_weights(dest: Path) -> bool:
    if not dest.exists():
        return False
    for entry in dest.iterdir():
        name = entry.name.lower()
        if name.endswith(".safetensors") or name.endswith(".bin"):
            return True
    return False


def download_qwen(source: str, force: bool) -> bool:
    dest = QWEN_LOCAL_DIR
    if not force and qwen_has_weights(dest):
        ok(f"{dest.relative_to(ROOT_DIR)} already has weights, skipping "
            f"(use --force to re-download)")
        return True

    if source == "modelscope":
        return download_qwen_modelscope(dest)
    if source == "huggingface":
        return download_qwen_huggingface(dest)
    if source == "auto":
        info("Trying ModelScope first (preferred for Mainland China)...")
        if download_qwen_modelscope(dest):
            return True
        warn("ModelScope failed, falling back to Hugging Face...")
        return download_qwen_huggingface(dest)

    fail(f"Unknown qwen source: {source}")
    return False


def list_planned(qwen_source: str) -> None:
    print(paint("ONNX models (GitHub releases):", Color.BOLD))
    for m in GITHUB_MODELS:
        sizes = github_api_asset_sizes(m.repo, m.tag)
        size_str = human_size(sizes[m.asset]) if m.asset in sizes else "size unknown"
        present = target_has_model(m)
        marker = paint("✓", Color.GREEN) if present else paint("✗", Color.RED)
        print(
            f"  {marker} {m.name:<7} {m.asset:<34} {size_str:>12}  -> "
            f"{m.target.relative_to(ROOT_DIR)}"
        )
        print(f"          {paint(m.label, Color.DIM)}")
    print(paint("Qwen3-ASR-1.7B:", Color.BOLD))
    present = qwen_has_weights(QWEN_LOCAL_DIR)
    marker = paint("✓", Color.GREEN) if present else paint("✗", Color.RED)
    src = qwen_source if qwen_source != "skip" else "skipped"
    print(
        f"  {marker} qwen    {QWEN_MODEL_ID:<34} {'large':>12}  -> "
        f"{QWEN_LOCAL_DIR.relative_to(ROOT_DIR)}  (source: {src})"
    )


def parse_args(argv: Optional[Iterable[str]] = None) -> argparse.Namespace:
    only_choices = [*GITHUB_MODEL_BY_NAME.keys(), "qwen"]
    p = argparse.ArgumentParser(
        description="Download Vocal2Midi model assets.",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    p.add_argument(
        "--only",
        action="append",
        choices=only_choices,
        help="Download only the selected model(s). Can be repeated. "
             f"Choices: {', '.join(only_choices)}.",
    )
    p.add_argument(
        "--force",
        action="store_true",
        help="Re-download even if the target looks complete.",
    )
    p.add_argument(
        "--qwen-source",
        choices=["auto", "modelscope", "huggingface", "skip"],
        default="auto",
        help="Source for the Qwen3-ASR model (default: auto = modelscope then huggingface).",
    )
    p.add_argument(
        "--no-qwen",
        action="store_true",
        help="Skip the Qwen3-ASR model entirely.",
    )
    p.add_argument(
        "--list",
        action="store_true",
        help="Show planned actions and asset status, then exit.",
    )
    return p.parse_args(argv)


def main(argv: Optional[Iterable[str]] = None) -> int:
    args = parse_args(argv)

    if args.list:
        list_planned(args.qwen_source if not args.no_qwen else "skip")
        return 0

    EXPERIMENTS_DIR.mkdir(parents=True, exist_ok=True)

    selected = args.only
    do_qwen = not args.no_qwen and (selected is None or "qwen" in selected)

    failures: list[str] = []

    for model in GITHUB_MODELS:
        if selected is not None and model.name not in selected:
            continue
        if not download_github_model(model, args.force):
            failures.append(model.name)

    if do_qwen:
        if not download_qwen(args.qwen_source, args.force):
            failures.append("qwen")

    print()
    if failures:
        fail("Failed to fetch: " + ", ".join(failures))
        print()
        print("Tips:")
        print("  - re-run with --force to retry")
        print("  - for Qwen, try: --qwen-source huggingface")
        return 1

    ok("All requested models are ready under experiments/")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
