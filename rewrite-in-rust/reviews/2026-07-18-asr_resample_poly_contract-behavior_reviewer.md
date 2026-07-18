# asr_resample_poly_contract - behavior_reviewer

PASS

Date: 2026-07-18
Decision: pass

## Findings

No blocking findings.

- Severity: none
- Location: rewrite-in-rust/rust/crates/v2m-core/src/asr_resample_poly.rs:32
- Issue: The Rust public seam accepts the selected fixture-bound inputs and preserves the selected SciPy validation order.
- Evidence: `resample_poly_1d_float32(input, target_rate, source_rate)` takes pre-decoded 1D `f32` samples and integer rates at `rust/crates/v2m-core/src/asr_resample_poly.rs:32`. It rejects non-positive rates before gcd reduction, identity, or empty-input handling at `rust/crates/v2m-core/src/asr_resample_poly.rs:37`. This matches SciPy 1.17.1, which validates integer/positive `up` and `down` at `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:3993` before the identity copy at `_signaltools.py:4010`. Fixture lines 17 and 18 cover equal invalid rates.
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/rust/crates/v2m-core/src/asr_resample_poly.rs:52
- Issue: Output shape, selected float32 values, NaN/Inf projection, empty input, and invalid-rate errors match the recorded contract.
- Evidence: The Rust test embeds `fixtures/asr_resample_poly_contract.jsonl` at `rust/crates/v2m-core/src/asr_resample_poly.rs:228`, runs every case through `resample_poly_1d_float32` at `rust/crates/v2m-core/src/asr_resample_poly.rs:239`, compares output length at `rust/crates/v2m-core/src/asr_resample_poly.rs:244`, compares finite samples with `2e-7` tolerance at `rust/crates/v2m-core/src/asr_resample_poly.rs:229`, compares projected sums with `1e-5` tolerance at `rust/crates/v2m-core/src/asr_resample_poly.rs:230`, and preserves `nan`, `inf`, and `-inf` projections in `assert_float_close` at `rust/crates/v2m-core/src/asr_resample_poly.rs:339`. `cargo test --manifest-path rust/Cargo.toml asr_resample_poly_contract` passed all 18 fixture cases.
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:1
- Issue: The fixture set covers the behavior categories requested for this seam.
- Evidence: Fixture inspection found 18 cases: identity, 44100 -> 16000, 48000 -> 16000, 22050 -> 16000, 32000 -> 16000 GCD reduction, 8000 -> 16000 upsample, single-sample, empty input, NaN/Inf, four long steady-state dual-sine cases, and five invalid-rate errors. The long cases assert shape plus 16 or 17 selected head/mid/tail samples and finite sum projections. The Python checker calls SciPy for every fixture case at `bootstrap/check_asr_resample_poly_contract.py:52`, and `uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py` passed with `asr_resample_poly_contract fixtures ok: 18 cases`.
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/manifest.yaml:1814
- Issue: Rollback and non-promotion boundaries remain intact.
- Evidence: The manifest keeps `asr_resample_poly_contract` at `status: reimplemented`, `current_owner: legacy`, and `target_owner: rust` at `manifest.yaml:1814`, with rollback to Python SciPy calls at `manifest.yaml:1841`. `rg -n "resample_poly_1d_float32" /home/fuurin/code/Vocal2Midi-for-linux --glob '!rewrite-in-rust/target/**'` found only record 0115 and the Rust module/test, so no production Python caller or bridge is using the Rust helper. The legacy source refs still call SciPy directly at `../inference/qwen3asr_dml/utils.py:85` and `../inference/romaji_asr/common.py:28`.
- Required fix: none.

## Evidence

- `manifest.yaml:1814` keeps the unit `reimplemented`, inventory-confirmed, and legacy-owned.
- `dependencies/asr_resample_poly_contract.yaml:4` defines the narrow default 1D float32 SciPy `resample_poly` capability.
- `bootstrap/asr_resample_poly_contract.md:31` records that identity short-circuiting must happen after positive-rate validation.
- `records/0113-bootstrap-asr-resample-poly-contract.md:11` confirms the narrow SciPy compatibility unit.
- `records/0114-fix-asr-resample-poly-bootstrap-review-findings.md:117` records the fixture expansion and validation-order fixes.
- `records/0115-implement-asr-resample-poly-contract.md:157` records the Rust helper and no-runtime-route implementation boundary.
- `reviews/2026-07-18-asr_resample_poly_contract-dependency_bootstrap_reviewer-rerun.md:3` passed the dependency/bootstrap rerun and confirmed the unit boundary.

## Checks

- `cargo test --manifest-path rust/Cargo.toml asr_resample_poly_contract`: passed; 1 matching `v2m-core` test passed, 0 failed.
- `uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py` from `/home/fuurin/code/Vocal2Midi-for-linux`: passed; `asr_resample_poly_contract fixtures ok: 18 cases`.
- `uv run python - <<'PY' ... fixture inspection ... PY`: passed; confirmed 18 cases, long selected-value projections, and shared invalid-rate error message.
- `rg -n "resample_poly_1d_float32" /home/fuurin/code/Vocal2Midi-for-linux --glob '!rewrite-in-rust/target/**'`: passed; found only record 0115 plus the Rust module/test.

## Residual Risk

This behavior review proves fixture-backed parity at the selected public seam. It does not replace the required `data_algorithm_reviewer` gate for deeper FIR/upfirdn numeric reasoning. The long steady-state fixtures use selected values plus finite sums rather than full-output arrays; that is acceptable for this behavior gate because the implementation is an algorithmic Rust helper and the recorded fixture tolerance is met, but full-output hashes or denser projections would reduce future regression risk.

Very large positive rates or input lengths beyond the fixture-scale ASR use cases were not stress-tested for `usize` arithmetic overflow. Non-default SciPy options, multidimensional arrays, complex dtypes, file IO, and model/runtime behavior remain explicitly legacy-owned.

## Promotion Note

This behavior review does not block promotion. Coordinator recommendation: count this as the requested behavior/stage-behavior gate for `asr_resample_poly_contract`, keep the unit `reimplemented` and legacy-owned until the required `data_algorithm_reviewer` gate also passes, then the coordinator may update state according to the manifest workflow. Do not promote runtime ownership from this review alone.
