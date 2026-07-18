# hfa_metrics_core Bootstrap

## Boundary

Cover the pure metric helpers in `inference/HubertFA/tools/metrics.py`:
`CustomPointTier`, `VlabelerEditsCount`, `VlabelerEditRatio`,
`IntersectionOverUnion`, `compute_lcs_matches`, `get_matched_pairs`,
`BoundaryEditDistance`, `BoundaryEditRatio`, and `BoundaryEditRatioWeighted`.

Keep this as an independent Rust library seam with synthetic point-tier
fixtures. Do not add PyO3, a subprocess bridge, a Python router, TextGrid file
IO, model execution, filesystem effects, or production caller routing.

## Dependency Decision

`textgrid==1.6.1` is locked in `uv.lock` and indexed in
`third_party/sources/manifest.json` at
`third_party/sources/textgrid-1.6.1`. The first-layer source was inspected for
`Point`, `PointTier.__len__`, `PointTier.__getitem__`, and
`PointTier.addPoint`. That is enough evidence to avoid deeper TextGrid work:
`metrics.py` only needs point `time`/`mark`, `.points`, length, indexing,
slicing, and `CustomPointTier.addPoint` sorted insertion. The custom override
bypasses upstream duplicate-time and bounds checks, so fixtures should model
the effective behavior directly rather than porting the full package.

`numpy==1.26.4` is locked and indexed at
`third_party/sources/numpy-1.26.4`. The public-seam NumPy use is limited to
`np.array([times])`, vector subtraction, `np.abs`, `np.sum`, scalar
accumulation, and `round(..., 6)` in the boundary metrics.

`ndarray` is acceptable as a Rust dependency for later model-adjacent and
numeric units, and it may be introduced by the metrics writer only if that
writer batch deliberately establishes a small shared numeric-array helper with
fixture evidence. It is not required for `hfa_metrics_core` parity: plain
`Vec<f64>` arithmetic is enough for the current one-dimensional absolute
difference and sum behavior. Either way, preserve observable return values and
errors in fixtures, and do not widen this unit into general NumPy ndarray
compatibility.

## Fixture Contract

Create Python golden fixtures before implementing Rust. Required coverage:

- `CustomPointTier` point insertion order, including duplicate-time
  `bisect_left` behavior where the new equal-time point is inserted before the
  existing one.
- Vlabeler edit count and ratio: empty inputs, empty target, empty pred,
  equal points, move-threshold lower-inclusive/upper-exclusive behavior,
  label mismatch, combined move and label mismatch, unequal lengths truncated
  before DP, repeated target labels changing insertion cost, denominator
  `2 * len(target)`, empty total returning `1.0`, rounding to 6 places, and
  reset after multiple updates.
- IoU: one-point tiers with no spans, overlapping spans, non-overlap, repeated
  marks, missing string phoneme returning `None`, list phoneme lookup
  preserving request order, dict compute returning `0.0` when
  `sum == intersection`, and string compute raising `ZeroDivisionError` for
  `sum == intersection == 0`.
- LCS: empty tiers, no matches, repeated-label tie behavior, exact match index
  pairs, and `get_matched_pairs` point lists.
- Boundary edit distance: equal labels, equal-length label mismatch returning
  `False`, unequal-length LCS fallback, no matched pairs returning `True`,
  `error_phonemes` accumulation, NumPy-scalar-like rounded output where
  observable, and `reset()` clearing distance/phonemes but not
  `error_phonemes`.
- Boundary edit ratio: successful duration accumulation as
  `target[-1].time - target[0].time`, failed update leaving duration unchanged,
  empty target raising `IndexError` after distance update succeeds, compute
  defaulting to `1.0` when duration is zero, no explicit reset implementation
  and therefore `NotImplementedError`.
- Weighted ratio: `counts` increments before distance evaluation, `error`
  increments on failed boundary update, duration/phoneme/count zero defaults,
  denominator correction with `error_phonemes / phonemes`, penalty
  `(error / counts) * 0.2`, rounding to 6 places, and inherited
  `NotImplementedError` reset.

## Split Decision

Confirm `hfa_metrics_core` as one writer unit. Splitting by metric class would
create shared point-tier, LCS, and boundary state fixtures with little benefit,
and the module has no model, file, network, native, or caller-route ownership.

## Rollback

Keep `inference.HubertFA.tools.metrics` as the only runtime owner. Since no
bridge or production route is introduced, rollback is removing the independent
Rust module/tests and fixture checker if this boundary is later re-cut.
