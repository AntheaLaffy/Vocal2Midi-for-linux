# runtime_device_normalization - dependency_bootstrap_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings after checking the full dependency_bootstrap_reviewer scope.

The manifest unit boundary is confirmed. The reviewed evidence supports keeping
`runtime_device_normalization` scoped to
`inference/device_utils.py::normalize_runtime_device` only, with ONNX Runtime
provider discovery, DirectML provider selection, DXGI adapter enumeration, and
production runtime validation remaining legacy-owned.

## Checks

- `sed -n '1,240p' /home/fuurin/.claude/skills/vocal2midi-rs-review-gate/SKILL.md`: read the required review-gate process completely before reviewing.
- `sed -n '1,240p' README.md`; `sed -n '1,260p' manifest.yaml`; `sed -n '1,240p' resources.md`; `sed -n '1,260p' notes.md`; `sed -n '1,240p' reviews/README.md`: read the required rewrite control-plane context and report format.
- `sed -n '1,220p' records/0001-initialize-rust-rewrite.md`; `sed -n '1,220p' records/0002-add-project-rewrite-skills.md`; `sed -n '1,220p' records/0003-treat-unit-inventory-as-provisional.md`; `sed -n '1,220p' records/0004-index-hand-written-replacement-sources.md`: read the relevant rewrite records.
- `sed -n '1,260p' dependencies/runtime_device_normalization.yaml`: confirmed capability coverage is pure runtime device name normalization, seam is a legacy-owned library seam with no bridge dependencies, inventory impact is confirmed, and ONNX Runtime/DXGI/provider selection are explicitly kept legacy-owned at lines 3-34.
- `sed -n '1,260p' bootstrap/runtime_device_normalization.md`: confirmed the bootstrap boundary excludes `resolve_onnx_providers`, `use_dml`, DXGI enumeration, provider availability, model runtime ownership, PyO3, subprocess, CLI, HTTP, ONNX Runtime, DirectML, and runtime-router work at lines 5-17 and 43-53.
- `sed -n '1,220p' ../inference/device_utils.py`: confirmed the source function only uses `_IS_WINDOWS`, `_DEVICE_ALIASES`, string coercion, strip/lowercase behavior, and default selection at lines 11-27 and 76-82; ONNX Runtime and DXGI/provider logic remain separate at lines 8, 40-73, 103-188, and 195-218.
- `sed -n '1,220p' fixtures/runtime_device_normalization.tsv`: confirmed fixtures cover non-Windows and Windows defaults, `None`, empty string, whitespace, aliases, case/whitespace normalization, unknown values, and explicit-default edges at lines 2-23.
- `sed -n '1,260p' rust/crates/v2m-core/src/device.rs`: confirmed the hand-written Rust replacement implements only device normalization and platform/default handling at lines 1-61 and consumes the parity fixture table in tests at lines 67-155.
- `sed -n '1,80p' rust/crates/v2m-core/Cargo.toml`: confirmed no Rust crate dependency was added for ONNX Runtime, DirectML, DXGI, PyO3, or bridge work.
- `rg -n "onnxruntime|onnxruntime-directml|directml|dxgi|provider" ../pyproject.toml ../uv.lock ../third_party/source_audit.json`: confirmed `pyproject.toml` keeps `onnxruntime` for non-Windows and `onnxruntime-directml` for Windows at lines 18-19; `uv.lock` records `onnxruntime` 1.27.0 and `onnxruntime-directml` 1.24.4 at lines 942-969; `third_party/source_audit.json` references the ONNX Runtime upstream source fallback and no audit errors at lines 1-24.
- `uv run python scripts/audit_vendored_sources.py`: passed from `/home/fuurin/code/Vocal2Midi-for-linux` with 135 Python packages, 41 native-extension packages, 269 covered foreign runtime native binaries, and 0 third-party binary artifacts.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml device`: passed, 5 device tests ok.
- `uv run python rewrite-in-rust/bootstrap/check_runtime_device_normalization.py`: passed with no output.
- `rg -n "from .*v2m|import .*v2m|rewrite-in-rust|rewrite_in_rust|pyo3|PyO3|maturin" inference application gui web_server.py web_task_manager.py scripts pyproject.toml uv.lock`: no matches, supporting that no production Python Rust bridge/import path was introduced.
- `git diff -- inference application gui web_server.py web_task_manager.py scripts pyproject.toml uv.lock`: no production Python or dependency-manifest diff for the reviewed bridge/import surface.

## Residual Risk

This review did not prove behavior parity beyond the dependency/bootstrap scope;
that remains the behavior reviewer role. Windows behavior is fixture-simulated by
toggling the legacy `_IS_WINDOWS` value in the bootstrap check, not exercised on
a native Windows host.

## Promotion Note

This role does not block promotion. The dependency/bootstrap evidence supports a
confirmed unit boundary, a narrow hand-written Rust replacement, no new missing
crate requirement, no ONNX Runtime/DXGI/provider migration in this unit, and no
production bridge/import changes. A coordinator may use this report as
dependency_bootstrap_reviewer evidence for state update decisions, subject to the
separate required behavior review policy.
