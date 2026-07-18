# 0124 - Close ASR Romaji Batch Metadata Gate

Date: 2026-07-18

## Unit

`asr_romaji_batch_metadata_contract`

## Decision

Mark `asr_romaji_batch_metadata_contract` as `verified`.

Python remains the runtime owner. This gate verifies only the independent Rust
library seam and fixture parity for fake-session metadata and synthetic-waveform
batch preparation.

## Review Evidence

Required reviews are complete:

- `reviews/2026-07-18-asr_romaji_batch_metadata_contract-dependency_bootstrap_reviewer.md`
- `reviews/2026-07-18-asr_romaji_batch_metadata_contract-behavior_reviewer.md`
- `reviews/2026-07-18-asr_romaji_batch_metadata_contract-error_tracing_reviewer-rerun.md`

Initial error/tracing review failed on negative fixed sample dimensions. Behavior
and dependency/bootstrap reviews also noted missing synthetic waveform key and
`attention_mask` float16 follow-ups. Record 0123 fixed all three with
Python-generated golden fixtures and Rust error projection changes.

## Fixture Evidence

`fixtures/asr_romaji_batch_metadata_contract.jsonl` contains 26 cases covering:

- fixed, dynamic, bool, one-dimensional, and negative metadata shapes
- case-sensitive ONNX dtype string mapping
- dynamic, fixed, zero-fixed fallback, and negative fixed sample lengths
- empty `audio_paths` and fixed batch-size mismatch before loading
- missing synthetic waveform keys and missing `input_values` after loading
- truncation, padding, zero-length waveform handling, used lengths, and sample
  rate passthrough
- `input_values` dtype casts and `attention_mask` dtype casts including float16

## Verification

Final commands:

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
uv run python -m py_compile inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
```

Additional checks:

```bash
uv run python - <<'PY'
from pathlib import Path
import yaml
for path in [Path('rewrite-in-rust/manifest.yaml'), Path('rewrite-in-rust/dependencies/asr_romaji_batch_metadata_contract.yaml')]:
    yaml.safe_load(path.read_text(encoding='utf-8'))
PY
git -C rewrite-in-rust diff --check
cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal | rg -n "ndarray|half|rodio|symphonia|hound" || true
```

The targeted `cargo tree` check shows direct `half` and `ndarray` usage and no
`rodio`, Symphonia, or `hound` dependency in this crate.

## Rollback

Keep `inference.romaji_asr.common` metadata and `prepare_batch` helpers as the
runtime owners.
