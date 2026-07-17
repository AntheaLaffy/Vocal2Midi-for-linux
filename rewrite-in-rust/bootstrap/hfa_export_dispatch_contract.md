# hfa_export_dispatch_contract Bootstrap

## Boundary

Cover `Exporter.export` and `InferenceBase.export` selection/default behavior
using injected HTK/TextGrid sinks. Membership checks are case-sensitive and
container-polymorphic, TextGrid is always called before HTK, unknown/duplicate
formats are ignored, and `None` fails in `Exporter.export`. `InferenceBase`
converts only `None` to `['textgrid']` and prints its final status only after
downstream success.

## Ordering And Prerequisites

Implement after `hfa_htk_label_export_core` and `hfa_textgrid_export_core` so
dispatch composes verified planners rather than absorbing their algorithms.
ONNX CLI lowercasing and `hfa_api` TextGrid artifact copying remain caller
evidence, not dependencies. No Click, TextGrid, filesystem, model, bridge, or
runtime-router dependency is needed by the dispatch policy.

## Fixture Contract

Use call-recording/error-injecting sinks. Cover list, tuple, string, mapping,
empty, and `None` inputs; duplicates; unknown/case variants; fixed call order;
downstream short-circuit; returns; repeated calls; `InferenceBase` default versus
explicit empty; output-folder forwarding; and exact status prints.

## Rollback

Keep `Exporter.export`, `InferenceBase.export`, ONNX CLI selection, and API
artifact routing as production owners.
