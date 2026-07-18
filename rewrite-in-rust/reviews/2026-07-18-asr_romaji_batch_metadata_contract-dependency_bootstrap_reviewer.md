# asr_romaji_batch_metadata_contract - dependency_bootstrap_reviewer

Date: 2026-07-18

## Findings

- Severity: low
- Location: `rewrite-in-rust/fixtures/asr_romaji_batch_metadata_contract.jsonl:17`, `rewrite-in-rust/rust/crates/v2m-core/src/asr_romaji_batch_metadata.rs:263`
- Issue: The dependency record justifies `half` for float16 feed projection, and the Rust adapter projects both `input_values` and `attention_mask` through `half::f16` when metadata asks for `tensor(float16)`. The fixture set covers `input_values` float16 and `attention_mask` int32/float32, but does not include an `attention_mask` float16 prepare-batch case.
- Evidence: Fixture line 17 covers `input_values` float16 with mask int32; line 20 covers mask float32. The Rust mask cast branch at `cast_attention_mask` includes `NumpyDType::Float16`. `dependencies/asr_romaji_batch_metadata_contract.yaml:33` records `half` as the float16 crate.
- Required fix: Add a promotion or next-follow-up fixture for `attention_mask` `tensor(float16)` if that dtype appears in real ONNX metadata or if this helper is exposed beyond the current fake-session fixture seam.

No critical, high, or medium dependency/bootstrap findings.

## Decision

Decision: pass-with-followups

Manifest unit boundary: confirmed.

This unit should remain independent. It owns fake-session metadata extraction, ONNX dtype-string mapping, and synthetic-waveform batch padding/mask assembly only. It should not be split, merged, deferred, or replaced based on the dependency evidence reviewed here.

## Scope Evidence

- Capability coverage is adequate for the selected seam. `inference/romaji_asr/common.py:52` through `inference/romaji_asr/common.py:120` shows the relevant legacy surface: shape metadata, case-sensitive dtype substring mapping, `load_audio` calls, NumPy zero arrays, padding/truncation, optional mask, and dtype casts.
- Kept-legacy decisions are appropriate. `inference/romaji_asr/common.py:23` through `inference/romaji_asr/common.py:49` keeps audio IO, resampling, provider selection, and ONNX Runtime session creation outside this unit. `bootstrap/asr_romaji_batch_metadata_contract.md:9` through `bootstrap/asr_romaji_batch_metadata_contract.md:24` states the same boundary.
- Crate reuse is narrow and justified. `ndarray 0.17.2` is used as direct `v2m-core` storage for 2D feed matrices, while metadata policy and Python-specific dtype/error ordering are hand-written. `half 2.7.1` is a direct `v2m-core` dependency and is limited to float16 value projection.
- First-layer Python source coverage is indexed. `third_party/sources/manifest.json` records `numpy-1.26.4`, `scipy-1.17.1`, and `soundfile-0.14.0`; `third_party/sources/MISSING_SOURCES.md` records the ONNX Runtime upstream fallback. The local source directories for numpy, scipy, and soundfile exist.
- Targeted transitive expansion was reasonably rejected. The public seam starts after fake metadata and synthetic waveform loading, so ONNX Runtime internals, libsndfile, SciPy resampling, and NumPy allocator/stride internals are not needed for this dependency gate.
- The compatibility adapter choice is reasonable. `prepare_batch_from_waveforms` records load calls and accepts synthetic waveforms, preserving the observable load-order/sample-rate part of `prepare_batch` without taking ownership of real file IO.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_romaji_batch_metadata_contract.py`: passed, 23 fixture cases.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_romaji_batch_metadata_contract -- --nocapture`: passed, 1 matching Rust fixture test.
- `uv run python - <<'PY' ... yaml.safe_load(...)`: passed for `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/asr_romaji_batch_metadata_contract.yaml`.
- `uv run python -m py_compile inference/romaji_asr/common.py`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed, source audit reports 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, and 0 third_party binary artifacts.
- `cargo tree --manifest-path rust/Cargo.toml -p v2m-core -i ndarray`: `ndarray v0.17.2` is a direct `v2m-core` dependency.
- `cargo tree --manifest-path rust/Cargo.toml -p v2m-core -i half`: `half v2.7.1` is a direct `v2m-core` dependency.

## Residual Risk

- This review does not prove real ONNX Runtime metadata objects, real audio file loading, resampling, or model execution. Those remain intentionally legacy-owned.
- The fixture seam assumes complete synthetic waveform maps. A missing synthetic path is not production behavior evidence; production `load_audio` remains outside this unit.
- Float16 projection is fixture-backed for `input_values`; `attention_mask` float16 remains the only low-risk fixture gap noted above.

## Promotion Note

This dependency/bootstrap role does not block coordinator state update. The unit still needs its behavior/stage-behavior and error/tracing reviews before it can be marked verified.
