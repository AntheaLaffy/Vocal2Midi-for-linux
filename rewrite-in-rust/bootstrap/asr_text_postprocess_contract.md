# asr_text_postprocess_contract Bootstrap

## Boundary Decision

Re-cut `asr_text_postprocess_contract`. The original umbrella mixed six
different compatibility surfaces:

- Chinese inverse text normalization in
  `inference/qwen3asr_dml/chinese_itn.py`.
- Qwen language normalization/validation in
  `inference/qwen3asr_dml/utils.py`.
- Qwen ASR schema dataclass and enum defaults in
  `inference/qwen3asr_dml/schema.py`.
- Qwen WAV PCM fallback decoding in `utils.py`, including optional
  `scipy.signal.resample_poly`.
- Romaji vocab loading, CTC decode, logits/id dispatch, and chunking in
  `inference/romaji_asr/common.py`.
- Romaji fake-session metadata and batch padding/mask assembly in
  `common.py`.

These should not be assigned to one writer. They have different dependency
risks, fixture formats, and review needs. Keep Python as runtime owner and keep
the Rust seam as an independent library plus fixtures under `rewrite-in-rust/`.
Do not add PyO3, a subprocess bridge, an HTTP service, a Python router, or
production caller routing.

## Split Units

`asr_chinese_itn_core` is verified. It covers
`chinese_to_num` and its helper tables/patterns: ranges, pure digits, numeric
values, consecutive values, percentages, fractions, ratios, times, dates,
units, idiom/fuzzy no-ops, and fallback no-op behavior on conversion errors.
Rollback is keeping `inference.qwen3asr_dml.chinese_itn` as runtime owner.

`asr_qwen_language_schema_contract` is verified. It covers
`SUPPORTED_LANGUAGES`, `normalize_language_name`, `validate_language`,
`MsgType`, `StreamingMessage`, `DecodeResult`, `ASREngineConfig`, and
`TranscribeResult`. Rollback is keeping the Python utils/schema classes as
runtime owner.

`asr_qwen_wav_pcm_decode_core` is verified for same-rate WAV PCM fallback. It
covers Python `wave` fallback behavior for unsigned 8-bit, signed 16-bit, signed
24-bit little-endian, signed 32-bit, unsupported sample widths, multichannel
mean, float32 output, and final slicing. Resampling remains separate.

`asr_resample_poly_contract` is verified as the narrow default-path
`scipy.signal.resample_poly` compatibility layer needed by the ASR audio helper
contracts.

`asr_romaji_vocab_ctc_decode_core` is verified. It covers
`load_vocab`, `decode_pred_ids`, `decode_logits`, `decode_outputs`, and
`chunked`. It excludes file audio loading, ONNX session creation, provider
selection, and model execution.

`asr_romaji_batch_metadata_contract` is verified. It covers
`get_fixed_batch_size`, `get_fixed_num_samples`, `ort_type_to_numpy_dtype`, and
`prepare_batch` using fake session/input metadata. It should not create
`onnxruntime.InferenceSession`; the verified fixtures use synthetic waveforms
and keep file loading/resampling legacy-owned.

## Dependency Evidence

The selected source refs import only local Python helpers plus standard
library modules, `numpy`, `scipy.signal`, `pydub`, `soundfile`, and
`onnxruntime` at first layer. `pyproject.toml`, `requirements*.txt`, and
`uv.lock` declare the broader environment, including `librosa`, `qwen-asr`,
`sentencepiece`, `torch`, and `transformers`, but the confirmed helper seams do
not call those model/runtime packages.

`third_party/sources/manifest.json` indexes first-layer source records for
`numpy-1.26.4`, `scipy-1.17.1`, `soundfile-0.14.0`, `pydub-0.25.1`,
`librosa-0.11.0`, `qwen-asr-0.0.6`, `sentencepiece-0.2.2`, and
`transformers-4.57.6`. `third_party/sources/MISSING_SOURCES.md` records
upstream fallbacks for `onnxruntime-1.27.0` and `torch-2.13.0+cpu`.
`third_party/source_audit.json` reports no audit errors.

No second-layer source expansion is justified for this bootstrap. A lockfile
edge alone is not evidence. Deeper SciPy/native work is deferred until
`asr_resample_poly_contract` proves an exact public-output need.

## Crate Decisions

Use `serde_json` for romaji vocab JSON parsing. Add a compatibility adapter for
Python `int(v)` conversion, id-to-token inversion, and `<blank>`/`PAD`/`0`
fallback behavior.

Use a maintained WAV parser for `asr_qwen_wav_pcm_decode_core` only if it
preserves Python `wave.getsampwidth()` byte-container semantics before rejecting
non-byte-aligned headers. Otherwise hand-write the narrow RIFF/WAVE PCM parser,
then hand-write the Python-compatible sample normalization and channel
averaging. The capability is WAV PCM parsing only, not arbitrary audio codecs.

`ndarray` is already allowed in this Rust workspace and should be decided per
child unit. Defer it for Chinese ITN, Qwen language/schema, and Qwen WAV PCM
decode because those are strings/DTOs/linear samples. It may be used for
romaji logits and batch metadata when an array-shaped API reduces ambiguity.
Do not treat it as broad NumPy compatibility.

Reject general resampler crate reuse for exact parity until
`asr_resample_poly_contract` defines tolerances against
`scipy.signal.resample_poly` fixtures.

## Fixture Strategy

Generate Python golden fixtures before Rust implementation. Keep fixtures small
and deterministic:

- Chinese ITN JSONL: each case stores input text, output text, and category.
- Qwen language/schema JSONL: each case stores constructor/input, output or
  error type/message, and schema default snapshot.
- Qwen WAV PCM fixtures: generate tiny WAV files or serialized sample vectors
  plus expected float32 samples after fallback decode/channel mean/slicing.
- Resample fixtures: serialized float vectors with exact sample-rate pairs,
  output length, expected float samples, and explicit tolerance.
- Romaji vocab/CTC fixtures: vocab JSON, predicted ids/logits, blank id,
  decoded tokens, and chunked outputs.
- Romaji batch metadata fixtures: fake input metadata, synthetic waveforms,
  expected input_values/attention_mask shapes and values, used lengths, dtype
  mapping, and error messages.

## Split Closure

The split umbrella is closed after all six child units reached `verified`:

- `asr_chinese_itn_core`
- `asr_qwen_language_schema_contract`
- `asr_qwen_wav_pcm_decode_core`
- `asr_resample_poly_contract`
- `asr_romaji_vocab_ctc_decode_core`
- `asr_romaji_batch_metadata_contract`

The umbrella itself still must not be assigned to a writer, and it does not
promote Rust into production callers.

## Kept Legacy

Keep pydub primary loading, soundfile/libsndfile arbitrary file IO, librosa,
ONNX Runtime sessions, provider resolution, Qwen encoder/decoder sessions,
llama subprocess inference, torch, transformers, sentencepiece, qwen-asr model
execution, and model weights legacy-owned.

## Verification

Recommended bootstrap checks from the repository root:

```bash
uv run python -m py_compile inference/qwen3asr_dml/chinese_itn.py inference/qwen3asr_dml/utils.py inference/qwen3asr_dml/schema.py inference/romaji_asr/common.py
uv run python scripts/audit_vendored_sources.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_language_schema_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_qwen_wav_pcm_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_resample_poly_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_vocab_ctc_decode_core
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract
uv run python - <<'PY'
import pathlib, yaml
for path in [
    'rewrite-in-rust/manifest.yaml',
    'rewrite-in-rust/dependencies/asr_text_postprocess_contract.yaml',
]:
    with pathlib.Path(path).open('r', encoding='utf-8') as f:
        yaml.safe_load(f)
print('yaml ok')
PY
```
