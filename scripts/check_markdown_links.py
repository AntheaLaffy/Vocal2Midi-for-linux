"""Check repository Markdown links that resolve to local files or directories."""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path
from urllib.parse import unquote, urlsplit

REPOSITORY_ROOT = Path(__file__).resolve().parents[1]
EXTERNAL_SCHEMES = {"data", "ftp", "http", "https", "mailto"}
INLINE_DESTINATION = re.compile(
    r"!?\[[^\]]*\]\(\s*(?:<(?P<angle>[^>]+)>|(?P<plain>[^\s)]+))"
)
REFERENCE_DESTINATION = re.compile(
    r"^\s{0,3}\[[^\]]+\]:\s*(?:<(?P<angle>[^>]+)>|(?P<plain>\S+))"
)


def markdown_paths() -> list[Path]:
    """Return tracked and untracked non-ignored Markdown paths."""
    result = subprocess.run(
        [
            "git",
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "--",
            "*.md",
        ],
        cwd=REPOSITORY_ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    return [REPOSITORY_ROOT / line for line in result.stdout.splitlines() if line]


def local_destinations(markdown: str) -> list[tuple[int, str]]:
    """Extract local link and image destinations with source line numbers."""
    destinations: list[tuple[int, str]] = []
    fence: str | None = None
    for line_number, line in enumerate(markdown.splitlines(), start=1):
        stripped = line.lstrip()
        fence_marker = stripped[:3]
        if fence_marker in {"```", "~~~"}:
            if fence is None:
                fence = fence_marker
            elif fence == fence_marker:
                fence = None
            continue
        if fence is not None:
            continue

        matches = list(INLINE_DESTINATION.finditer(line))
        reference = REFERENCE_DESTINATION.match(line)
        if reference is not None:
            matches.append(reference)

        for match in matches:
            destination = match.group("angle") or match.group("plain")
            parsed = urlsplit(destination)
            if parsed.scheme.lower() in EXTERNAL_SCHEMES or parsed.netloc:
                continue
            if not parsed.path or parsed.path.startswith("/"):
                continue
            destinations.append((line_number, unquote(parsed.path)))
    return destinations


def main() -> int:
    """Report missing local link targets and return a process exit code."""
    failures: list[str] = []
    paths = markdown_paths()
    for source in paths:
        markdown = source.read_text(encoding="utf-8")
        for line, destination in local_destinations(markdown):
            target = (source.parent / destination).resolve()
            if not target.exists():
                relative_source = source.relative_to(REPOSITORY_ROOT)
                failures.append(f"{relative_source}:{line}: missing {destination}")

    if failures:
        print("\n".join(failures), file=sys.stderr)
        return 1

    print(f"checked {len(paths)} Markdown files")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
