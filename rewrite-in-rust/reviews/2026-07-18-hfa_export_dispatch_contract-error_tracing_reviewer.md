# hfa_export_dispatch_contract - error_tracing_reviewer

Date: 2026-07-18
Decision: pass-with-followups

## Findings

- Severity: low
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:192`
- Issue: `HfaPlanningExportSink` drops downstream partial-plan diagnostics when it wraps TextGrid or HTK planner failures into `HfaExportDispatchError`.
- Evidence: The accepted TextGrid and HTK planner error surfaces both expose a failure object with `error` plus `partial_plan` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_textgrid_export.rs:72`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_htk_export.rs:73`), and their error-tracing reviews treated those partial plans as part of the diagnosability evidence (`rewrite-in-rust/reviews/2026-07-18-hfa_textgrid_export_core-error_tracing_reviewer-rerun.md:15`, `rewrite-in-rust/reviews/2026-07-18-hfa_htk_label_export_core-error_tracing_reviewer-rerun3.md:14`). The dispatch planning adapter maps only `failure.error.exception_type()` and `failure.error.message()` into `HfaExportDispatchError` for TextGrid and HTK failures, discarding `failure.partial_plan` (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:192`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:204`). This does not break the current legacy-compatible fixture projection, which only asserts exception type/message and call/status behavior (`rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:11`, `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:12`, `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:17`), but it weakens diagnostics if the composed planning sink is later used as promotion evidence for failing multi-prediction exports.
- Required fix: Before promotion planning or any bridge/effect executor consumes `HfaPlanningExportSink`, add a dispatch-level failure shape that preserves the failing format and downstream partial plan, or explicitly document that this adapter intentionally projects only legacy exception type/message and cannot report partially planned side effects.

- Severity: none
- Location: `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:73`
- Issue: No blocking error/tracing issue found in the selected dispatch/default/status seam.
- Evidence: `HfaExportDispatchError` carries a structured Python exception type plus message and implements `Display`/`Error` without adding unrelated context (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:73`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:80`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:93`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:104`). `Exporter.export(None)` projects the legacy `TypeError` before any sink call (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:44`, `rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:89`; Python source at `inference/HubertFA/tools/export_tool.py:65`). Downstream sink failures propagate immediately through `?`, preserving TextGrid-before-HTK short-circuit behavior (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:121`; Python source at `inference/HubertFA/tools/export_tool.py:65`). `InferenceBase.export` status output is modeled as a returned print line only after downstream success (`rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs:144`; Python source at `inference/HubertFA/tools/infer_base.py:240`), and the fixtures cover final status suppression after downstream failure (`rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl:17`).
- Required fix: None for this gate.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py`: passed; validated 18 `hfa_export_dispatch_contract` fixtures against legacy Python.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_export_dispatch_contract`: passed; 1 focused `v2m_core` fixture-parity test passed, with 117 filtered out; `v2m_quant_bridge` had 0 matching tests.
- `rg -n "std::fs|fs::|File::|OpenOptions|create_dir|write_all|println!|eprintln!|dbg!|log::|tracing::|Command::|stderr|stdout" rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs`: no matches; the module adds no filesystem, process, logging, or runtime print side effects.
- `rg -n "panic!|todo!|unimplemented!|unwrap\\(|expect\\(" rewrite-in-rust/rust/crates/v2m-core/src/hfa_export_dispatch.rs`: inspected; all matches are inside `#[cfg(test)]` fixture decoding/assertion code.
- `git diff --check -- reviews/2026-07-18-hfa_export_dispatch_contract-error_tracing_reviewer.md rust/crates/v2m-core/src/hfa_export_dispatch.rs bootstrap/check_hfa_export_dispatch_contract.py fixtures/hfa_export_dispatch_contract.jsonl dependencies/hfa_export_dispatch_contract.yaml bootstrap/hfa_export_dispatch_contract.md records/0096-implement-hfa-export-dispatch-contract.md manifest.yaml`: passed after this report was written.

## Residual Risk

This review covers only `hfa_export_dispatch_contract` as `error_tracing_reviewer`. It does not re-review behavior parity beyond error/status evidence, dependency/bootstrap sufficiency, Rust style, architecture, data/algorithm choices, or product ergonomics. Real filesystem IO, status printing, API artifact copying, ONNX/HubertFA execution, and production caller routing remain intentionally legacy-owned. The current fixtures project downstream failures through injected sink type/message only, so they do not prove preservation of downstream partial plans at the composed planning-sink layer.

## Promotion Note

This role does not block coordinator consumption of the current reimplemented unit, but it leaves a follow-up for promotion planning: do not treat `HfaPlanningExportSink` as a fully diagnosable effect plan boundary until downstream partial-plan loss is resolved or explicitly accepted. This report does not mark the manifest verified and did not edit production Rust or Python code.
