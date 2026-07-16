# ustx_project_export_core - data_algorithm_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: The hand-written YAML scalar renderer is too narrow for in-scope user data. `render_ustx_project` accepts arbitrary `project_name` and note `lyric` strings, but `yaml_scalar` only quotes empty strings plus a small boolean/null subset and otherwise emits the raw value. Legacy `save_ustx` serializes the selected project tree with `yaml.safe_dump(..., allow_unicode=True, sort_keys=False)`, so PyYAML quotes or reformats additional plain-scalar hazards such as `yes`, `no`, numeric-looking strings, dates, `a: b`, `#tag`, and multiline text. Rust output can therefore be a parity mismatch, type-changing YAML, or invalid YAML for inputs that are inside the current project export surface.
- Evidence: `inference/API/ustx_api.py:413` builds `name` and note `lyric` from filepath stem and note data, then writes with PyYAML at `inference/API/ustx_api.py:460`. PyYAML source routes `safe_dump` through `SafeDumper` in `third_party/sources/pyyaml-6.0.3/lib/yaml/__init__.py:263`, preserves mapping order when `sort_keys=False` via `represent_mapping` in `third_party/sources/pyyaml-6.0.3/lib/yaml/representer.py:103`, represents strings as YAML string scalars at `third_party/sources/pyyaml-6.0.3/lib/yaml/representer.py:147`, and chooses quoted styles after scalar analysis in `third_party/sources/pyyaml-6.0.3/lib/yaml/emitter.py:494` and `third_party/sources/pyyaml-6.0.3/lib/yaml/emitter.py:626`. A direct legacy probe printed `name: 'yes'` and `lyric: 'a: b'`; current Rust would emit `name: yes` and `lyric: a: b` because neither value matches the explicit quote list at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:202`.
- Required fix: Either implement a PyYAML-compatible scalar subset for all project-name and lyric values accepted by this unit, or add a documented API precondition plus validation/error behavior that excludes unsupported scalar values. Add golden fixtures for at least one resolver-sensitive value (`yes` or `1`) and one syntax-sensitive value (`a: b` or leading `#`) before promotion.

## Checked Scope

- Numeric conversion: Legacy tick conversion is `int(round(seconds * tempo * 8.0))` at `inference/API/ustx_api.py:26`; Rust uses explicit half-even rounding at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:160` and `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:174`. The fixture at `rewrite-in-rust/fixtures/ustx_project_export_core.jsonl:2` covers a 0.5 tick tie, a 2.5 tick tie, minimum duration, and a 61.5 pitch tie; the golden YAML shows the expected positions, duration, and tone at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:247`.
- Filtering and ordering: Legacy `_finite_notes` filters non-finite onset/offset/pitch and `offset <= onset` at `inference/API/ustx_api.py:30`; Rust mirrors this at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:153`. Rust uses stable `sort_by` after filtering at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:54`, matching Python's stable `sorted(..., key=lambda n: n.onset)` at `inference/API/ustx_api.py:370`.
- Tone clamp and duration: Legacy tone conversion uses `int(np.clip(round(note.pitch), 0, 127))` at `inference/API/ustx_api.py:377`; Rust clamps through `clamp_tone` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:164`. NumPy source evidence shows `np.clip` dispatches to a method/umath clip path at `third_party/sources/numpy-1.26.4/numpy/core/fromnumeric.py:2101` and `third_party/sources/numpy-1.26.4/numpy/core/_methods.py:90`. The fixture validates low and high clamps at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:249` and `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:297`.
- Project structures: The fixed expression, track, note pitch, vibrato, empty curves, and wave-part structures in Rust are static mirrors of the legacy dictionary assembled at `inference/API/ustx_api.py:413`. Empty voice-part duration is pinned at `rewrite-in-rust/fixtures/ustx_project_export_core/empty_project.ustx:241`; non-empty max-end duration is pinned at `rewrite-in-rust/fixtures/ustx_project_export_core/edge_project.ustx:241`.
- Complexity: Both implementations perform one filter pass, one stable sort, and one render pass over notes, plus fixed-size project skeleton emission. I found no algorithmic complexity regression in the narrowed `rmvpe_result=None` surface.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`: pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`: pass, 3 tests
- `uv run python -c "import yaml; vals=['yes','no','on','off','true','false','1','2026-07-16','a: b','#tag','multi\\nline','你']; print(yaml.safe_dump({'values': vals}, allow_unicode=True, sort_keys=False), end='')"`: pass; confirms PyYAML quotes resolver-sensitive and syntax-sensitive scalars while leaving the UTF-8 scalar plain.
- `uv run python -c "... save_ustx(... lyric='a: b') ..."`: pass; confirms legacy USTX output uses `name: 'yes'` and `lyric: 'a: b'`.
- `uv run python -c "... print([(v, int(round(v))) ...]); print([int(np.clip(round(v),0,127)) ...])"`: pass; confirms Python half-even tick/tone tie behavior for representative positive and negative ties.

## Residual Risk

The fixture table has only two cases. It does not separately pin same-onset valid-note ordering, all non-finite positions of the `(onset, offset, pitch)` tuple, negative-time rounding, exponent-style tempo formatting, large tick bounds, or every PyYAML scalar style. The scalar issue above is a concrete mismatch, not only residual risk.

## Promotion Note

This role blocks promotion. Numeric filtering, half-even rounding, min duration, tone clamp, voice-part duration, fixed project data structures, and complexity are acceptable for the narrowed surface, but YAML scalar handling must be fixed or explicitly rejected before `ustx_project_export_core` can be treated as data/algorithm reviewed.
