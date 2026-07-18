# asr_resample_poly_contract - dependency_bootstrap_reviewer

PASS

Date: 2026-07-18
Decision: pass

## Findings

No blocking findings.

The previous blockers are fixed:

- Severity: none
- Location: rewrite-in-rust/fixtures/asr_resample_poly_contract.jsonl:10
- Issue: The fixture set now includes projected long steady-state cases that exercise the SciPy default polyphase FIR/upfirdn path beyond short boundary behavior.
- Evidence: `fixtures/asr_resample_poly_contract.jsonl:10` adds 44100 -> 16000 with 9000 input samples and 3266 output samples. For the reduced 160/441 ratio, SciPy's default filter has 8821 taps, so this case reaches interior behavior beyond the prior short-only fixtures. `fixtures/asr_resample_poly_contract.jsonl:11` adds a moderate 48000 -> 16000 reduced 1/3 case with 256 input samples, 86 output samples, and a 61-tap default filter. `fixtures/asr_resample_poly_contract.jsonl:12` and `fixtures/asr_resample_poly_contract.jsonl:13` add 22050 -> 16000 and 8000 -> 16000 projected cases. Each long case asserts shape, head/mid/tail selected values, `finite_sum`, and `finite_abs_sum`.
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py:52
- Issue: The checker now calls SciPy for every fixture case and preserves invalid equal-rate validation.
- Evidence: `result_for()` always calls `resample_poly(x, case["target_rate"], case["source_rate"])` before building either success or error projections at `bootstrap/check_asr_resample_poly_contract.py:52`. The fixture file includes equal invalid-rate cases at `fixtures/asr_resample_poly_contract.jsonl:17` and `fixtures/asr_resample_poly_contract.jsonl:18`. A monkeypatch probe counted 18 SciPy calls for 18 fixture cases, including `(16000, 16000)`, `(0, 0)`, and `(-16000, -16000)`.
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml:58
- Issue: The dependency/bootstrap records now document the default Kaiser window source expansion needed by a writer.
- Evidence: `dependencies/asr_resample_poly_contract.yaml:58` records targeted deep expansion through `_signaltools.py::resample_poly`, `_fir_filter_design.py::firwin`, `_upfirdn.py::upfirdn`, `_upfirdn_apply.pyx`, and the `firwin -> get_window -> kaiser -> scipy.special.i0` default-window path. `bootstrap/asr_resample_poly_contract.md:68` gives the writer steps for FIR construction, beta 5.0 Kaiser weights, pre-padding, constant-zero upfirdn, trimming, and f32 output. SciPy source confirms validation before identity at `third_party/sources/scipy-1.17.1/scipy/signal/_signaltools.py:3993`, default FIR construction at `_signaltools.py:4024`, pre-padding/trimming at `_signaltools.py:4036`, upfirdn keep slicing at `_signaltools.py:4072`, `firwin` applying `get_window` at `_fir_filter_design.py:553`, and Kaiser using `special.i0` at `windows/_windows.py:1319`.
- Required fix: none.

## Boundary Decision

The manifest unit boundary is confirmed. `asr_resample_poly_contract` should stay an independent narrow unit for default 1D float32 SciPy `resample_poly` parity shared by Qwen WAV fallback and romaji audio loading. It should not be split, merged, deferred, or replaced.

The crate decision remains acceptable. Rejecting generic audio resampler crates for this unit is justified because the claimed compatibility surface is SciPy's default FIR design, Kaiser weights, zero-padding, upfirdn output length, trimming, dtype projection, and error text rather than perceptual audio quality.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py`: passed, `asr_resample_poly_contract fixtures ok: 18 cases`.
- `uv run python - <<'PY' ... yaml.safe_load(...) ... PY`: passed for `rewrite-in-rust/manifest.yaml` and `rewrite-in-rust/dependencies/asr_resample_poly_contract.yaml`.
- `uv run python - <<'PY' ... fixture inspection ... PY`: confirmed 18 fixture cases, 4 `long_steady_*` cases, selected head/mid/tail values and finite sum projections in each long case, and 2 equal invalid-rate cases.
- `uv run python - <<'PY' ... monkeypatch checker resample_poly ... PY`: passed; counted 18 SciPy calls for 18 fixture cases, including identity and equal-invalid rates.
- `uv run python -m py_compile inference/qwen3asr_dml/utils.py inference/romaji_asr/common.py`: passed.
- `uv run python scripts/audit_vendored_sources.py`: passed, `Source audit passed: 135 Python packages, 41 native-extension packages, 269 foreign runtime native binaries, 0 third_party binary artifacts.`

## Residual Risk

This review did not assess a Rust implementation because none is in scope yet. Behavior parity, data/algorithm precision, and runtime promotion remain separate manifest-required review gates after writer work.

The long fixtures use selected values and aggregate finite sums rather than full-output arrays. That is acceptable for dependency/bootstrap writer readiness, but later behavior/data-algorithm reviews should ensure the Rust implementation is not merely fitting the projected indices.

## Promotion Note

This `dependency_bootstrap_reviewer` rerun does not block writer readiness. The previous blockers are fixed, and the unit is ready for coordinator state update for this review role only.
