# slicer_heuristic_policy_core - behavior_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No behavior findings for this review.

## Scope Reviewed

- Unit and role reviewed: `slicer_heuristic_policy_core`, `behavior_reviewer`.
- Writer/reviewer separation: confirmed. This review inspected the target artifacts and wrote only this report; it did not edit production code, fixtures, bootstrap records, dependency records, control-plane records, or the manifest.
- Manifest state: `rewrite-in-rust/manifest.yaml:1064` names the unit, `rewrite-in-rust/manifest.yaml:1066` marks it `reimplemented`, `rewrite-in-rust/manifest.yaml:1072` defines the public behavior policy, `rewrite-in-rust/manifest.yaml:1078` through `rewrite-in-rust/manifest.yaml:1084` list the fixture/checker/Rust verification evidence, and `rewrite-in-rust/manifest.yaml:1086` keeps rollback on legacy `heuristic_slice`.
- Boundary: record 0049 confirms this as a policy orchestration unit covering stage ordering, fixed dependency parameters, long-segment split dispatch, parent-offset adjustment, exact-min/short/ultra-short retention, sort-before-merge, and merge-helper handoff (`rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:21` through `rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:26`). It excludes default `Slicer` internals, RMS/window splitting, merge internals, grid search, pitch/RMVPE smart slicing, process pools, IO, CLI, GUI, Web, and model execution (`rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:28` through `rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:30`).

## Parity Notes

- The legacy function constructs `Slicer` with caller threshold/min-silence values and fixed `min_interval=200` and `max_sil_kept=100` at `inference/API/slicer_api.py:395` through `inference/API/slicer_api.py:402`. Rust preserves those pre-slicer parameters in `HeuristicConfig::pre_slicer_params` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:36` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:46`.
- Legacy long-segment handling uses `len(segment["waveform"]) / sr`, calls `_sliding_window_split` with `frame_length=2048` and `hop_length=512`, then adds the parent offset to each subchunk at `inference/API/slicer_api.py:408` through `inference/API/slicer_api.py:422`. Rust mirrors that policy with `legacy_len_duration_sec`, `split_request`, and parent-offset adjustment at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:48` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:57` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:169` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:198`.
- Legacy retains exact-min, short, and ultra-short segments before sorting and merge at `inference/API/slicer_api.py:423` through `inference/API/slicer_api.py:433`, then runs the short-segment merge only when chunks remain at `inference/API/slicer_api.py:436` through `inference/API/slicer_api.py:447`. Rust preserves the same observable output policy by passing every non-long segment through, sorting by offset, then calling `merge_tiny_chunks` and conditionally `merge_short_segments` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:176` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:193`.
- The Python checker replaces `Slicer` and `_sliding_window_split` with fakes at `rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:83` through `rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:128`, then compares chunks, slicer calls, and split calls against the fixture table at `rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:140` through `rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:150`.
- The shared JSONL fixtures cover empty pre-slicer parameter capture, long split args and offset adjustment, sort-before-merge with leading tiny merge, short merge handoff, and the stereo `len(waveform)` policy duration case at `rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl:1` through `rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl:5`.
- The Rust test includes the same fixture table and replays it through `apply_heuristic_policy`, including split-call capture and expected pre-slicer params, at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:205` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:331` through `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:387`.
- Runtime rollback remains intact. The rewrite crate exposes `slicer_heuristic` only inside `v2m-core` at `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:22`, while production `slice_audio` still dispatches heuristic slicing to Python `heuristic_slice` at `inference/API/slicer_api.py:680` through `inference/API/slicer_api.py:681`, and the custom-bounds wrapper temporarily replaces then restores the Python global at `inference/API/slicer_api.py:756` through `inference/API/slicer_api.py:788`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py`: pass, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_heuristic`: pass; 2 `slicer_heuristic` tests passed, 83 `v2m_core` tests and 5 `v2m_quant_bridge` tests were filtered out.
- `git diff --check`: pass, exit 0 with no output.
- `rg -n "slicer_heuristic|heuristic_slice" --glob '!rewrite-in-rust/rust/target/**'`: inspected; Rust references are confined to rewrite artifacts and `v2m-core`, while production runtime references remain in legacy Python.
- `rg -n "<<<<<<<|=======|>>>>>>>" rewrite-in-rust/manifest.yaml rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md rewrite-in-rust/dependencies/slicer_heuristic_policy_core.yaml rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl`: pass, exit 1 because no conflict-marker matches were found.

## Residual Risk

This behavior review proves parity only for the confirmed synthetic dependency-injected policy boundary. It does not re-prove default `Slicer` internals, RMS-dB/window splitting, merge-helper internals, grid-search scoring, pitch/RMVPE smart slicing, `librosa.pyin`, model execution, audio decoding, filesystem writes, CLI parsing, GUI, Web, multiprocessing, or a Python/Rust bridge (`rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:23` through `rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:27`).

The Rust module has a composed `heuristic_slice` helper, but promotion still needs an explicit bridge/adapter decision for waveform payload validation, logging text, and error mapping before production callers import it (`rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:63` through `rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:66` and `rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:108` through `rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:110`).

## Promotion Note

This behavior role does not block promotion evidence for `slicer_heuristic_policy_core`. The coordinator should not mark the unit `verified` from this report alone; the manifest still lists separate dependency/bootstrap and data/algorithm review gates for this unit.
