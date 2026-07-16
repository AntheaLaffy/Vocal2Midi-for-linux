# ustx_project_export_core - error_tracing_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:197
- Issue: The Rust YAML scalar renderer only quotes empty strings and a small set
  of boolean/null-like words, but `save_ustx` delegates all user-controlled
  project names and lyrics to `yaml.safe_dump(..., allow_unicode=True,
  sort_keys=False)`. Strings such as `stem: value` or `a: b` are valid legacy
  inputs that PyYAML quotes, while Rust would emit them raw through
  `yaml_scalar` from project-name and lyric fields. That can produce invalid or
  semantically different YAML while returning `Ok`, leaving callers with no
  structured error, warning, or test diagnostic.
- Evidence: `inference/API/ustx_api.py:413` builds `name` from
  `filepath.stem`, `inference/API/ustx_api.py:384` uses `note.lyric or "a"`,
  and `inference/API/ustx_api.py:460` serializes via PyYAML. The probe
  `uv run python -c '...'` with stem `stem: value` and lyric `a: b` produced
  `name: 'stem: value'` and `lyric: 'a: b'`. Rust writes project names through
  `yaml_scalar` at `rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:70`
  and `:90`, and lyrics at `:149`; the current fixture table has only
  `empty_project` and `edge_project` rows, so this path is not covered.
- Required fix: Replace the ad hoc scalar renderer with a fixture-proven
  PyYAML-compatible scalar escaping policy for the supported project tree, or
  return a structured `UstxProjectError` when an unsupported scalar is seen.
  Add fixture coverage for YAML-sensitive project names and lyrics before this
  role can pass.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/ustx_project.rs:23
- Issue: `UstxProjectError` is structured only at the variant level and does not
  implement `Display`/`std::error::Error` or carry context such as the offending
  tempo, note index, field, or tick value. This is not yet a runtime mapping
  blocker because filesystem writes and Python caller-facing errors remain
  legacy-owned, but it weakens promotion diagnostics.
- Evidence: the Rust enum has only `InvalidTempo` and `TickOutOfRange`; the
  only explicit error test is `ustx_project_rejects_non_finite_tempo_without_panicking`.
  Targeted legacy probes show non-finite tempo behavior differs by data path:
  empty input with `NaN`/`inf` writes `.nan`/`.inf`, while non-empty input raises
  `ValueError` during tick conversion. Tick overflow and `inf` tempo are not
  separately asserted by current Rust tests.
- Required fix: Before runtime promotion, define the caller-facing mapping for
  `InvalidTempo` and `TickOutOfRange`, add explicit tests for `NaN`, `inf`, and
  tick overflow paths, and include enough context in errors or bridge diagnostics
  to identify the failing field.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ustx_project_export_core.py`:
  pass
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ustx_project`:
  pass, 3 tests
- Targeted legacy `save_ustx` probe for YAML-sensitive stem/lyric strings:
  PyYAML quotes `stem: value` and `a: b`
- Targeted legacy `save_ustx` probe for non-finite tempo: empty input writes
  `.nan`/`.inf`; non-empty input raises `ValueError` during tick conversion

## Residual Risk

The existing fixtures prove the happy-path project shape, skipped-note warning
parity, UTF-8 lyrics, and scalar numeric conversion for two cases only. They do
not prove YAML safety for arbitrary user-controlled stems or lyrics, nor do they
prove edge diagnostics for `inf`, very large ticks, or promotion-time
filesystem/write error mapping.

## Promotion Note

Missing filesystem/write error ownership and skipped-note warning mapping do not
block this reimplementation review by themselves: the manifest, bootstrap, and
record keep `inference.API.ustx_api.save_ustx` as runtime owner, and the Rust
surface intentionally returns YAML plus `skipped_invalid_notes` without printing
or writing files. The YAML scalar handling finding does block this role because
the Rust renderer can currently return successful output that is not
PyYAML-compatible for legal legacy strings.
