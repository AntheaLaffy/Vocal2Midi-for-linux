#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_BIN="${PYTHON_BIN:-python3}"

export PYTHONPATH="$ROOT"
export PYTHONNOUSERSITE=1

cd "$ROOT"
exec "$PYTHON_BIN" app_fluent.py
