#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

export PYTHONPATH="$ROOT"
export PYTHONNOUSERSITE=1

cd "$ROOT"

if command -v uv >/dev/null 2>&1 && [ -z "${PYTHON_BIN:-}" ]; then
  exec uv run python app_fluent.py
fi

PYTHON_BIN="${PYTHON_BIN:-python3}"
exec "$PYTHON_BIN" app_fluent.py
