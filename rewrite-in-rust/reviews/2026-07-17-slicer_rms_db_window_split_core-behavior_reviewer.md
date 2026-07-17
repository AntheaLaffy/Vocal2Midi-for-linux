# slicer_rms_db_window_split_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No behavior findings for this review.

## Scope Reviewed

- Unit and role reviewed: `slicer_rms_db_window_split_core`, `behavior_reviewer`.
- Writer/reviewer separation: confirmed. This review inspected the implementation and wrote only this report; it did not edit production code, fixtures, bootstrap files, records, or the manifest.
- Manifest state: `rewrite-in-rust/manifest.yaml:1036` names the unit, `rewrite-in-rust/manifest.yaml:1038` marks it `reimplemented`, `rewrite-in-rust/manifest.yaml:1044` defines the public behavior policy, `rewrite-in-rust/manifest.yaml:1050` through `rewrite-in-rust/manifest.yaml:1056` list the fixture/checker/Rust verification evidence, and `rewrite-in-rust/manifest.yaml:1057` keeps rollback on legacy Python.
- Boundary: record 0047 split the former wide slicer heuristic/grid target into `slicer_rms_db_window_split_core`, `slicer_heuristic_policy_core`, and `slicer_grid_search_policy_core` (`rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:20`). It also keeps this unit as an independent `v2m-core` library with no PyO3, subprocess, HTTP, runtime router, ndarray, or broad audio crate (`rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:31`), with rollback by retaining `inference/API/slicer_api.py` as runtime owner (`rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:46`).

## Parity Notes

- Python `get_rms_db` averages multidimensional waveforms over axis 0, computes center-padded librosa RMS, clips at `1e-10`, and converts to `20 * log10` at `inference/API/slicer_api.py:228`. Rust mirrors this through `waveform_mean_samples`, `get_rms(..., PadMode::Constant)`, and the same dB conversion at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:58`, backed by the shared RMS implementation at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_default.rs:99`.
- Python `_sliding_window_split` returns the whole waveform when `total_sec <= max_len_sec`, otherwise converts the search window to frames, clamps the frame range, chooses the latest below-threshold frame, falls back to the first local minimum, slices mono/stereo samples, appends a tail chunk, and advances by cut time at `inference/API/slicer_api.py:253`. Rust follows the same control flow at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:78`.
- Python's current empty-window path assigns `cut_frame` but raises `UnboundLocalError` when formatting the unset `cut_type` at `inference/API/slicer_api.py:280` and `inference/API/slicer_api.py:303`. Rust returns `SlicerWindowError::MissingCutType` with the Python-compatible type and message at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:23`, `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:35`, and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:123`.
- The JSONL fixture table covers RMS-dB center padding, stereo averaging and floor clipping, no-split return, latest threshold-safe selection, first local-min fallback, stereo slicing, and the empty-window error path (`rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:1` through `rewrite-in-rust/fixtures/slicer_rms_db_window_split_core.jsonl:7`).
- The Python checker replays the same fixture table against legacy `get_rms_db` and `_sliding_window_split` at `rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py:95`, `rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py:103`, and `rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py:105`. The Rust test includes the same fixture file and replays it at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:241` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_window.rs:341`.
- Runtime rollback remains intact. The Rust module is exposed only from the rewrite crate at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:23`; production heuristic/smart callers still use the Python helpers at `inference/API/slicer_api.py:412` and `inference/API/slicer_api.py:530`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_rms_db_window_split_core.py`: pass, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_window`: pass; 2 `slicer_window` tests passed, 81 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `git diff --check`: pass, exit 0 with no output.
- `rg -n "slicer_window|sliding_window_split|rms_db\\(" --glob '!rewrite-in-rust/rust/target/**'`: inspected; `slicer_window` hits are confined to rewrite artifacts and `v2m-core`, while production runtime references remain in legacy Python.

## Residual Risk

This behavior review proves parity only for the confirmed synthetic mono and channel-major stereo fixture boundary. It does not prove heuristic policy orchestration, grid-search scoring, default silence `Slicer` internals, segment merge internals, pitch/RMVPE smart slicing, `librosa.pyin`, model execution, audio decoding, filesystem writes, CLI parsing, GUI, Web, multiprocessing, or a Python/Rust bridge.

Unsupported boundary inputs such as ragged stereo payloads, empty stereo payloads, non-finite timing values, invalid sample rates, and invalid frame or hop sizes are handled as Rust boundary errors rather than exhaustively matched to NumPy/librosa exception shapes. Before runtime promotion, the bridge or adapter should define validation and error mapping for those cases.

## Promotion Note

This behavior role does not block promotion evidence for `slicer_rms_db_window_split_core`. The coordinator should not mark the unit `verified` from this report alone; the manifest still lists separate dependency/bootstrap and data/algorithm review gates for this unit.
