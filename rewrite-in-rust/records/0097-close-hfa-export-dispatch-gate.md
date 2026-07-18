# 0097 - Close HFA Export Dispatch Gate

Date: 2026-07-18

## Decision

`hfa_export_dispatch_contract` is verified.

The Rust implementation remains an independent `v2m-core::hfa_export_dispatch`
library seam with injected sinks. Python `Exporter.export`,
`InferenceBase.export`, ONNX CLI selection, and `hfa_api` artifact routing
remain the runtime owners. No bridge, production caller route, filesystem
executor, model execution path, or HTK/TextGrid serializer ownership changed in
this gate.

## Evidence

Final fixture state:

- `rewrite-in-rust/fixtures/hfa_export_dispatch_contract.jsonl` contains 18
  Python 3.12 injected-sink cases covering list, tuple, string, mapping, empty,
  and `None` format inputs; duplicates; unknown and case variants; fixed
  TextGrid-before-HTK order; `Exporter.export(None)` `TypeError`;
  `InferenceBase.export` `None` default versus explicit empty formats;
  output-folder forwarding; return values; repeated calls; exact final status
  print; and downstream error short-circuiting with no status after failure.

Required review reports:

- `rewrite-in-rust/reviews/2026-07-18-hfa_export_dispatch_contract-dependency_bootstrap_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_export_dispatch_contract-behavior_reviewer.md`
- `rewrite-in-rust/reviews/2026-07-18-hfa_export_dispatch_contract-error_tracing_reviewer.md`

Dependency/bootstrap and behavior passed without findings. Error/tracing passed
with a low promotion-planning follow-up: `HfaPlanningExportSink` preserves
downstream exception type/message but drops HTK/TextGrid downstream
`partial_plan` diagnostics. This does not break the current legacy-owned
dispatch fixture contract. Before promotion planning or any bridge/effect
executor treats `HfaPlanningExportSink` as a fully diagnosable effect-plan
boundary, preserve the failing format plus downstream partial plan or explicitly
record why exception type/message projection is sufficient.

## Verification

Coordinator checks run before closeout:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml hfa_export_dispatch_contract
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path rewrite-in-rust/rust/Cargo.toml --no-deps
uv run python -m py_compile inference/HubertFA/tools/export_tool.py inference/HubertFA/tools/infer_base.py rewrite-in-rust/bootstrap/check_hfa_export_dispatch_contract.py
uv run python scripts/audit_vendored_sources.py
git diff --check
```

All passed after fixing one clippy-only `let_and_return` finding in a Rust test
helper.

## Reversal

Rollback remains keeping Python `Exporter.export`, `InferenceBase.export`, ONNX
CLI selection, and `hfa_api` artifact routing as runtime owners. Because no
production route changed, reversal is removing the independent Rust dispatch
module, fixture, checker, and manifest verification entries if this seam is
later re-cut.
