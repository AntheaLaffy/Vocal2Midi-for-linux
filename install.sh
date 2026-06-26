#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python3}"

cd "$ROOT"

if [ ! -x "$PYTHON_BIN" ] && ! command -v "$PYTHON_BIN" >/dev/null 2>&1; then
  echo "[ERROR] python3 was not found."
  exit 1
fi

PIP_BIN="${PIP_BIN:-pip}"

if ! command -v "$PIP_BIN" >/dev/null 2>&1; then
  echo "[ERROR] pip was not found."
  exit 1
fi

"$PIP_BIN" install --upgrade pip setuptools wheel
"$PIP_BIN" install -r requirements.txt
