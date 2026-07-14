#!/usr/bin/env python3
"""Vendor native/FFI source trees used by Python dependencies."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import tarfile
import tempfile
import urllib.request
import zipfile
from pathlib import Path
from urllib.parse import urlparse


ROOT = Path(__file__).resolve().parents[1]
NATIVE_OUTPUT_DIR = ROOT / "third_party" / "native_sources"
CARGO_OUTPUT_DIR = ROOT / "third_party" / "cargo_vendor"

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


NATIVE_ARCHIVES = [
    {
        "name": "ffmpeg",
        "version": "8.0.1",
        "url": "https://ffmpeg.org/releases/ffmpeg-8.0.1.tar.xz",
        "covers": ["av.libs/libavcodec", "av.libs/libavformat", "av.libs/libavutil"],
    },
    {
        "name": "svt-av1",
        "version": "4.1.0",
        "url": "https://gitlab.com/AOMediaCodec/SVT-AV1/-/archive/v4.1.0/SVT-AV1-v4.1.0.tar.gz",
        "covers": ["av.libs/libSvtAv1Enc"],
    },
    {
        "name": "libvpx",
        "version": "1.15.2",
        "url": "https://github.com/webmproject/libvpx/archive/refs/tags/v1.15.2.tar.gz",
        "covers": ["av.libs/libvpx"],
    },
    {
        "name": "libwebp",
        "version": "1.6.0",
        "url": "https://github.com/webmproject/libwebp/archive/refs/tags/v1.6.0.tar.gz",
        "covers": ["av.libs/libwebp", "pillow.libs/libwebp", "pillow.libs/libsharpyuv"],
    },
    {
        "name": "dav1d",
        "version": "1.5.1",
        "url": "https://code.videolan.org/videolan/dav1d/-/archive/1.5.1/dav1d-1.5.1.tar.gz",
        "covers": ["av.libs/libdav1d"],
    },
    {
        "name": "opencore-amr",
        "version": "0.1.6",
        "url": "https://downloads.sourceforge.net/project/opencore-amr/opencore-amr/opencore-amr-0.1.6.tar.gz",
        "covers": ["av.libs/libopencore-amrnb", "av.libs/libopencore-amrwb"],
    },
    {
        "name": "lame",
        "version": "3.100",
        "url": "https://downloads.sourceforge.net/project/lame/lame/3.100/lame-3.100.tar.gz",
        "covers": ["av.libs/libmp3lame"],
    },
    {
        "name": "opus",
        "version": "1.5.2",
        "url": "https://github.com/xiph/opus/archive/refs/tags/v1.5.2.tar.gz",
        "covers": ["av.libs/libopus"],
    },
    {
        "name": "x264",
        "version": "stable-b35605ace3ddf7c1a5d67a2eb553f034aef41d55",
        "url": "https://code.videolan.org/videolan/x264/-/archive/b35605ace3ddf7c1a5d67a2eb553f034aef41d55/x264-b35605ace3ddf7c1a5d67a2eb553f034aef41d55.tar.gz",
        "covers": ["av.libs/libx264"],
    },
    {
        "name": "x265",
        "version": "4.1",
        "url": "https://bitbucket.org/multicoreware/x265_git/get/4.1.tar.gz",
        "covers": ["av.libs/libx265"],
    },
    {
        "name": "libvpl",
        "version": "2.16.0",
        "url": "https://github.com/intel/libvpl/archive/refs/tags/v2.16.0.tar.gz",
        "covers": ["av.libs/libvpl"],
    },
    {
        "name": "libdrm",
        "version": "2.4.125",
        "url": "https://dri.freedesktop.org/libdrm/libdrm-2.4.125.tar.xz",
        "covers": ["av.libs/libdrm"],
    },
    {
        "name": "libxcb",
        "version": "1.17.0",
        "url": "https://xorg.freedesktop.org/archive/individual/lib/libxcb-1.17.0.tar.xz",
        "covers": ["av.libs/libxcb", "pillow.libs/libxcb"],
    },
    {
        "name": "xcb-proto",
        "version": "1.17.0",
        "url": "https://xorg.freedesktop.org/archive/individual/proto/xcb-proto-1.17.0.tar.xz",
        "covers": ["libxcb build input"],
    },
    {
        "name": "libxau",
        "version": "1.0.12",
        "url": "https://xorg.freedesktop.org/archive/individual/lib/libXau-1.0.12.tar.xz",
        "covers": ["av.libs/libXau", "pillow.libs/libXau"],
    },
    {
        "name": "alsa-lib",
        "version": "1.2.14",
        "url": "https://github.com/alsa-project/alsa-lib/archive/refs/tags/v1.2.14.tar.gz",
        "covers": ["av.libs/libasound"],
    },
    {
        "name": "gnutls",
        "version": "3.8.10",
        "url": "https://www.gnupg.org/ftp/gcrypt/gnutls/v3.8/gnutls-3.8.10.tar.xz",
        "covers": ["av.libs/libgnutls"],
    },
    {
        "name": "nettle",
        "version": "3.10.2",
        "url": "https://ftp.gnu.org/gnu/nettle/nettle-3.10.2.tar.gz",
        "covers": ["av.libs/libnettle", "av.libs/libhogweed"],
    },
    {
        "name": "gmp",
        "version": "6.3.0",
        "url": "https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz",
        "covers": ["av.libs/libgmp"],
    },
    {
        "name": "libunistring",
        "version": "1.3",
        "url": "https://ftp.gnu.org/gnu/libunistring/libunistring-1.3.tar.xz",
        "covers": ["av.libs/libunistring"],
    },
    {
        "name": "libavif",
        "version": "1.4.2",
        "url": "https://github.com/AOMediaCodec/libavif/archive/refs/tags/v1.4.2.tar.gz",
        "covers": ["pillow.libs/libavif"],
    },
    {
        "name": "brotli",
        "version": "1.2.0",
        "url": "https://github.com/google/brotli/archive/refs/tags/v1.2.0.tar.gz",
        "covers": ["pillow.libs/libbrotlicommon", "pillow.libs/libbrotlidec"],
    },
    {
        "name": "freetype",
        "version": "2.14.3",
        "url": "https://download.savannah.gnu.org/releases/freetype/freetype-2.14.3.tar.xz",
        "covers": ["pillow.libs/libfreetype"],
    },
    {
        "name": "harfbuzz",
        "version": "12.1.0",
        "url": "https://github.com/harfbuzz/harfbuzz/archive/refs/tags/12.1.0.tar.gz",
        "covers": ["pillow.libs/libharfbuzz"],
    },
    {
        "name": "libjpeg-turbo",
        "version": "3.1.2",
        "url": "https://github.com/libjpeg-turbo/libjpeg-turbo/archive/refs/tags/3.1.2.tar.gz",
        "covers": ["pillow.libs/libjpeg"],
    },
    {
        "name": "little-cms",
        "version": "2.19",
        "url": "https://github.com/mm2/Little-CMS/archive/refs/tags/lcms2.19.tar.gz",
        "covers": ["pillow.libs/liblcms2"],
    },
    {
        "name": "xz",
        "version": "5.8.1",
        "url": "https://github.com/tukaani-project/xz/releases/download/v5.8.1/xz-5.8.1.tar.xz",
        "covers": ["pillow.libs/liblzma"],
    },
    {
        "name": "openjpeg",
        "version": "2.5.4",
        "url": "https://github.com/uclouvain/openjpeg/archive/refs/tags/v2.5.4.tar.gz",
        "covers": ["pillow.libs/libopenjp2"],
    },
    {
        "name": "libpng",
        "version": "1.6.50",
        "url": "https://download.sourceforge.net/libpng/libpng-1.6.50.tar.xz",
        "covers": ["pillow.libs/libpng16"],
    },
    {
        "name": "libtiff",
        "version": "4.7.1",
        "url": "https://download.osgeo.org/libtiff/tiff-4.7.1.tar.xz",
        "covers": ["pillow.libs/libtiff"],
    },
    {
        "name": "zstd",
        "version": "1.5.7",
        "url": "https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-1.5.7.tar.gz",
        "covers": ["pillow.libs/libzstd"],
    },
    {
        "name": "zlib",
        "version": "1.3.2",
        "url": "https://zlib.net/zlib-1.3.2.tar.gz",
        "covers": ["pillow zlib feature", "system-linked zlib dependency"],
    },
    {
        "name": "raqm",
        "version": "0.10.5",
        "url": "https://github.com/HOST-Oman/libraqm/releases/download/v0.10.5/raqm-0.10.5.tar.xz",
        "covers": ["pillow raqm feature"],
    },
    {
        "name": "libsndfile",
        "version": "1.2.2",
        "url": "https://github.com/libsndfile/libsndfile/archive/refs/tags/1.2.2.tar.gz",
        "covers": ["soundfile _soundfile_data/libsndfile_x86_64.so"],
    },
    {
        "name": "flac",
        "version": "1.5.0",
        "url": "https://github.com/xiph/flac/archive/refs/tags/1.5.0.tar.gz",
        "covers": ["libsndfile codec dependency"],
    },
    {
        "name": "libogg",
        "version": "1.3.6",
        "url": "https://github.com/xiph/ogg/archive/refs/tags/v1.3.6.tar.gz",
        "covers": ["libsndfile/opus/vorbis codec dependency"],
    },
    {
        "name": "libvorbis",
        "version": "1.3.7",
        "url": "https://github.com/xiph/vorbis/archive/refs/tags/v1.3.7.tar.gz",
        "covers": ["libsndfile codec dependency"],
    },
    {
        "name": "openblas",
        "version": "0.3.23",
        "url": "https://github.com/OpenMathLib/OpenBLAS/archive/refs/tags/v0.3.23.tar.gz",
        "covers": ["numpy.libs/libopenblas", "scipy.libs/libscipy_openblas"],
    },
    {
        "name": "gcc-runtime",
        "version": "10.2.0",
        "url": "https://ftp.gnu.org/gnu/gcc/gcc-10.2.0/gcc-10.2.0.tar.xz",
        "covers": ["libgfortran", "libquadmath", "libgomp", "libstdc++"],
    },
    {
        "name": "llvm-project",
        "version": "22.1.0",
        "url": "https://github.com/llvm/llvm-project/archive/refs/tags/llvmorg-22.1.0.tar.gz",
        "covers": ["llvmlite/binding/libllvmlite.so"],
    },
]


CARGO_MANIFESTS = [
    "third_party/sources/hf-xet-1.5.1/hf_xet/Cargo.toml",
    "third_party/sources/orjson-3.11.9/Cargo.toml",
    "third_party/sources/pydantic-core-2.46.4/Cargo.toml",
    "third_party/sources/rpds-py-2026.6.3/Cargo.toml",
    "third_party/sources/safetensors-0.8.0/bindings/python/Cargo.toml",
    "third_party/sources/tiktoken-0.13.0/Cargo.toml",
    "third_party/sources/tokenizers-0.22.2/bindings/python/Cargo.toml",
]


def safe_part(value: str) -> str:
    return "".join(ch if ch.isalnum() or ch in ".+-_" else "_" for ch in value)


def assert_safe_member(base: Path, member_name: str) -> None:
    target = (base / member_name).resolve()
    base_resolved = base.resolve()
    if os.path.commonpath([base_resolved, target]) != str(base_resolved):
        raise ValueError(f"Archive member escapes output directory: {member_name}")


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
    request = urllib.request.Request(url, headers={"User-Agent": "vocal2midi-native-source-vendor/1.0"})
    with urllib.request.urlopen(request, timeout=180) as response:
        with target.open("wb") as fp:
            shutil.copyfileobj(response, fp)


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


def vendor_archive(entry: dict, output_dir: Path, force: bool) -> dict:
    source_dir = output_dir / f"{safe_part(entry['name'])}-{safe_part(entry['version'])}"
    record = {
        "name": entry["name"],
        "version": entry["version"],
        "url": entry["url"],
        "source_dir": str(source_dir.relative_to(ROOT)),
        "covers": entry.get("covers", []),
    }
    if source_dir.exists() and not force:
        purged = remove_binary_artifacts(source_dir)
        record["status"] = "exists"
        if purged:
            record["purged_binary_artifacts"] = purged
        return record

    tmp_root = output_dir / ".tmp"
    tmp_root.mkdir(parents=True, exist_ok=True)
    archive_name = Path(urlparse(entry["url"]).path).name or f"{entry['name']}-{entry['version']}.src"
    with tempfile.TemporaryDirectory(prefix=f"{entry['name']}-", dir=tmp_root) as tmp_name:
        tmp_dir = Path(tmp_name)
        archive = tmp_dir / archive_name
        print(f"Downloading native source {entry['name']}=={entry['version']}")
        download(entry["url"], archive)
        unpacked = tmp_dir / "unpacked"
        extract_archive(archive, unpacked)
        move_unpacked_archive(unpacked, source_dir)

    purged = remove_binary_artifacts(source_dir)
    record["status"] = "downloaded"
    if purged:
        record["purged_binary_artifacts"] = purged
    return record


def has_lockfile(manifest_path: Path) -> bool:
    root = manifest_path.parent
    while root != ROOT and root != root.parent:
        if (root / "Cargo.lock").exists():
            return True
        if (root / "pyproject.toml").exists() and root != manifest_path.parent:
            break
        root = root.parent
    return (manifest_path.parent / "Cargo.lock").exists()


def vendor_cargo(manifest: Path, output_dir: Path, force: bool) -> dict:
    manifest = manifest.resolve()
    package_name = manifest.parent.name
    if package_name in {"python", "hf_xet"}:
        package_name = manifest.parent.parent.name if manifest.parent.parent.name else package_name
    source_dir = output_dir / safe_part(str(manifest.relative_to(ROOT)).replace("/", "__").replace("Cargo.toml", "crates"))
    record = {
        "manifest": str(manifest.relative_to(ROOT)),
        "source_dir": str(source_dir.relative_to(ROOT)),
        "locked": has_lockfile(manifest),
    }
    if source_dir.exists() and not force:
        purged = remove_binary_artifacts(source_dir)
        record["status"] = "exists"
        if purged:
            record["purged_binary_artifacts"] = purged
        return record

    if source_dir.exists():
        shutil.rmtree(source_dir)
    source_dir.parent.mkdir(parents=True, exist_ok=True)
    command = [
        "cargo",
        "vendor",
        "--manifest-path",
        str(manifest),
        "--versioned-dirs",
        str(source_dir),
    ]
    if record["locked"]:
        command.insert(2, "--locked")
    print(f"Vendoring Cargo crates for {manifest.relative_to(ROOT)}")
    result = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=True)
    (source_dir / "cargo-vendor-config.txt").write_text(result.stdout, encoding="utf-8")
    purged = remove_binary_artifacts(source_dir)
    record["status"] = "vendored"
    if purged:
        record["purged_binary_artifacts"] = purged
    return record


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--native-output-dir", type=Path, default=NATIVE_OUTPUT_DIR)
    parser.add_argument("--cargo-output-dir", type=Path, default=CARGO_OUTPUT_DIR)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    native_output = args.native_output_dir.resolve()
    cargo_output = args.cargo_output_dir.resolve()
    native_output.mkdir(parents=True, exist_ok=True)
    cargo_output.mkdir(parents=True, exist_ok=True)

    native_records = [vendor_archive(entry, native_output, args.force) for entry in NATIVE_ARCHIVES]
    tmp_dir = native_output / ".tmp"
    if tmp_dir.exists():
        shutil.rmtree(tmp_dir)

    cargo_records = []
    for manifest in CARGO_MANIFESTS:
        path = ROOT / manifest
        if path.exists():
            cargo_records.append(vendor_cargo(path, cargo_output, args.force))
        else:
            cargo_records.append({"manifest": manifest, "status": "missing"})

    remaining_binary_artifacts = find_binary_artifacts(native_output) + find_binary_artifacts(cargo_output)

    manifest = {
        "native_source_count": len(native_records),
        "cargo_vendor_count": len(cargo_records),
        "purged_binary_artifact_count": sum(
            len(record.get("purged_binary_artifacts", [])) for record in native_records + cargo_records
        ),
        "remaining_binary_artifact_count": len(remaining_binary_artifacts),
        "remaining_binary_artifacts": remaining_binary_artifacts,
        "native_sources": native_records,
        "cargo_vendors": cargo_records,
    }
    (native_output / "manifest.json").write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    if remaining_binary_artifacts:
        raise SystemExit(f"Binary artifacts remain in vendored native sources: {remaining_binary_artifacts[:20]}")
    print(
        f"Vendored {manifest['native_source_count']} native source archives, "
        f"{manifest['cargo_vendor_count']} Cargo vendor sets, "
        f"purged {manifest['purged_binary_artifact_count']} binary artifacts."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
