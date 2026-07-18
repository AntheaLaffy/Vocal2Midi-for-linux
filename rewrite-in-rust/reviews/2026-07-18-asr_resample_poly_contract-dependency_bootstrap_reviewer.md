# asr_resample_poly_contract - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:2
- Issue: The successful resampling fixtures do not exercise SciPy's steady-state polyphase path, so the fixture set is not adequate for the claimed exact-parity writer handoff.
- Evidence: SciPy computes `half_len = 10 * max_rate` and then pads/trims around that filter in `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4024` and `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4036`. The current non-identity successful cases have input lengths 9, 8, 8, 6, 4, 1, 0, and 5 in `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:2`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:3`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:4`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:5`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:6`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:7`, `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:8`, and `rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:9`. For those same ratios, the reduced SciPy default FIR tap counts are 8821 for 44100->16000, 61 for 48000->16000, 8821 for 22050->16000, 41 for 32000->16000, and 41 for 8000->16000. The bootstrap says the writer should hand-write FIR design, pre-padding, 1D constant-zero `upfirdn`, and output trimming in `rewrite-in-rust/bootstrap/asr_resample_poly_contract.md:61`, but the fixtures can be passed by an implementation that only matches very short boundary behavior.
- Required fix: Add at least one deterministic fixture with input length greater than the default FIR tap count for a common ASR ratio, preferably 44100->16000 or 22050->16000, and at least one moderate fixture over the 48000->16000 reduced 1/3 path. Keep full expected output or an agreed deterministic fixture projection that still catches interior phase/convolution errors.

- Severity: medium
- Location: rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py:36
- Issue: The checker bypasses SciPy validation when `target_rate == source_rate`, so equal invalid rates would be accepted by the harness even though SciPy rejects them.
- Evidence: The checker returns `x.copy()` before calling SciPy at `rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py:36`. SciPy validates integer and positive `up`/`down` before the identity return at `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:3993` and `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:3999`, with identity only after validation at `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:4010`. A direct check showed `resample_poly(float32[1], 0, 0)` and `resample_poly(float32[1], -1, -1)` both raise `ValueError: up and down must be >= 1`, while the current harness structure would copy for equal rates if such a fixture were added. This conflicts with the dependency record's invalid-rate contract at `rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml:12`.
- Required fix: Move positive integer validation before the identity shortcut, or call SciPy for all cases in the fixture verifier. Add at least one equal-invalid fixture such as `target_rate=0, source_rate=0` or both negative.

- Severity: medium
- Location: rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml:60
- Issue: Targeted source expansion misses the Kaiser window generation branch needed for exact SciPy default FIR parity.
- Evidence: The dependency proof names `_signaltools.py::resample_poly`, `_fir_filter_design.py::firwin`, `_upfirdn.py::upfirdn`, and `_upfirdn_apply.pyx` at `rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml:60`, and the reference source list stops at the same files at `rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml:82`. But `firwin` imports `get_window` from `.windows` at `third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py:12` and applies it at `third_party/sources/scipy-1.17.1/scipy/signal/_fir_filter_design.py:553`. The default `window=('kaiser', 5.0)` path dispatches through `third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py:2388` and `third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py:2546`, and `kaiser` uses `special.i0` at `third_party/sources/scipy-1.17.1/scipy/signal/windows/_windows.py:1319`. The current bootstrap tells the writer to build a Kaiser-windowed FIR at `rewrite-in-rust/bootstrap/asr_resample_poly_contract.md:65`, but does not record this source path or the acceptable replacement for `special.i0`.
- Required fix: Extend the dependency/bootstrap record to include `scipy/signal/windows/_windows.py::get_window` and `kaiser`, plus the `special.i0` decision. Either name the source-backed formula/approximation the Rust writer should use, or add fixtures tight enough to validate the chosen approximation.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py`: passed, `asr_resample_poly_contract fixtures ok: 12 cases`.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/romaji_asr/common.py`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed, `Source audit passed: 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.`
- `jq -r 'select(.expected.ok == true and .target_rate != .source_rate) | [.category, (.input|length), (.target_rate|tostring), (.source_rate|tostring)] | @tsv' fixtures/asr_resample_poly_contract.jsonl`: confirmed all successful non-identity resampling inputs are length <= 9.
- `jq ... | awk ...`: computed fixture reduced ratios and default SciPy tap counts; 44100->16000 and 22050->16000 require 8821 taps, 48000->16000 requires 61 taps, 32000->16000 and 8000->16000 require 41 taps.
- `uv run python - <<'PY' ... resample_poly(..., 0, 0) ... PY`: confirmed SciPy raises `ValueError: up and down must be >= 1` for equal invalid rates before the identity path.

## Boundary Decision

The manifest unit boundary is confirmed: `asr_resample_poly_contract` is the right independent unit for default 1D float32 SciPy `resample_poly` parity shared by Qwen WAV fallback and romaji audio loading. It should not be split, merged, deferred, or replaced.

The crate decision is directionally sound: rejecting `rubato` and `samplerate` for exact SciPy parity is justified because the public contract is SciPy's default `firwin + upfirdn` behavior, shape, dtype, trimming, and error text, not perceptual audio-quality resampling.

## Residual Risk

This review did not assess Rust implementation quality because no Rust implementation is in scope for this unit yet. It also did not review behavior parity, data/algorithm details, or runtime promotion. Those remain separate required roles in `manifest.yaml`.

## Promotion Note

This `dependency_bootstrap_reviewer` role blocks writer readiness. The unit boundary and crate rejection are acceptable, but the bootstrap evidence is not sufficient for a writer handoff until the steady-state fixtures, invalid equal-rate harness behavior, and Kaiser window source-expansion gap are fixed.
