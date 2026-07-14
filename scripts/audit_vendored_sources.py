#!/usr/bin/env python3
"""Audit that installed Python dependencies have source-only vendored inputs."""

from __future__ import annotations

import configparser
import importlib.metadata as metadata
import json
import os
import sysconfig
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
THIRD_PARTY = ROOT / "third_party"
PYTHON_MANIFEST = THIRD_PARTY / "sources" / "manifest.json"
NATIVE_MANIFEST = THIRD_PARTY / "native_sources" / "manifest.json"
AUDIT_REPORT = THIRD_PARTY / "source_audit.json"

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

RUNTIME_NATIVE_SUFFIXES = {
    ".a",
    ".dll",
    ".dylib",
    ".pyd",
    ".so",
    ".wasm",
}

FOREIGN_NATIVE_PREFIXES = (
    "_soundfile_data/",
    "av.libs/",
    "dyNET38.libs/",
    "llvmlite/binding/libllvmlite",
    "numpy.libs/",
    "onnxruntime/capi/",
    "pillow.libs/",
    "PyQt5/Qt5/",
    "scikit_learn.libs/",
    "scipy.libs/",
    "torch/lib/",
)

FOREIGN_NATIVE_COVERAGE = (
    ("_soundfile_data/libsndfile", "third_party/native_sources/libsndfile-1.2.2"),
    ("av.libs/libSvtAv1Enc", "third_party/native_sources/svt-av1-4.1.0"),
    ("av.libs/libXau", "third_party/native_sources/libxau-1.0.12"),
    ("av.libs/libasound", "third_party/native_sources/alsa-lib-1.2.14"),
    ("av.libs/libav", "third_party/native_sources/ffmpeg-8.0.1"),
    ("av.libs/libdav1d", "third_party/native_sources/dav1d-1.5.1"),
    ("av.libs/libdrm", "third_party/native_sources/libdrm-2.4.125"),
    ("av.libs/libgmp", "third_party/native_sources/gmp-6.3.0"),
    ("av.libs/libgnutls", "third_party/native_sources/gnutls-3.8.10"),
    ("av.libs/libhogweed", "third_party/native_sources/nettle-3.10.2"),
    ("av.libs/libmp3lame", "third_party/native_sources/lame-3.100"),
    ("av.libs/libnettle", "third_party/native_sources/nettle-3.10.2"),
    ("av.libs/libopencore", "third_party/native_sources/opencore-amr-0.1.6"),
    ("av.libs/libopus", "third_party/native_sources/opus-1.5.2"),
    ("av.libs/libsharpyuv", "third_party/native_sources/libwebp-1.6.0"),
    ("av.libs/libsw", "third_party/native_sources/ffmpeg-8.0.1"),
    ("av.libs/libunistring", "third_party/native_sources/libunistring-1.3"),
    ("av.libs/libvpl", "third_party/native_sources/libvpl-2.16.0"),
    ("av.libs/libvpx", "third_party/native_sources/libvpx-1.15.2"),
    ("av.libs/libwebp", "third_party/native_sources/libwebp-1.6.0"),
    ("av.libs/libx264", "third_party/native_sources/x264-stable-b35605ace3ddf7c1a5d67a2eb553f034aef41d55"),
    ("av.libs/libx265", "third_party/native_sources/x265-4.1"),
    ("av.libs/libxcb", "third_party/native_sources/libxcb-1.17.0"),
    ("dyNET38.libs/libdynet", "third_party/upstream_sources/dynet38-2.2"),
    ("llvmlite/binding/libllvmlite", "third_party/native_sources/llvm-project-22.1.0"),
    ("numpy.libs/libgfortran", "third_party/native_sources/gcc-runtime-10.2.0"),
    ("numpy.libs/libopenblas", "third_party/native_sources/openblas-0.3.23"),
    ("numpy.libs/libquadmath", "third_party/native_sources/gcc-runtime-10.2.0"),
    ("onnxruntime/capi/", "third_party/upstream_sources/onnxruntime-1.27.0"),
    ("pillow.libs/libXau", "third_party/native_sources/libxau-1.0.12"),
    ("pillow.libs/libavif", "third_party/native_sources/libavif-1.4.2"),
    ("pillow.libs/libbrotli", "third_party/native_sources/brotli-1.2.0"),
    ("pillow.libs/libfreetype", "third_party/native_sources/freetype-2.14.3"),
    ("pillow.libs/libharfbuzz", "third_party/native_sources/harfbuzz-12.1.0"),
    ("pillow.libs/libjpeg", "third_party/native_sources/libjpeg-turbo-3.1.2"),
    ("pillow.libs/liblcms2", "third_party/native_sources/little-cms-2.19"),
    ("pillow.libs/liblzma", "third_party/native_sources/xz-5.8.1"),
    ("pillow.libs/libopenjp2", "third_party/native_sources/openjpeg-2.5.4"),
    ("pillow.libs/libpng", "third_party/native_sources/libpng-1.6.50"),
    ("pillow.libs/libsharpyuv", "third_party/native_sources/libwebp-1.6.0"),
    ("pillow.libs/libtiff", "third_party/native_sources/libtiff-4.7.1"),
    ("pillow.libs/libwebp", "third_party/native_sources/libwebp-1.6.0"),
    ("pillow.libs/libxcb", "third_party/native_sources/libxcb-1.17.0"),
    ("pillow.libs/libzstd", "third_party/native_sources/zstd-1.5.7"),
    ("PyQt5/Qt5/", "third_party/upstream_sources/pyqt5-qt5-5.15.19"),
    ("scikit_learn.libs/libgomp", "third_party/native_sources/gcc-runtime-10.2.0"),
    ("scipy.libs/libgfortran", "third_party/native_sources/gcc-runtime-10.2.0"),
    ("scipy.libs/libquadmath", "third_party/native_sources/gcc-runtime-10.2.0"),
    ("scipy.libs/libscipy_openblas", "third_party/native_sources/openblas-0.3.23"),
    ("torch/lib/", "third_party/upstream_sources/torch-2.13.0+cpu"),
)


def normalize_name(value: str) -> str:
    return value.replace("_", "-").lower()


def load_json(path: Path) -> dict:
    with path.open(encoding="utf-8") as fp:
        return json.load(fp)


def installed_distributions() -> dict[tuple[str, str], metadata.Distribution]:
    installed: dict[tuple[str, str], metadata.Distribution] = {}
    for dist in metadata.distributions():
        name = dist.metadata.get("Name")
        if name:
            installed[(normalize_name(name), dist.version)] = dist
    return installed


def suffixes(path: Path | str) -> set[str]:
    return {suffix.lower() for suffix in Path(str(path)).suffixes}


def binary_artifacts_under(path: Path) -> list[str]:
    if not path.exists():
        return []
    found: list[str] = []
    for item in sorted(path.rglob("*")):
        if item.is_file() and suffixes(item).intersection(BINARY_ARTIFACT_SUFFIXES):
            found.append(str(item.relative_to(ROOT)))
    return found


def site_package_roots() -> list[Path]:
    roots = {
        Path(sysconfig.get_paths()[key]).resolve()
        for key in ("purelib", "platlib")
        if key in sysconfig.get_paths()
    }
    return sorted(root for root in roots if root.exists())


def relative_to_site_packages(path: Path, roots: list[Path]) -> str | None:
    resolved = path.resolve()
    for root in roots:
        try:
            return resolved.relative_to(root).as_posix()
        except ValueError:
            continue
    return None


def runtime_native_binaries(roots: list[Path]) -> list[str]:
    found: list[str] = []
    for root in roots:
        for item in sorted(root.rglob("*")):
            if not item.is_file() or not suffixes(item).intersection(RUNTIME_NATIVE_SUFFIXES):
                continue
            rel = relative_to_site_packages(item, roots)
            if rel is not None:
                found.append(rel)
    return sorted(set(found))


def distribution_native_binaries(dist: metadata.Distribution) -> list[str]:
    binaries: list[str] = []
    for file in dist.files or []:
        if suffixes(file).intersection(RUNTIME_NATIVE_SUFFIXES):
            binaries.append(str(file))
    return binaries


def assert_source_dir(path: str, errors: list[str]) -> None:
    source_dir = ROOT / path
    if not source_dir.exists():
        errors.append(f"Missing source directory: {path}")


def audit_recursive_submodules(source_dir: Path, errors: list[str]) -> dict:
    gitmodules = source_dir / ".gitmodules"
    result = {"source_dir": str(source_dir.relative_to(ROOT)), "submodule_count": 0, "missing": [], "empty": []}
    if not gitmodules.exists():
        return result

    parser = configparser.ConfigParser()
    parser.read(gitmodules)
    result["submodule_count"] = len(parser.sections())
    for section in parser.sections():
        rel_path = parser[section].get("path")
        if not rel_path:
            continue
        path = source_dir / rel_path
        if not path.exists():
            result["missing"].append(rel_path)
        elif not any(child.is_file() for child in path.rglob("*")):
            result["empty"].append(rel_path)

    for rel_path in result["missing"]:
        errors.append(f"Missing recursive submodule source: {source_dir.relative_to(ROOT)}/{rel_path}")
    for rel_path in result["empty"]:
        errors.append(f"Empty recursive submodule source: {source_dir.relative_to(ROOT)}/{rel_path}")
    return result


def coverage_for_foreign_binary(path: str) -> str | None:
    for prefix, source in FOREIGN_NATIVE_COVERAGE:
        if path.startswith(prefix):
            return source
    return None


def main() -> int:
    errors: list[str] = []
    python_manifest = load_json(PYTHON_MANIFEST)
    native_manifest = load_json(NATIVE_MANIFEST)

    installed = installed_distributions()
    package_records = {
        (normalize_name(record["name"]), record["version"]): record
        for record in python_manifest.get("packages", [])
    }

    missing_from_manifest = sorted(set(installed) - set(package_records))
    extra_in_manifest = sorted(set(package_records) - set(installed))
    for name, version in missing_from_manifest:
        errors.append(f"Installed package missing from Python source manifest: {name}=={version}")
    for name, version in extra_in_manifest:
        errors.append(f"Python source manifest package is not installed: {name}=={version}")

    missing_source_dirs: list[str] = []
    missing_upstream_fallbacks: list[str] = []
    recursive_submodules: list[dict] = []
    for key, record in sorted(package_records.items()):
        source_dir = record.get("source_dir")
        if record.get("status") == "missing-sdist":
            upstream_dir = record.get("upstream_source_dir")
            if not upstream_dir or "upstream_status" not in record:
                missing_upstream_fallbacks.append(f"{key[0]}=={key[1]}")
                continue
            assert_source_dir(upstream_dir, errors)
            if record.get("upstream_method") == "git-recursive":
                marker = ROOT / upstream_dir / ".vendor-source.json"
                if not marker.exists():
                    errors.append(f"Missing git-recursive vendor marker: {upstream_dir}")
                recursive_submodules.append(audit_recursive_submodules(ROOT / upstream_dir, errors))
        elif source_dir:
            if not (ROOT / source_dir).exists():
                missing_source_dirs.append(f"{key[0]}=={key[1]} -> {source_dir}")

    for item in missing_source_dirs:
        errors.append(f"Missing Python source directory: {item}")
    for item in missing_upstream_fallbacks:
        errors.append(f"Missing upstream fallback for no-sdist package: {item}")

    native_source_dirs = [
        record.get("source_dir")
        for record in native_manifest.get("native_sources", [])
        if record.get("source_dir")
    ]
    cargo_source_dirs = [
        record.get("source_dir")
        for record in native_manifest.get("cargo_vendors", [])
        if record.get("status") != "missing" and record.get("source_dir")
    ]
    for path in native_source_dirs + cargo_source_dirs:
        assert_source_dir(path, errors)

    third_party_binary_artifacts = binary_artifacts_under(THIRD_PARTY)
    for path in third_party_binary_artifacts[:20]:
        errors.append(f"Binary artifact remains under third_party: {path}")
    if len(third_party_binary_artifacts) > 20:
        errors.append(f"Additional binary artifacts under third_party: {len(third_party_binary_artifacts) - 20}")

    git_metadata_dirs = [str(path.relative_to(ROOT)) for path in THIRD_PARTY.rglob(".git") if path.is_dir()]
    for path in git_metadata_dirs:
        errors.append(f"Git metadata directory remains under third_party: {path}")

    native_binary_packages: dict[str, int] = {}
    for key, dist in installed.items():
        binaries = distribution_native_binaries(dist)
        if binaries:
            native_binary_packages[f"{key[0]}=={key[1]}"] = len(binaries)
            if key not in package_records:
                errors.append(f"Native extension package has no source record: {key[0]}=={key[1]}")

    roots = site_package_roots()
    runtime_binaries = runtime_native_binaries(roots)
    foreign_runtime_binaries = [
        path for path in runtime_binaries if path.startswith(FOREIGN_NATIVE_PREFIXES)
    ]
    uncovered_foreign_binaries: list[str] = []
    covered_foreign_binaries: dict[str, str] = {}
    for path in foreign_runtime_binaries:
        source = coverage_for_foreign_binary(path)
        if source is None:
            uncovered_foreign_binaries.append(path)
            continue
        covered_foreign_binaries[path] = source
        assert_source_dir(source, errors)
    for path in uncovered_foreign_binaries:
        errors.append(f"Foreign native runtime binary has no mapped source: {path}")

    report = {
        "installed_package_count": len(installed),
        "python_source_package_count": len(package_records),
        "native_binary_package_count": len(native_binary_packages),
        "runtime_native_binary_count": len(runtime_binaries),
        "foreign_runtime_native_binary_count": len(foreign_runtime_binaries),
        "covered_foreign_runtime_native_binary_count": len(covered_foreign_binaries),
        "third_party_binary_artifact_count": len(third_party_binary_artifacts),
        "git_metadata_dir_count": len(git_metadata_dirs),
        "recursive_submodules": recursive_submodules,
        "errors": errors,
    }
    AUDIT_REPORT.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    if errors:
        print(json.dumps(report, indent=2))
        raise SystemExit(1)

    print(
        "Source audit passed: "
        f"{len(package_records)} Python packages, "
        f"{len(native_binary_packages)} native-extension packages, "
        f"{len(foreign_runtime_binaries)} foreign runtime native binaries, "
        f"{len(third_party_binary_artifacts)} third_party binary artifacts."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
