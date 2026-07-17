# slicer_rms_db_window_split_core - data_algorithm_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: `rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:2`
- Issue: Stereo RMS-dB averaging is implemented, but the durable fixture set does not prove a non-identical, non-canceling stereo average. The RMS stereo fixture cancels channels to zero and primarily proves clipping to the `-200 dB` floor; the stereo split fixture uses identical channels and proves channel slicing rather than averaging.
- Evidence: Legacy Python averages channels before RMS at `inference/API/slicer_api.py:234`, while Rust averages `Waveform::Stereo` samples at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:145`. Current coverage is the cancel-to-floor RMS case at `rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:2` and identical-channel stereo slicing at `rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:6`. A supplemental legacy probe with `[[1,0,1,0],[0,1,0,1]]` produced non-floor RMS-dB values `[-9.030900001525879, -6.020599842071533, -6.020599842071533, -6.020599842071533, -9.030900001525879]`.
- Required fix: Before runtime promotion, add one `rms_db` fixture with non-identical, non-canceling stereo channels, or explicitly record that this exact averaging proof is delegated to accepted `slicer_rms_and_default_core` evidence.

- Severity: low
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:66`
- Issue: Threshold branch selection is strict and can be sensitive to numeric dtype at the cut boundary. Legacy `librosa.feature.rms` returns `float32` by default for the `y=` path, while Rust computes RMS and dB values as `f64`; current fixtures do not include a near-threshold case that would expose a branch flip from sub-micro-dB rounding differences.
- Evidence: Legacy `get_rms_db` calls `librosa.feature.rms(...)[0]` and then applies `20 * np.log10(...)` at `inference/API/slicer_api.py:236`; librosa's RMS default dtype is `np.float32` at `third_party/sources/librosa-0.11.0/librosa/feature/spectral.py:801`, `third_party/sources/librosa-0.11.0/librosa/feature/spectral.py:809`, and `third_party/sources/librosa-0.11.0/librosa/feature/spectral.py:885`. Rust maps f64 RMS values directly to dB at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:66`. The split decision uses strict `<` in Python at `inference/API/slicer_api.py:284` and in Rust at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:169`.
- Required fix: Before runtime promotion, either add a near-threshold fixture that fixes the accepted rounding policy for strict threshold cuts, or intentionally cast/round the Rust RMS-dB values to the legacy float32 behavior before threshold comparison.

## Scope Confirmation

- Unit and role reviewed: `slicer_rms_db_window_split_core`, `data_algorithm_reviewer`.
- Manifest state: the unit is `reimplemented`, confirmed, and requires dependency, behavior, and data/algorithm reviews at `rewrite-in-rust/manifest.yaml:1036`, `rewrite-in-rust/manifest.yaml:1038`, and `rewrite-in-rust/manifest.yaml:1045`.
- Confirmed boundary: record 0047 split the former wide slicer heuristic/grid target into RMS/window, heuristic policy, and grid policy units at `rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:20`. This unit covers only `get_rms_db` and `_sliding_window_split`; heuristic orchestration, grid scoring, default `Slicer` internals, merge helpers, pitch/RMVPE, audio IO, multiprocessing, CLI, GUI, Web, model execution, and production routing remain out of scope at `rewrite-in-rust/bootstrap/slicer_rms_db_window_split_core.md:5` and `rewrite-in-rust/bootstrap/slicer_rms_db_window_split_core.md:22`.
- Writer/reviewer separation: confirmed for this review. I inspected sources, fixtures, and checks, and wrote only this report. I did not edit production code, manifests, fixtures, bootstrap records, dependency records, or Rust modules.

## Data And Algorithm Assessment

- RMS-dB framing matches the intended legacy subset for normal mono/channel-major stereo inputs. Python averages stereo with `np.mean(..., axis=0)`, uses center-padded librosa RMS, clips at `1e-10`, and converts with `20 * log10` at `inference/API/slicer_api.py:228`. Rust averages channel-major stereo, calls the shared constant-padded RMS helper, and applies the same clip/log transform at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:58`; the shared RMS helper pads and steps frames directly at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:99`.
- Frame/time conversion follows the scalar librosa path used by the splitter. Python calls `librosa.time_to_frames`, clamps to `[0, len(rms_db)]`, and converts back with `librosa.frames_to_time` at `inference/API/slicer_api.py:274` and `inference/API/slicer_api.py:295`; upstream librosa defines these as time-to-samples, floor sample-to-frame, frame-to-samples, and samples-to-time at `third_party/sources/librosa-0.11.0/librosa/core/convert.py:318` and `third_party/sources/librosa-0.11.0/librosa/core/convert.py:240`. Rust uses truncating nonnegative sample conversion, integer frame division, and `frame * hop / sr` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:186` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:198`.
- Threshold and local-min tie behavior is correctly mirrored for the fixture boundary. Python selects the latest safe frame with `safe_cut_indices[-1]` or falls back to `np.argmin`, which returns the first minimum, at `inference/API/slicer_api.py:283`. Rust mirrors this with `next_back()` over strict-threshold matches and a first-min `min_by` fallback at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:169` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:177`.
- Stereo slicing uses the same last-axis sample semantics for valid channel-major stereo. Python slices `waveform[:, start_sample:end_sample]` at `inference/API/slicer_api.py:270` and `inference/API/slicer_api.py:300`; Rust slices every stereo channel over the same sample interval at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:218`.
- Complexity is acceptable for this compatibility unit. RMS is direct O(number of frames * frame_length), matching the simple librosa time-domain RMS path; sliding-window splitting recomputes RMS once and then scans only each selected search window. No benchmark is required before this fixture-bound state update.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py`: pass, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_window`: pass; 2 `slicer_window` tests passed, 81 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- Supplemental legacy probe for scalar frame/time conversion, strict threshold latest-safe selection, first `np.argmin` tie behavior, non-canceling stereo RMS-dB shape, and local-min fixture offsets: pass after rerun outside the sandbox because `uv` needed to create a cache lock under `~/.cache/uv`.
- Supplemental legacy dtype probe: pass after rerun outside the sandbox; confirmed librosa RMS and dB outputs are `float32` for the checked `y=` path.

## Residual Risk

- The Rust `Waveform::Stereo(Vec<Vec<f64>>)` representation can express malformed shapes that normal NumPy arrays would not represent. `rms_db` rejects empty or ragged stereo after it reaches averaging, but the early no-split path returns the waveform as-is. This is acceptable for the current synthetic fixture seam; a promotion bridge should define rectangular stereo validation and error mapping.
- Inputs with non-finite timing values, non-positive sample rates, zero frame/hop sizes, or tiny `min_len_sec` values that do not advance beyond the current frame are not exhaustively matched to NumPy/librosa behavior. The current unit assumes normal finite caller policy parameters.
- The implementation reuses `slicer_default::get_rms`; dependency review has already recorded the need to document that internal Rust prerequisite before promotion.

## Promotion Note

This data/algorithm role does not block coordinator state update if the low-severity follow-ups are tracked or explicitly accepted. Behavior review already passed, and dependency/bootstrap review passed with follow-ups; the coordinator should not treat this report as permission to introduce a runtime bridge without resolving or accepting the promotion-time validation and numeric-policy risks above.
