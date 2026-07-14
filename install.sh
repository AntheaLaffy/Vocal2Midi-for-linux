#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cd "$ROOT"

if command -v uv >/dev/null 2>&1 && [ -z "${PYTHON_BIN:-}" ] && [ -z "${PIP_BIN:-}" ]; then
  uv python pin 3.12
  uv sync
  if [ "${VENDOR_SOURCES:-0}" = "1" ]; then
    uv run python scripts/vendor_sources.py
    uv run python scripts/vendor_native_sources.py
    uv run python scripts/audit_vendored_sources.py
  fi
  exit 0
fi

PYTHON_BIN="${PYTHON_BIN:-python3}"
PIP_BIN="${PIP_BIN:-pip}"

if [ ! -x "$PYTHON_BIN" ] && ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "[ERROR] python3 was not found."
  exit 1
fi

if ! command -v "$PIP_BIN" >/dev/null 2>&1; then
  echo "[ERROR] pip was not found."
  exit 1
fi

"$PIP_BIN" install --upgrade pip setuptools wheel
"$PIP_BIN" install -r requirements-linux.txt
