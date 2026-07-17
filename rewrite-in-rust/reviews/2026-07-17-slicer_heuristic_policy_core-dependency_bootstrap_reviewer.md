# slicer_heuristic_policy_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass-with-followups

## Findings

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:117
- Issue: The unit's durable fixture evidence proves the dependency-injected policy function and pre-slicer parameter shape, but it does not directly invoke the public Rust `heuristic_slice` helper that composes the verified `Slicer` and `sliding_window_split` dependencies.
- Evidence: The bootstrap says the Rust surface includes both a fixture-provided policy function and an actual helper that composes verified dependencies (`rewrite-in-rust/bootstrap/slicer_heuristic_policy_core.md:63`). The composed helper exists at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:117` and wires `Slicer::new`, `Slicer::slice`, and `sliding_window_split` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:123` and `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:140`. The fixture test calls `apply_heuristic_policy` with injected split outputs at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:352`, then synthesizes the expected pre-slicer call from `pre_slicer_params` at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:365`. `rg -n "heuristic_slice\(" rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs` found only the public function definition, so the unit check compile-proves and code-inspects this wiring rather than fixture-proving it.
- Required fix: Before coordinator promotion, either add one small Rust test that exercises `heuristic_slice` through the real verified slicer/window dependencies on synthetic data, or narrow the bootstrap claim to say this unit proves only the injected policy layer while promotion will separately prove composed-helper wiring.

- Severity: low
- Location: rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl:2
- Issue: Exact `seg_dur == max_len_sec` retention is not fixture-proven.
- Evidence: Legacy Python only sends a segment to `_sliding_window_split` when `seg_dur > max_len_sec` (`inference/API/slicer_api.py:410`), and Rust mirrors the strict comparison at `rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs:171`. The fixture table covers a too-long split case at `rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl:2` and exact-min/short/tiny retention plus merge handoff at `rewrite-in-rust/fixtures/slicer_heuristic_policy_core.jsonl:3`, but no row pins the exact max boundary. This is a fixture coverage risk for the policy boundary named in `rewrite-in-rust/manifest.yaml:1072`.
- Required fix: Add a no-split fixture where `len(segment["waveform"]) / sr == max_len_sec`, or explicitly accept code-inspection coverage for that boundary until promotion.

## Boundary Decision

The manifest unit boundary is confirmed with followups. It should not be split, merged, deferred, or replaced for dependency/bootstrap reasons.

Record 0047 split the former wide heuristic/grid candidate into RMS/window, heuristic policy, and grid policy units, while keeping the seam as an independent `v2m-core` library with no PyO3, CLI/subprocess routing, HTTP, runtime router, ndarray, or broad audio crate (`rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:22`, `rewrite-in-rust/records/0047-split-slicer-heuristic-grid-boundary.md:31`). Record 0049 confirms this unit as policy orchestration only, covering stage ordering, fixed dependency parameters, long-segment split dispatch, offset adjustment, retention, sorting, and merge-helper handoff (`rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:21`), while excluding default `Slicer` internals, RMS/window internals, merge internals, grid search, pitch/RMVPE smart slicing, multiprocessing, audio IO, CLI, GUI, Web, and model execution (`rewrite-in-rust/records/0049-confirm-slicer-heuristic-policy-boundary.md:28`).

The dependency record matches that boundary: it keeps the seam as a legacy-owned library unit with no bridge dependencies (`rewrite-in-rust/dependencies/slicer_heuristic_policy_core.yaml:12`), confirms the inventory after the split (`rewrite-in-rust/dependencies/slicer_heuristic_policy_core.yaml:19`), justifies a hand-written local policy replacement (`rewrite-in-rust/dependencies/slicer_heuristic_policy_core.yaml:22`), and keeps default slicing, RMS/window splitting, merge internals, grid search, pitch/RMVPE, multiprocessing, audio IO, CLI, filesystem, model, GUI, and Web behavior outside this unit (`rewrite-in-rust/dependencies/slicer_heuristic_policy_core.yaml:27`). The manifest also keeps status `reimplemented`, `current_owner: legacy`, and rollback to Python `heuristic_slice` (`rewrite-in-rust/manifest.yaml:1066`, `rewrite-in-rust/manifest.yaml:1068`, `rewrite-in-rust/manifest.yaml:1086`).

The prerequisite slicer units are represented as verified manifest dependencies before this unit: `slicer_segment_merge_core` (`rewrite-in-rust/manifest.yaml:982`), `slicer_rms_and_default_core` (`rewrite-in-rust/manifest.yaml:1008`), and `slicer_rms_db_window_split_core` (`rewrite-in-rust/manifest.yaml:1036`). The Rust crate has no new slicer-specific external dependency; `v2m-core` currently depends on `encoding_rs`, `md-5`, and `serde_json` only (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:12`). No production Python bridge was introduced; the Rust module is exported only inside the independent crate (`rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:22`).

Writer/reviewer separation was preserved. This report covers exactly `dependency_bootstrap_reviewer` for `slicer_heuristic_policy_core`; I reviewed only and did not edit production code, tests, fixtures, bootstrap records, dependency records, manifest entries, or Rust modules.

## Checks

- `PYTHONDONTWRITEBYTECODE=1 uv run python rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py`: passed, exit 0 with no output.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slicer_heuristic`: passed; 2 `slicer_heuristic` tests passed, 83 unrelated `v2m-core` tests filtered, and 5 bridge tests filtered.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: passed; no ndarray, audio, PyO3, router, or slicer-specific crate was introduced.
- `rg -n "heuristic_slice\(" rewrite-in-rust/rust/crates/v2m-core/src/slicer_heuristic.rs rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: inspected; only the Rust helper definition was found in the Rust crate.
- `rg -n "mod slicer_heuristic|slicer_heuristic|heuristic_slice\(" rewrite-in-rust/rust inference application gui web_server.py web_task_manager.py scripts tests -S`: inspected; Rust references stay inside `v2m-core`, while production Python callers still use `inference/API/slicer_api.py::heuristic_slice`.
- `git diff --check`: passed, exit 0 with no output.

## Residual Risk

This dependency/bootstrap review does not replace the required behavior or data/algorithm review. It does not approve runtime promotion, external waveform payload validation, logging text parity, error mapping, numeric policy for promotion-time near-threshold behavior in dependencies, or any grid-search or pitch/RMVPE smart-slicing behavior.

The fixture harness intentionally uses fake `Slicer` and fake `_sliding_window_split` on the Python side (`rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:86`, `rewrite-in-rust/bootstrap/check_slicer_heuristic_policy_core.py:93`) so it can verify policy orchestration without re-testing accepted dependencies. That is the right seam for this unit, with the follow-up caveat that the public composed helper should receive at least one focused wiring proof before coordinator promotion.

## Promotion Note

This role does not block promotion if the two low-severity fixture/wiring followups are resolved or explicitly accepted by the coordinator. The unit still needs the remaining required review roles before any coordinator state update to `verified`, and the manifest should not be marked verified by this review alone.
