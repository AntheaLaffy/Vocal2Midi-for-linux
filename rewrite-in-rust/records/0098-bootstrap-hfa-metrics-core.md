# 0098 - Bootstrap HFA Metrics Core

Date: 2026-07-18

## Decision

Confirm `hfa_metrics_core` as a single planned writer unit.

The unit covers only deterministic metric behavior in
`inference/HubertFA/tools/metrics.py`: synthetic point-tier ordering,
vlabeler edit count/ratio, IoU accumulation, LCS matching, boundary edit
distance, boundary edit ratio, weighted boundary ratio, and the legacy reset
quirks. Python remains the runtime owner. No bridge, Python caller route,
TextGrid IO port, required NumPy-array crate dependency, or Rust production code is introduced
by this bootstrap.

## Dependency Evidence

- `pyproject.toml`, `requirements.txt`, and `uv.lock` declare
  `numpy<2.0.0` / `numpy==1.26.4` and `textgrid` / `textgrid==1.6.1`.
- `third_party/sources/manifest.json` indexes `numpy` at
  `third_party/sources/numpy-1.26.4` and `textgrid` at
  `third_party/sources/textgrid-1.6.1`.
- `third_party/sources/MISSING_SOURCES.md` does not list `numpy` or
  `textgrid`; both have first-layer source records.
- `third_party/source_audit.json` reports no audit errors and no compiled
  binary artifacts under `third_party/`.
- `third_party/native_sources/manifest.json` maps NumPy's OpenBLAS native
  dependency, but no public seam path reaches BLAS or native ABI behavior.

`textgrid.PointTier` source was inspected because `CustomPointTier` subclasses
it. The only required inherited behavior is `.points`, `__len__`,
`__getitem__`, slicing through Python list semantics, and point comparison by
time. The custom `addPoint` override bypasses upstream bounds and duplicate
checks, so writer fixtures can model the effective point/tier behavior directly
without porting the full TextGrid package.

`ndarray` is now considered acceptable for future AI/model-inference and
numeric migration units. For `hfa_metrics_core`, it is deferred rather than
required: the metric seam only needs one-dimensional f64 absolute differences
and sums, so requiring ndarray here would not materially improve this unit.
The writer may introduce it only as an intentional shared numeric-array helper
with fixture evidence, not as a broad NumPy compatibility layer.

## Fixture Lessons

Discovery probes against Python 3.12/installed dependencies found fixture-worthy
quirks:

- Duplicate-time `CustomPointTier.addPoint` inserts the new equal-time point
  before the existing point.
- Vlabeler edits truncate unequal pred/target lengths before DP but the ratio
  denominator still uses the original target length.
- `IntersectionOverUnion.compute()` dict mode returns `0.0` when
  `sum == intersection`, while string mode can raise `ZeroDivisionError` when
  both are zero.
- LCS backtracking chooses the target-side decrement on equal DP scores, which
  affects repeated-label match pairs.
- `BoundaryEditDistance.update()` can return `False` on equal-length label
  mismatch without mutating distance/phoneme counters.
- `BoundaryEditDistance.reset()` clears `distance` and `phonemes` but leaves
  `error_phonemes` unchanged.
- `BoundaryEditRatio` and `BoundaryEditRatioWeighted` do not implement
  `reset()`, so calling reset raises `NotImplementedError`.
- Empty target tiers can raise `IndexError` in ratio update after the nested
  distance metric reports success.

## Reversal

Rollback remains keeping `inference.HubertFA.tools.metrics` as runtime owner.
If implementation discovery later finds the unit too broad, split after
preserving the shared point-tier and LCS fixture contract rather than changing
Python callers.
