# quantization_bayesian_core Bootstrap

## Boundary

`quantization_bayesian_core` covers only Bayesian quantization behavior from
`inference/quant/quantization.py`:

- `_resolve_bayes_shift_limit`
- `_filter_bayes_candidate_pairs`
- `_build_bayes_candidate_pairs`
- `_estimate_segment_phase_center`
- `_segment_split_indices_bayesian`
- `_metrical_position_penalty`
- `_note_value_penalty`
- `_preferred_sv_duration`
- `_build_piece_specific_priors`
- `_bayes_local_cost`
- `_decode_segment_bayesian`
- `_quantize_notes_bayesian`

The compatibility surface is:

- no-op behavior for empty note lists and `quantization_step <= 0`
- in-place sorting by note onset when quantization runs
- raw note-pair construction with `raw_end = max(onset + 1, offset)` and falsey
  lyrics normalized to `""`
- gap annotation with non-negative gaps from previous raw note end
- Bayes candidate shift limits from factor, floor, cap, and Python half-even
  rounding
- Bayes candidate filtering, including fallback to the best candidate by
  combined start/end shift and duration distance
- short-duration candidate construction on the half grid, bounded by minimum
  `30`, and normal full-grid candidate construction
- phase-center estimation by median raw shift, mean absolute spread, phase
  median/spread, and late-pullback policy
- Bayesian segment splitting by large raw gap and hard maximum segment length
- metrical position, note-value, preferred-SV-duration, segment-center, motif
  duration/gap/phase, and asymmetric local cost terms
- piece-specific priors grouped by duration multiple, gap multiple, and tie
  lyric, with motif-count threshold, capped strength, and median preferred
  duration/gap/phase values
- per-segment Bayesian DP with non-overlap predecessor filtering, fallback to
  overlapping predecessors, strict first-min tie behavior, and traceback
- final overlap repair after segment decode
- onset/offset mutation only; pitch and lyric metadata stay with the sorted note
  objects

`quantize_notes` public dispatch, activation policy, simple mode, smart duration
DP, phrase DP/asymmetric quantization, GUI/Web/application defaults, pipeline
routing, and runtime promotion stay legacy-owned or separately verified.

## Dependency Expansion

`inference/quant/quantization.py` imports:

- stdlib: `__future__.annotations`, `typing.Any`
- third party: `numpy`

The selected Bayesian path uses NumPy for small-array `median`, `mean`, `abs`,
and scalar `ceil` behavior. It does not need BLAS kernels, ndarray as a public
data structure, model runtime, ONNX Runtime, Torch, PyQt, Flask, or an
FFI/service bridge. Rust should implement the behavior directly with `Vec`,
sorting, sums, explicit first-min scans, and existing quantization helper
functions.

Dependency evidence:

- `pyproject.toml` and `requirements.txt` include `numpy<2.0.0`.
- `uv.lock` records `numpy==1.26.4`.
- `third_party/sources/manifest.json` records
  `third_party/sources/numpy-1.26.4`.
- `third_party/native_sources/manifest.json` records native coverage for
  NumPy/SciPy-linked runtime binaries, but this unit does not need BLAS or array
  kernels.
- `third_party/source_audit.json` reports all foreign runtime native binaries
  covered and no remaining `third_party` binary artifacts.

No NumPy, ndarray, PyO3, subprocess, CLI, HTTP, or runtime-router dependency
should be added for this unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- runtime owner: legacy Python
- bridge dependencies: none

No production Python caller should import Rust output until
`quantization_pipeline_promotion` chooses and verifies a bridge.

## Fixture Harness

Before writer work, add durable parity tables and a Python checker:

```text
rewrite-in-rust/fixtures/quantization_bayesian_helpers.tsv
rewrite-in-rust/fixtures/quantization_bayesian_core.tsv
rewrite-in-rust/bootstrap/check_quantization_bayesian_core.py
```

The helper table should cover:

- `_resolve_bayes_shift_limit` floor, cap, and half-even rounding behavior
- `_filter_bayes_candidate_pairs` accepted candidates, empty candidates, and
  fallback best-by-shift-then-duration ranking
- `_build_bayes_candidate_pairs` short-note half-grid and normal full-grid paths
- `_estimate_segment_phase_center` empty segment, normal center/spread strength,
  and late-pullback threshold behavior
- `_segment_split_indices_bayesian` empty input, large-gap split, hard max split,
  and no `"-"` lyric tie suppression
- `_metrical_position_penalty`, `_note_value_penalty`, and
  `_preferred_sv_duration`
- `_build_piece_specific_priors` motif count below six, exactly six, strength
  cap, duration/gap/tie grouping, median preferred duration/gap/phase, and
  fallback prior shape
- `_bayes_local_cost` Bayes asymmetry, metrical penalty, note-value penalty,
  preferred duration prior, segment-center penalty, and motif prior terms
- `_decode_segment_bayesian` non-overlap predecessor filtering, fallback to
  overlapping predecessors, first-min tie behavior, final min tie behavior, and
  traceback

The sequence table should use the existing note encoding:

```text
case_id<TAB>tempo<TAB>step<TAB>input_notes<TAB>expected_notes
```

Each note is encoded as:

```text
onset,offset,pitch,lyric
```

Notes are joined with `|`; `__empty__` represents an empty lyric.

The golden sequence table must cover:

- empty input no-op
- `quantization_step <= 0` no-op without sorting or metadata changes
- sorting by onset and metadata preservation when quantization runs
- non-default positive tempo and step
- short-note fine-grid candidate path
- prior-driven repeated motif behavior
- phase-center normal and late-pullback behavior
- segmentation by large raw gap
- overlap repair where decoded start is clamped to the previous fixed end
- minimum end extension where `end_tick <= start_tick`

## Repeated-Call Behavior

The function mutates the provided note list in place. For a fixed note sequence,
tempo, and positive quantization step, repeated calls starting from the same
original input must produce the same sorted and quantized sequence. The function
must not depend on model state, GUI/Web state, filesystem state, export
settings, or global runtime configuration.

Until bridge/promotion work changes the public Rust boundary, this unit assumes
finite note timings and positive finite tempo when quantization runs. Promotion
work must add explicit validation or error mapping before Rust owns calls that
may receive `NaN`, infinities, overflow-sized ticks, or non-positive tempo.

## Rollback

Rollback is keeping all production imports unchanged:

```text
inference.quant.quantization._quantize_notes_bayesian
```
