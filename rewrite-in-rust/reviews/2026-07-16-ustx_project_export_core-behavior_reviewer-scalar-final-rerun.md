# ustx_project_export_core - behavior_reviewer scalar final rerun

Date: 2026-07-16
Decision: pass-with-followups

## Findings

No blocking findings.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:899
- Issue: The requested scalar blocker is closed, but broad arbitrary-string PyYAML parity is still fixture-bound rather than exhaustive. The full USTX fixture table now pins one exponent-like project stem and one binary-looking lyric, while the other requested variants are covered by focused helper assertions and live PyYAML probes instead of separate golden USTX rows. This is acceptable for the current `save_ustx(..., rmvpe_result=None)` behavior rerun because `yaml_scalar` is the single renderer used for project names and lyrics, and the helper tests cover the resolver-active and non-hazard forms requested here.
- Evidence: Legacy accepts note lyrics via `note.lyric or "a"` at `inference/API/ustx_api.py:384`, project names from `filepath.stem` at `inference/API/ustx_api.py:414` and `inference/API/ustx_api.py:447`, then serializes with `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)` at `inference/API/ustx_api.py:460`. Rust routes those same dynamic fields through `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:90`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:149`. The current fixture pins `stem: "1.0e-07"` and `lyric: "0b101"` at `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2`, with quoted PyYAML output at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:1`, `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:242`, and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:274`. Helper assertions cover non-hazard `1e-7`, `-x`, `?x`, `0:01`; resolver-active exponent, signed exponent, binary, octal, hex, and sexagesimal forms; and control characters at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:878` through `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:920`.
- Required fix: Before runtime promotion, consider adding a small scalar-focused golden fixture row for signed exponent, hex/octal, and sexagesimal examples if exact byte-for-byte YAML output for arbitrary filenames and lyrics becomes a promotion requirement.

## Checked Scope

- The prior behavior fail is closed. `looks_like_yaml_number` now includes integer and float resolver guards at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:343`, with binary/hex/octal handling at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:347`, exponent-float handling at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:395`, and sexagesimal detection at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:470`.
- Control-character parity is closed for the selected scalar surface. Double-quoted control escaping is at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:250`, newline-only multiline scalar rendering is at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:243`, and the fixture pins `mix\n\t\r` plus `line\nbreak` at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:298` and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:322`.
- Non-hazard strings remain plain where PyYAML leaves them plain. The Rust tests assert `1e-7`, `-x`, `?x`, and `0:01` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:891`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:892`, `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:900`, and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:910`; the focused PyYAML probe printed those same values as plain scalar output.
- The review stayed inside the confirmed boundary: `save_ustx(..., rmvpe_result=None)` project rendering, with RMVPE pitch curves, filesystem writes, and runtime bridge behavior out of scope per `rewrite-in-rust/bootstrap/ustx_project_export_core.md:5` and rollback still legacy-owned at `rewrite-in-rust/manifest.yaml:917`.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass; 6 tests passed.
- `uv run python -c 'import yaml; values=["1.0e-07","1.0e-7","+1.0e-7","-1.0e-7","1.0e+20","0b101","-0b101","077","0x10","0o77","1:20","0:01","1e-7","-x","?x","line\nbreak","mix\n\t\r"]; ...'`: pass; confirmed PyYAML quotes resolver-active string scalars and leaves `0o77`, `0:01`, `1e-7`, `-x`, and `?x` plain.
- `uv run python -c 'import yaml; project={"name":"line\nbreak","voice_parts":[{"name":"line\nbreak"}]}; ...'`: pass; confirmed PyYAML's root and nested indentation for newline-bearing project names matches the Rust `key_indent` strategy.
- `uv run python -c 'import yaml; paths=[...]; ...'`: pass; `empty_project.ustx` and `edge_project.ustx` load project names and note lyrics as strings, including `1.0e-07`, `0b101`, `mix\n\t\r`, and `line\nbreak`.
- `rg -n '1\\.0e-07|1\\.0e-7|0b101|077|0x10|1:20|0:01|1e-7|-x|\\?x|line|mix' ...`: pass; confirmed the prior failing examples are present in current fixtures, Rust assertions, or prior failure evidence.

## Residual Risk

This review does not claim full PyYAML package parity. It verifies the selected USTX project tree, the dynamic project-name and lyric scalar renderer, and the scalar cases named for this rerun. Non-finite tempo, tick overflow diagnostics, runtime filesystem behavior, skipped-note warning mapping after promotion, and RMVPE-derived pitch curves remain outside this behavior pass.

## Promotion Note

This behavior role no longer blocks coordinator verification for `ustx_project_export_core`. The coordinator should still wait for the separate data/algorithm rerun result after the same scalar fix, because this report covers only the behavior role requested here.
