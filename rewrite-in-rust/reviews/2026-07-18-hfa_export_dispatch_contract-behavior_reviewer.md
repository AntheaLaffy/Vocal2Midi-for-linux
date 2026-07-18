# hfa_export_dispatch_contract - behavior_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

Behavior evidence:

- `inference/HubertFA/tools/export_tool.py:65` dispatches by Python membership, checks lowercase `textgrid` before lowercase `htk`, ignores duplicates/unknowns through membership semantics, and naturally raises `TypeError` for `None`.
- `inference/HubertFA/tools/infer_base.py:240` defaults only `output_format is None` to `['textgrid']`, forwards `output_folder`, delegates to `Exporter.export`, and prints final status only after successful downstream completion.
- `inference/HubertFA/onnx_infer.py:84` lowercases the Click choice before passing a one-item list, and `inference/API/hfa_api.py:158` reaches this seam with `['textgrid']`; both remain caller context rather than runtime ownership changes.
- `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:1` contains 18 Python-generated injected-sink cases covering list, tuple, string, mapping, empty, `None`, duplicates, unknown and case variants, fixed TextGrid-before-HTK order, default/status behavior, output-folder forwarding, repeated calls, and downstream short-circuiting.
- `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:121` preserves `Exporter.export` dispatch order and error propagation against an injected sink.
- `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:144` preserves the `InferenceBase.export` default/status policy by treating absent format as `['textgrid']`, leaving explicit empty formats as no-op, and returning the status line only after success.
- `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:403` runs the shared JSONL fixtures as the focused Rust parity test.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py`: pass, `validated 18 hfa_export_dispatch_contract fixtures`.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_export_dispatch_contract`: pass, `1 passed; 0 failed; 117 filtered out` in `v2m_core`; bridge binary had `0` matching tests.

## Residual Risk

This review covers behavior parity for the selected public dispatch/default/status seam only. It does not re-review HTK/TextGrid serializer internals, dependency/bootstrap choices, error/tracing quality, production bridge design, or promotion readiness. The repository worktree was already dirty with related rewrite files; no production Rust/Python code or manifest state was edited by this review.

## Promotion Note

The `stage_behavior_reviewer` role passes and does not block coordinator state update for behavior. Do not mark the manifest verified from this report alone; this unit still lists `dependency_bootstrap_reviewer` and `error_tracing_reviewer` as required review roles in `manifest.yaml`.
