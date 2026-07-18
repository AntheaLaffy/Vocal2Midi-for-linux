# hfa_metrics_core - behavior_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:588
- Issue: `python_round_6` does not preserve Python `round(value, 6)` tie behavior. Rust `f64::round()` rounds half values away from zero after scaling, while Python's public metric outputs use `round(..., 6)` and can round half-even. This affects observable outputs from `VlabelerEditRatio.compute`, `IntersectionOverUnion.compute_*`, `BoundaryEditDistance.compute`, `BoundaryEditRatio.compute`, and `BoundaryEditRatioWeighted.compute`.
- Evidence: Python source uses `round(..., 6)` at `/home/fuurin/code/Vocal2Midi-for-linux/inference/HubertFA/tools/metrics.py:102`, `:156`, `:162`, `:172`, `:255`, `:278`, and `:305`. A public BoundaryEditDistance case with one matching phoneme at `0.0` vs `0.0000005` returns `0.0` in Python: `uv run python -c 'import textgrid as tg; from inference.HubertFA.tools.metrics import CustomPointTier, BoundaryEditDistance; p=CustomPointTier(name="p", minTime=0, maxTime=1); t=CustomPointTier(name="t", minTime=0, maxTime=1); p.addPoint(tg.Point(0.0,"a")); t.addPoint(tg.Point(0.0000005,"a")); m=BoundaryEditDistance(); print(m.update(p,t), m.distance, m.compute())'` printed `True 5e-07 0.0`. A Rust probe of the current helper expression printed `0.000000500000 -> 0.000001000000`.
- Required fix: Replace the rounding helper with Python-compatible six-decimal rounding for f64 values used by these public compute methods, and add fixtures that hit half-even ties such as `BoundaryEditDistance` distance `0.0000005`.

- Severity: medium
- Location: rust/crates/v2m-core/src/hfa_metrics.rs:352
- Issue: `IntersectionOverUnion::compute_list` documents caller-provided ordering but returns a `BTreeMap`, which sorts keys and cannot preserve Python list-mode dict insertion order. The Python implementation builds the dict by iterating the caller's `phonemes` list in order at `/home/fuurin/code/Vocal2Midi-for-linux/inference/HubertFA/tools/metrics.py:170`.
- Evidence: The fixture contract requires list phoneme results to preserve requested order, but the Rust fixture runner bypasses `compute_list` and manually iterates the request list at `rust/crates/v2m-core/src/hfa_metrics.rs:788`, so the public Rust helper's ordering is not tested.
- Required fix: Change list-mode output to an order-preserving projection, or explicitly narrow the Rust public API and fixture tests so no caller can observe sorted-key behavior where Python would expose insertion order.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_metrics_core.py`: passed, validated 11 `hfa_metrics_core` fixtures.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_metrics_core`: passed, `hfa_metrics::tests::hfa_metrics_core_fixture_parity` ok.
- `uv run python -c 'import textgrid as tg; from inference.HubertFA.tools.metrics import CustomPointTier, BoundaryEditDistance; p=CustomPointTier(name="p", minTime=0, maxTime=1); t=CustomPointTier(name="t", minTime=0, maxTime=1); p.addPoint(tg.Point(0.0,"a")); t.addPoint(tg.Point(0.0000005,"a")); m=BoundaryEditDistance(); print(m.update(p,t), m.distance, m.compute())'`: passed, demonstrated Python output `True 5e-07 0.0`.
- `/tmp/hfa_round_probe`: passed, demonstrated the Rust helper expression rounds `0.0000005` to `0.000001`.

## Residual Risk

The existing JSONL fixtures cover the named metric classes and state-reset quirks, but they do not cover Python half-even rounding ties or direct use of `IntersectionOverUnion::compute_list`. I did not review dependency strategy, algorithm complexity, Rust style, or error tracing beyond behavior-observable parity.

## Promotion Note

This role blocks promotion. The unit should not be marked verified until the rounding parity issue is fixed and the IoU list-order behavior is either corrected or made non-observable by contract.
