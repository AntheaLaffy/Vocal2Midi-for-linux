# slicer_pitch_override_core - behavior_reviewer rerun

Date: 2026-07-17
Role: behavior_reviewer
Unit: slicer_pitch_override_core

## Findings

- Severity: medium
- Location: `inference/API/slicer_api.py:478`, `inference/API/slicer_api.py:481`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:177`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:189`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:191`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:212`
- Issue: The new internal already-computed-voiced-flags seam changed `pitch_based_split_with_override` ordering for short inputs. Legacy Python checks `total_sec <= max_len_sec` and returns the segment before calling `get_pitch_curve`; Rust now resolves the supplied-mask/pyin branch before reaching the short early return inside `pitch_based_split_with_voiced_flags`, so a short direct split with no override returns `Err(PyinUnsupported)` instead of the original chunk.
- Evidence: Python early-returns at `inference/API/slicer_api.py:478` before `get_pitch_curve` at `inference/API/slicer_api.py:481`. Rust chooses the override branch at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:177` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:190`, then calls the helper at line 191 where the early return now lives at line 212. Probe command: `uv run python - <<'PY' ... _pitch_based_split(... voiced_flag_override=None ...)` returned `[(0.0, [0.0, 1.0, 2.0])]` without calling `get_pitch_curve`; compiled Rust probe against `pitch_based_split_with_override` with the same short waveform and no override printed `Err(PyinUnsupported)`.
- Required fix: Restore the Python order by checking the short-segment early return before requiring/resampling a supplied override, or add an equivalent public wrapper path. Add a fixture for short direct split with `voiced_flag_override=None` or non-positive step to prevent this regression.

- Severity: low
- Location: `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:7`, `rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:104`, `inference/API/slicer_api.py:506`, `inference/API/slicer_api.py:507`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:233`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:237`
- Issue: The prior low fixture coverage follow-up is not fully closed for the voiced-frame clamp path. The new `pitch_split_direct_voiced_frame_clamp` row injects `direct_voiced_flag` length `4`, while the first split window is frames `2..4`, so `end_frame` equals `len(voiced_flag)` and does not actually clip past the vector length. It proves the direct voiced seam and boundary indexing, but not out-of-range voiced frame clamping.
- Evidence: The row at `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:7` uses `sr=1`, `hop_length=1`, `min_len_sec=2.0`, and `max_len_sec=4.0`, so Python computes `start_frame=2` and `end_frame=4` before clamp. The injected vector is `direct_voiced_flag:[true,true,false,false]`, length `4`; Python's `min(end_frame, len(voiced_flag))` at `inference/API/slicer_api.py:507` and Rust's `end_frame.min(voiced_len)` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:237` both leave `end_frame` at `4`. The checker does monkeypatch `get_pitch_curve` for direct voiced rows at `rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py:104`, but this specific row does not force clipping.
- Required fix: Change or add a voiced-clamp row where the split window exceeds the already-computed voiced vector, for example by shortening `direct_voiced_flag` or increasing `max_len_sec`/window end, and keep the expected chunks tied to Python output.

- Severity: none
- Location: `rewrite-in-rust/fixtures/slicer_pitch_override_core.jsonl:8`, `inference/API/slicer_api.py:531`, `inference/API/slicer_api.py:533`, `inference/API/slicer_api.py:534`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:248`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:252`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:253`
- Issue: The RMS fallback clamp half of the prior low follow-up is closed.
- Evidence: The `pitch_split_rms_fallback_frame_clamp` row supplies `rms_db` length `3` while the first split window reaches frame `4`; Python clamps `end_frame_rms` at `inference/API/slicer_api.py:534`, and Rust mirrors the same clamp at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_pitch.rs:253`. The shared Python fixture checker and Rust fixture test both pass with this row.
- Required fix: none for RMS fallback clamp coverage.

- Severity: none
- Location: `rewrite-in-rust/manifest.yaml:1120`, `rewrite-in-rust/manifest.yaml:1122`, `rewrite-in-rust/manifest.yaml:1124`, `rewrite-in-rust/manifest.yaml:1142`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:61`, `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:110`, `rewrite-in-rust/records/0053-confirm-slicer-pitch-override-boundary.md:49`
- Issue: Rollback and runtime ownership remain preserved in the follow-up.
- Evidence: The manifest keeps `slicer_pitch_override_core` as `status: reimplemented` with `current_owner: legacy` and rollback to Python pitch slicing. The bootstrap keeps the seam as an independent Rust library with no bridge and names the Python rollback imports at `rewrite-in-rust/bootstrap/slicer_pitch_override_core.md:110`. Record 0053 still says no production bridge is introduced.
- Required fix: none.

## Decision

fail

Prior low follow-up status: not closed. RMS fallback frame-clamp coverage is closed, but voiced-frame clamp coverage is still not proven, and the follow-up introduced a short-input public behavior mismatch.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_pitch_override_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_pitch`: pass; 3 focused `slicer_pitch` tests passed, 89 filtered in `v2m_core`, and 0 matching bridge tests ran.
- `git diff --check`: pass.
- Python short-input probe for `_pitch_based_split` with no override: pass for legacy behavior; returned one original chunk without calling `get_pitch_curve`.
- Rust short-input probe for `pitch_based_split_with_override` with no override: mismatch reproduced; printed `Err(PyinUnsupported)`.

## Residual Risk

This rerun only reviewed the follow-up changes. The broader unit remains fixture-bound and still intentionally excludes `librosa.pyin`, RMVPE/model execution, real process scheduling, GUI/Web/CLI routing, and production bridge error mapping.

## Promotion Note

This rerun blocks closing the prior behavior follow-up. Do not use this rerun as coordinator evidence for verified behavior until the short-input ordering mismatch is fixed and the voiced-frame clamp fixture actually forces clipping.
