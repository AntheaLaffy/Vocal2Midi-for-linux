#!/usr/bin/env python3
"""Vendor source distributions for packages installed from uv.lock."""

from __future__ import annotations

import argparse
import hashlib
import importlib.metadata as metadata
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
import tomllib
from urllib.parse import urlparse
import urllib.request
import zipfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
LOCK_FILE = ROOT / "uv.lock"
DEFAULT_OUTPUT_DIR = ROOT / "third_party" / "sources"
DEFAULT_UPSTREAM_OUTPUT_DIR = ROOT / "third_party" / "upstream_sources"

UPSTREAM_FALLBACKS = {
    ("dynet38", "2.2"): {
        "url": "https://github.com/taishi-i/dynet/archive/79e80bfa56867319f35e14d8a7098d0da93ab243.tar.gz",
        "ref": "refs/heads/develop_python312_wheels",
        "commit": "79e80bfa56867319f35e14d8a7098d0da93ab243",
    },
    ("flatbuffers", "25.12.19"): {
        "url": "https://github.com/google/flatbuffers/archive/refs/tags/v25.12.19.tar.gz",
        "ref": "refs/tags/v25.12.19",
        "commit": "7e163021e59cca4f8e1e35a7c828b5c6b7915953",
    },
    ("onnxruntime", "1.27.0"): {
        "url": "https://github.com/microsoft/onnxruntime/archive/refs/tags/v1.27.0.tar.gz",
        "git_url": "https://github.com/microsoft/onnxruntime.git",
        "git_ref": "v1.27.0",
        "method": "git-recursive",
        "ref": "refs/tags/v1.27.0",
        "commit": "8f0278c77bf44b0cc83c098c6c722b92a36ac4b5",
    },
    ("pyqt5-qt5", "5.15.19"): {
        "url": "https://download.qt.io/archive/qt/5.15/5.15.19/single/qt-everywhere-opensource-src-5.15.19.tar.xz",
        "ref": "qt-everywhere-opensource-src-5.15.19",
        "commit": "archive",
    },
    ("torch", "2.13.0+cpu"): {
        "url": "https://github.com/pytorch/pytorch/archive/refs/tags/v2.13.0.tar.gz",
        "git_url": "https://github.com/pytorch/pytorch.git",
        "git_ref": "v2.13.0",
        "method": "git-recursive",
        "ref": "refs/tags/v2.13.0",
        "commit": "cf30153c4c131c8164ee7798e5022d810682e2cb",
    },
}

BINARY_ARTIFACT_SUFFIXES = {
    ".a",
    ".class",
    ".dll",
    ".dylib",
    ".elf",
    ".exe",
    ".jar",
    ".lib",
    ".o",
    ".obj",
    ".pyd",
    ".pyc",
    ".so",
    ".wasm",
    ".whl",
}

VENDOR_MARKER = ".vendor-source.json"


def normalize_name(value: str) -> str:
    return value.replace("_", "-").lower()


def safe_part(value: str) -> str:
    return "".join(ch if ch.isalnum() or ch in ".+-_" else "_" for ch in value)


def installed_distributions() -> dict[tuple[str, str], str]:
    installed: dict[tuple[str, str], str] = {}
    for dist in metadata.distributions():
        name = dist.metadata.get("Name")
        if not name:
            continue
        installed[(normalize_name(name), dist.version)] = name
    return installed


def load_locked_packages() -> dict[tuple[str, str], dict]:
    with LOCK_FILE.open("rb") as fp:
        lock = tomllib.load(fp)

    packages: dict[tuple[str, str], dict] = {}
    for package in lock.get("package", []):
        name = normalize_name(package["name"])
        version = package["version"]
        packages[(name, version)] = package
    return packages


def verify_hash(path: Path, expected: str | None) -> None:
    if not expected:
        return
    algorithm, _, digest = expected.partition(":")
    if algorithm != "sha256" or not digest:
        raise ValueError(f"Unsupported hash value: {expected}")

    hasher = hashlib.sha256()
    with path.open("rb") as fp:
        for chunk in iter(lambda: fp.read(1024 * 1024), b""):
            hasher.update(chunk)
    actual = hasher.hexdigest()
    if actual != digest:
        raise ValueError(f"Hash mismatch for {path.name}: expected {digest}, got {actual}")


def assert_safe_member(base: Path, member_name: str) -> Path:
    target = (base / member_name).resolve()
    base_resolved = base.resolve()
    if os.path.commonpath([base_resolved, target]) != str(base_resolved):
        raise ValueError(f"Archive member escapes output directory: {member_name}")
    return target


def extract_archive(archive: Path, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    if tarfile.is_tarfile(archive):
        with tarfile.open(archive) as tf:
            for member in tf.getmembers():
                assert_safe_member(destination, member.name)
            tf.extractall(destination, filter="data")
        return

    if zipfile.is_zipfile(archive):
        with zipfile.ZipFile(archive) as zf:
            for member in zf.namelist():
                assert_safe_member(destination, member)
            zf.extractall(destination)
        return

    raise ValueError(f"Unsupported source archive type: {archive.name}")


def download(url: str, target: Path) -> None:
    request = urllib.request.Request(url, headers={"User-Agent": "vocal2midi-source-vendor/1.0"})
    with urllib.request.urlopen(request, timeout=120) as response:
        with target.open("wb") as fp:
            shutil.copyfileobj(response, fp)


def sha256_digest(path: Path) -> str:
    hasher = hashlib.sha256()
    with path.open("rb") as fp:
        for chunk in iter(lambda: fp.read(1024 * 1024), b""):
            hasher.update(chunk)
    return hasher.hexdigest()


def move_unpacked_archive(unpacked: Path, source_dir: Path) -> None:
    if source_dir.exists():
        shutil.rmtree(source_dir)
    source_dir.parent.mkdir(parents=True, exist_ok=True)

    children = [child for child in unpacked.iterdir()]
    if len(children) == 1 and children[0].is_dir():
        shutil.move(str(children[0]), source_dir)
    else:
        source_dir.mkdir()
        for child in children:
            shutil.move(str(child), source_dir / child.name)


def remove_git_metadata(source_dir: Path) -> None:
    for path in sorted(source_dir.rglob(".git"), reverse=True):
        if path.is_dir():
            shutil.rmtree(path)
        elif path.exists():
            path.unlink()


def write_vendor_marker(source_dir: Path, fallback: dict, resolved_commit: str | None) -> None:
    marker = {
        "method": fallback.get("method", "archive"),
        "url": fallback.get("git_url") or fallback["url"],
        "ref": fallback.get("git_ref") or fallback["ref"],
        "expected_commit": fallback.get("commit"),
        "resolved_commit": resolved_commit,
    }
    (source_dir / VENDOR_MARKER).write_text(json.dumps(marker, indent=2) + "\n", encoding="utf-8")


def has_vendor_marker(source_dir: Path, fallback: dict) -> bool:
    marker_path = source_dir / VENDOR_MARKER
    if not marker_path.exists():
        return False
    try:
        marker = json.loads(marker_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        return False
    if marker.get("method") != fallback.get("method", "archive"):
        return False
    expected_commit = fallback.get("commit")
    return not expected_commit or marker.get("resolved_commit") == expected_commit


def remove_binary_artifacts(source_dir: Path) -> list[str]:
    removed: list[str] = []
    for path in sorted(source_dir.rglob("*")):
        if not path.is_file():
            continue
        suffixes = {suffix.lower() for suffix in path.suffixes}
        if not suffixes.intersection(BINARY_ARTIFACT_SUFFIXES):
            continue
        removed.append(str(path.relative_to(ROOT)))
        path.unlink()

    for path in sorted(source_dir.rglob("__pycache__"), reverse=True):
        if path.is_dir():
            shutil.rmtree(path)

    return removed


def find_binary_artifacts(source_dir: Path) -> list[str]:
    found: list[str] = []
    if not source_dir.exists():
        return found
    for path in sorted(source_dir.rglob("*")):
        if not path.is_file():
            continue
        suffixes = {suffix.lower() for suffix in path.suffixes}
        if suffixes.intersection(BINARY_ARTIFACT_SUFFIXES):
            found.append(str(path.relative_to(ROOT)))
    return found


def vendor_git_fallback(fallback: dict, source_dir: Path, tmp_root: Path) -> str:
    tmp_root.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix="git-source-", dir=tmp_root) as tmp_name:
        tmp_dir = Path(tmp_name)
        clone_dir = tmp_dir / "source"
        command = [
            "git",
            "clone",
            "--depth",
            "1",
            "--branch",
            fallback["git_ref"],
            "--recurse-submodules",
            "--shallow-submodules",
            fallback["git_url"],
            str(clone_dir),
        ]
        env = os.environ.copy()
        env["GIT_TERMINAL_PROMPT"] = "0"
        subprocess.run(command, cwd=ROOT, env=env, check=True)
        resolved_commit = subprocess.check_output(
            ["git", "-C", str(clone_dir), "rev-parse", "HEAD"],
            text=True,
        ).strip()
        if fallback.get("commit") and resolved_commit != fallback["commit"]:
            raise ValueError(
                f"Git fallback commit mismatch for {fallback['git_url']}: "
                f"expected {fallback['commit']}, got {resolved_commit}"
            )
        remove_git_metadata(clone_dir)
        if source_dir.exists():
            shutil.rmtree(source_dir)
        shutil.move(str(clone_dir), source_dir)
    return resolved_commit


def vendor_source(package: dict, destination: Path, force: bool) -> dict:
    name = normalize_name(package["name"])
    version = package["version"]
    source_dir = destination / f"{safe_part(name)}-{safe_part(version)}"
    sdist = package.get("sdist")
    record = {
        "name": name,
        "version": version,
        "source_dir": str(source_dir.relative_to(ROOT)),
    }

    if not sdist:
        record["status"] = "missing-sdist"
        record["reason"] = "uv.lock has no source distribution for this package"
        return record

    if source_dir.exists() and not force:
        purged = remove_binary_artifacts(source_dir)
        record["status"] = "exists"
        record["url"] = sdist["url"]
        if purged:
            record["purged_binary_artifacts"] = purged
        return record

    tmp_root = destination / ".tmp"
    tmp_root.mkdir(parents=True, exist_ok=True)
    archive_name = Path(urlparse(sdist["url"]).path).name or f"{name}-{version}.sdist"
    with tempfile.TemporaryDirectory(prefix=f"{name}-", dir=tmp_root) as tmp_name:
        tmp_dir = Path(tmp_name)
        archive = tmp_dir / archive_name
        download(sdist["url"], archive)
        verify_hash(archive, sdist.get("hash"))

        unpacked = tmp_dir / "unpacked"
        extract_archive(archive, unpacked)

        move_unpacked_archive(unpacked, source_dir)

    purged = remove_binary_artifacts(source_dir)
    record["status"] = "downloaded"
    record["url"] = sdist["url"]
    record["hash"] = sdist.get("hash")
    if purged:
        record["purged_binary_artifacts"] = purged
    return record


def vendor_upstream_fallbacks(records: list[dict], destination: Path, force: bool) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    tmp_root = destination / ".tmp"

    for record in records:
        if record["status"] != "missing-sdist":
            continue
        fallback = UPSTREAM_FALLBACKS.get((record["name"], record["version"]))
        if fallback is None:
            continue

        source_dir = destination / f"{safe_part(record['name'])}-{safe_part(record['version'])}"
        record["upstream_source_dir"] = str(source_dir.relative_to(ROOT))
        record["upstream_url"] = fallback["url"]
        record["upstream_ref"] = fallback["ref"]
        record["upstream_commit"] = fallback["commit"]
        record["upstream_method"] = fallback.get("method", "archive")

        source_is_complete = fallback.get("method") != "git-recursive" or has_vendor_marker(source_dir, fallback)
        if source_dir.exists() and not force and source_is_complete:
            purged = remove_binary_artifacts(source_dir)
            if not (source_dir / VENDOR_MARKER).exists():
                write_vendor_marker(source_dir, fallback, fallback.get("commit"))
            record["upstream_status"] = "exists"
            if purged:
                record["purged_upstream_binary_artifacts"] = purged
            continue

        tmp_root.mkdir(parents=True, exist_ok=True)
        resolved_commit = None
        if fallback.get("method") == "git-recursive":
            print(f"Cloning recursive upstream source for {record['name']}=={record['version']}")
            resolved_commit = vendor_git_fallback(fallback, source_dir, tmp_root)
            record["upstream_resolved_commit"] = resolved_commit
        else:
            archive_name = Path(urlparse(fallback["url"]).path).name or f"{record['name']}-{record['version']}.tar.gz"
            with tempfile.TemporaryDirectory(prefix=f"{record['name']}-", dir=tmp_root) as tmp_name:
                tmp_dir = Path(tmp_name)
                archive = tmp_dir / archive_name
                print(f"Downloading upstream source for {record['name']}=={record['version']}")
                download(fallback["url"], archive)
                record["upstream_archive_sha256"] = sha256_digest(archive)

                unpacked = tmp_dir / "unpacked"
                extract_archive(archive, unpacked)
                move_unpacked_archive(unpacked, source_dir)

        purged = remove_binary_artifacts(source_dir)
        write_vendor_marker(source_dir, fallback, resolved_commit or fallback.get("commit"))
        record["upstream_status"] = "downloaded"
        if purged:
            record["purged_upstream_binary_artifacts"] = purged

    if tmp_root.exists():
        shutil.rmtree(tmp_root)


def write_missing_report(records: list[dict], output_dir: Path) -> None:
    missing = [record for record in records if record["status"] == "missing-sdist"]
    upstream = [record for record in missing if "upstream_status" in record]
    binary_only = [record for record in missing if "upstream_status" not in record]
    report = output_dir / "MISSING_SOURCES.md"
    lines = [
        "# Missing Source Distributions",
        "",
        "These installed packages are pinned in `uv.lock` but do not expose an sdist in the lock file.",
        "When a pinned upstream ref is available, the source is vendored under `third_party/upstream_sources/`.",
        "",
    ]

    if upstream:
        lines.append("## Upstream Source Fallbacks")
        lines.append("")
        lines.append("| Package | Version | Ref | Source directory |")
        lines.append("| --- | --- | --- | --- |")
        for record in sorted(upstream, key=lambda item: item["name"]):
            lines.append(
                f"| `{record['name']}` | `{record['version']}` | "
                f"`{record['upstream_ref']}` | `{record['upstream_source_dir']}` |"
            )
        lines.append("")

    if binary_only:
        lines.append("## Binary-only Exceptions")
        lines.append("")
        lines.append("| Package | Version | Reason |")
        lines.append("| --- | --- | --- |")
        for record in sorted(binary_only, key=lambda item: item["name"]):
            lines.append(f"| `{record['name']}` | `{record['version']}` | {record['reason']} |")
    else:
        lines.append("All packages without sdists have upstream source fallbacks.")
    lines.append("")
    report.write_text("\n".join(lines), encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, default=DEFAULT_OUTPUT_DIR)
    parser.add_argument("--upstream-output-dir", type=Path, default=DEFAULT_UPSTREAM_OUTPUT_DIR)
    parser.add_argument("--force", action="store_true", help="redownload and replace existing source directories")
    args = parser.parse_args()

    if not LOCK_FILE.exists():
        raise SystemExit("uv.lock does not exist. Run `uv lock` first.")

    output_dir = args.output_dir.resolve()
    output_dir.mkdir(parents=True, exist_ok=True)

    installed = installed_distributions()
    locked = load_locked_packages()

    records: list[dict] = []
    missing_lock: list[dict] = []
    for key, display_name in sorted(installed.items()):
        package = locked.get(key)
        if package is None:
            missing_lock.append({"name": normalize_name(display_name), "version": key[1]})
            continue
        records.append(vendor_source(package, output_dir, args.force))

    if missing_lock:
        raise SystemExit(f"Installed packages missing from uv.lock: {missing_lock}")

    tmp_dir = output_dir / ".tmp"
    if tmp_dir.exists():
        shutil.rmtree(tmp_dir)

    vendor_upstream_fallbacks(records, args.upstream_output_dir.resolve(), args.force)

    remaining_binary_artifacts = find_binary_artifacts(output_dir) + find_binary_artifacts(args.upstream_output_dir.resolve())

    manifest = {
        "lock_file": str(LOCK_FILE.relative_to(ROOT)),
        "package_count": len(records),
        "downloaded_count": sum(1 for record in records if record["status"] == "downloaded"),
        "existing_count": sum(1 for record in records if record["status"] == "exists"),
        "missing_sdist_count": sum(1 for record in records if record["status"] == "missing-sdist"),
        "upstream_fallback_count": sum(1 for record in records if "upstream_status" in record),
        "purged_binary_artifact_count": sum(
            len(record.get("purged_binary_artifacts", []))
            + len(record.get("purged_upstream_binary_artifacts", []))
            for record in records
        ),
        "remaining_binary_artifact_count": len(remaining_binary_artifacts),
        "remaining_binary_artifacts": remaining_binary_artifacts,
        "packages": sorted(records, key=lambda item: item["name"]),
    }
    (output_dir / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    write_missing_report(records, output_dir)

    if remaining_binary_artifacts:
        raise SystemExit(f"Binary artifacts remain in vendored sources: {remaining_binary_artifacts[:20]}")

    print(
        "Vendored "
        f"{manifest['downloaded_count']} downloaded, "
        f"{manifest['existing_count']} existing, "
        f"{manifest['missing_sdist_count']} missing sdist "
        f"({manifest['upstream_fallback_count']} upstream fallbacks) "
        f"and purged {manifest['purged_binary_artifact_count']} binary artifacts "
        f"out of {manifest['package_count']} installed packages."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
