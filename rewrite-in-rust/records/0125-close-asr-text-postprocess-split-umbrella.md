# 0125 - Close ASR Text Postprocess Split Umbrella

Date: 2026-07-18

## Unit

`asr_text_postprocess_contract`

## Decision

Mark the split umbrella as `verified`.

This is a control-plane closure, not a Rust writer unit and not a production
promotion. Record 0102 split the original umbrella before writer work because it
mixed text normalization, DTO/schema defaults, WAV PCM decode, SciPy
resampling, romaji CTC/vocab helpers, and ONNX-adjacent batch metadata. The
manifest explicitly says this umbrella should not be assigned to a writer.

## Child Evidence

All split child units are now verified:

- `asr_chinese_itn_core`: closed by
  `records/0107-close-asr-chinese-itn-gate.md`
- `asr_qwen_language_schema_contract`: closed by
  `records/0109-close-asr-qwen-language-schema-gate.md`
- `asr_qwen_wav_pcm_decode_core`: closed by
  `records/0112-close-asr-qwen-wav-pcm-decode-gate.md`
- `asr_resample_poly_contract`: closed by
  `records/0116-close-asr-resample-poly-gate.md`
- `asr_romaji_vocab_ctc_decode_core`: closed by
  `records/0120-close-asr-romaji-vocab-ctc-gate.md`
- `asr_romaji_batch_metadata_contract`: closed by
  `records/0124-close-asr-romaji-batch-metadata-gate.md`

The split keeps excluded runtime/model capabilities out of scope:

- `create_session`
- ONNX Runtime sessions and provider execution
- Qwen encoder/decoder workers
- llama subprocess inference
- arbitrary audio codec handling
- model weight parsing or GGUF/model runtime ownership

## Dependency Evidence

The dependency/bootstrap record remains the durable split decision. Current
source evidence still matches it:

- `pyproject.toml`, `requirements*.txt`, and `uv.lock` declare NumPy, SciPy,
  pydub, soundfile, ONNX Runtime, qwen-asr, sentencepiece, torch, and
  transformers in the wider environment.
- `third_party/sources/manifest.json` indexes first-layer source records for
  NumPy, SciPy, soundfile, pydub, librosa, qwen-asr, sentencepiece, and
  transformers.
- `third_party/sources/MISSING_SOURCES.md` records upstream fallbacks for
  ONNX Runtime and torch.
- No second-layer expansion is needed for this umbrella closure because each
  child unit owns its own public seam and review evidence.

## Verification

Run from the repository root:

```bash
uv run python -m py_compile inference/qwen3asr_dml/chinese_itn.py inference/qwen3asr_dml/utils.py inference/qwen3asr_dml/schema.py inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_language_schema_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_resample_poly_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_vocab_ctc_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract
```

Additional control-plane checks:

```bash
uv run python - <<'PY'
from pathlib import Path
import yaml
for path in [Path('rewrite-in-rust/manifest.yaml'), Path('rewrite-in-rust/dependencies/asr_text_postprocess_contract.yaml')]:
    yaml.safe_load(path.read_text(encoding='utf-8'))
PY
git -C rewrite-in-rust diff --check
```

## Rollback

Keep `qwen3asr_dml` and `romaji_asr` Python helpers as runtime owners. This
closure does not add a bridge or production caller route.
