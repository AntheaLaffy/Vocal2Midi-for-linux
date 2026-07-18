# hfa_pyyaml_safe_load_contract - dependency_bootstrap_reviewer

Date: 2026-07-18
Decision: pass

## Findings

No findings.

- Severity: none
- Location: rewrite-in-rust/manifest.yaml:1524
- Issue: The unit boundary is confirmed and writer-ready from a dependency/bootstrap perspective.
- Evidence: The manifest keeps `hfa_pyyaml_safe_load_contract` as a distinct active unit with `inventory_status: confirmed`, legacy runtime ownership, and a policy covering UTF-8 file loading, PyYAML 6.0.3 safe-load values, aliases/merges, duplicate behavior, safe tag rejection, single-document behavior, and structured errors (`rewrite-in-rust/manifest.yaml:1524`, `rewrite-in-rust/manifest.yaml:1527`, `rewrite-in-rust/manifest.yaml:1535`). The previous re-cut record explicitly separated this wider loader from `hfa_config_validation_core` and required tagged fixtures plus a selected parser strategy before writer start (`rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:50`, `rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:56`, `rewrite-in-rust/records/0079-recut-hfa-config-file-contract.md:75`). The current bootstrap closes that gate and allows exactly one future Rust library unit (`rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:105`).
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:50
- Issue: The `saphyr-parser` dependency is appropriately scoped as a lower-layer parser/event/span dependency and does not overclaim PyYAML parity.
- Evidence: The dependency record assigns only YAML event parsing, tags, aliases, and source locations to `saphyr-parser`, while explicitly reserving YAML 1.1 resolution, SafeConstructor values, merge/duplicate behavior, alias identity, tag/document guards, and Python-style error projection for a hand-written adapter (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:50`, `rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:55`, `rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:59`, `rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:93`). The parser crate source supports the claimed lower layer with public `Event`, `Tag`, `Parser`, and spanned receiver concepts (`/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/parser.rs:47`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/parser.rs:101`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/parser.rs:148`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/parser.rs:201`) and `Marker`/`Span` locations (`/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/scanner.rs:62`, `/home/fuurin/.cargo/registry/src/rsproxy.cn-e3de039b2554c837/saphyr-parser-0.0.11/src/scanner.rs:99`). Record 0084 repeats that this is not a direct PyYAML replacement and names the adapter-owned gaps (`rewrite-in-rust/records/0084-select-hfa-pyyaml-parser-layer.md:26`).
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:100
- Issue: Source expansion, kept-legacy decisions, low-level/native boundary, fixture matrix, and rollback route are specific enough for writer handoff.
- Evidence: The first-layer PyYAML 6.0.3 source is indexed and no deeper Python dependency expansion is claimed (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:100`). The hand-written adapter references the exact PyYAML loader/resolver/constructor/composer/error source files (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:118`). Legacy ownership remains explicit for complete SafeLoader and production config loading (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:127`). The low-level boundary excludes PyYAML C/libyaml and generic YAML 1.2 value behavior from the compatibility owner (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:109`). Rollback is to keep `config_utils.load_yaml` and PyYAML as runtime owners (`rewrite-in-rust/manifest.yaml:1553`, `rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:119`). The executable fixture harness projects Python-specific types, identity, and structured errors (`rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py:24`, `rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py:41`, `rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py:156`), and the fixture file contains 47 golden rows (`rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl:1`).
- Required fix: none.

- Severity: none
- Location: rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15
- Issue: The pinned dependency matches the recorded strategy and does not introduce unnecessary bridge/runtime changes.
- Evidence: `v2m-core` pins `saphyr-parser = "=0.0.11"` with default features disabled (`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml:15`), matching record 0084 (`rewrite-in-rust/records/0084-select-hfa-pyyaml-parser-layer.md:20`). The lockfile adds `saphyr-parser` and its normal dependencies, and `v2m-core` depends on it directly (`rewrite-in-rust/rust/Cargo.lock:128`, `rewrite-in-rust/rust/Cargo.lock:225`). The dependency record keeps `bridge_dependencies: []` (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:28`), and the rollback record states production config loading remains Python-owned until a separate promotion decision (`rewrite-in-rust/records/0084-select-hfa-pyyaml-parser-layer.md:88`).
- Required fix: none.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`: passed, validated 47 fixtures.
- `cargo tree --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --edges normal`: passed; `saphyr-parser v0.0.11` appears only as a normal `v2m-core` dependency with `arraydeque` and `thiserror` below it.
- `rg -n 'PyYAML|pyyaml|6\.0\.3' pyproject.toml requirements.txt uv.lock third_party/sources/manifest.json`: confirmed PyYAML is declared, locked as 6.0.3, and vendored under `third_party/sources/pyyaml-6.0.3`.
- `rg -n 'pub enum Event|pub struct Parser|pub struct Span|pub struct Marker|pub struct ScanError|pub struct Tag' ~/.cargo/registry/src -g '*.rs' | rg 'saphyr-parser-0\.0\.11'`: confirmed the local crate source exposes the lower-layer event/span API claimed by the records.
- Reviewed coordinator-provided checks: cargo test, fmt, clippy, rustdoc, Python checker, py_compile, vendored source audit, and `git diff --check` were reported passed before this review.

## Residual Risk

The future writer still has to implement the adapter and prove Rust parity against the 47-case tagged projection. This is correctly captured as remaining writer work, not as an open bootstrap dependency gap (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:41`).

Resource-limit and security-policy fixtures for large aliases, deeply nested inputs, and scanner/parser limits are deferred until production-facing promotion planning (`rewrite-in-rust/dependencies/hfa_pyyaml_safe_load_contract.yaml:43`, `rewrite-in-rust/bootstrap/hfa_pyyaml_safe_load_contract.md:113`). That is acceptable because no runtime owner switch or bridge is part of this unit.

Error message parity may still be the highest-risk writer area because `saphyr-parser` owns only scan/parse events and locations, while PyYAML class/message projection remains adapter-owned. The fixture matrix gives the writer concrete targets, but behavior review and error/tracing review must still verify the Rust implementation.

## Promotion Note

This dependency/bootstrap role does not block writer handoff. The manifest unit boundary is confirmed; it should not be split, merged, deferred, or replaced at this gate. The coordinator may treat the dependency/bootstrap gate as passed, while leaving manifest status changes and later behavior/error reviews to their separate gates.
