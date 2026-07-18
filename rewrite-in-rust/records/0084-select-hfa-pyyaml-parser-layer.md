# 0084 - Select HFA PyYAML Parser Layer

## Context

Record 0083 established the 47-case tagged fixture harness for
`hfa_pyyaml_safe_load_contract`, but deliberately left one dependency gate open:
choose a parser/event crate or justify a fully hand-written parser before writer
work starts.

The compatibility target remains `config_utils.load_yaml`: open a UTF-8 file
and return or raise what Python 3.12 plus PyYAML 6.0.3 `safe_load` returns or
raises. The deterministic `hfa_config_validation_core` unit is already verified
and does not parse YAML.

## Decision

Use `saphyr-parser` 0.0.11 as the Rust lower-layer parser/event/span substrate
for the future `v2m-core::hfa_pyyaml` writer unit.

The crate is pinned in `rewrite-in-rust/rust/crates/v2m-core/Cargo.toml`:

```toml
saphyr-parser = { version = "=0.0.11", default-features = false }
```

This is not a direct PyYAML replacement. The selected ownership split is:

- `saphyr-parser`: syntax scanning/parsing events, tags, anchors/aliases, and
  source spans/markers;
- hand-written adapter: PyYAML 6.0.3 YAML 1.1 resolver, SafeConstructor value
  construction, merge flattening, duplicate replacement, alias identity,
  single-document and tag rejection behavior, and structured Python-style error
  projection;
- legacy Python: production config loading until a separate promotion record
  defines routing, resource limits, user-facing error presentation, and rollback.

## Evidence

`cargo info` showed the current candidates visible to this workspace:

- `saphyr-parser` 0.0.11: pure parser crate, MIT OR Apache-2.0, MSRV 1.85.0,
  YAML 1.2 target, direct parser events and Span/Marker locations;
- `saphyr` 0.0.11: high-level YAML library on top of the parser;
- `yaml-rust2` 0.11.0: YAML 1.2 parser/value crate with event APIs;
- `rust-yaml` 1.1.0: broader YAML 1.2 library with loader/emitter/resolver
  surface and default mmap/preserve-order features;
- `serde_yaml_ng` 0.10.0: serde/libyaml-oriented path.

Source inspection confirmed that `saphyr-parser` exposes `Event`, `Tag`,
`Parser`, `Span`, `Marker`, and `ScanError` without requiring its high-level
loader. `yaml-rust2` exposes `MarkedEventReceiver`, but its high-level loader
rejects duplicate keys while PyYAML keeps the first key object and replaces the
value. `rust-yaml` has a public event surface but brings a larger loader,
emitter, resolver, constructor, and resource-limit surface than this adapter
needs. `serde_yaml_ng` keeps the wrong high-level ownership boundary and pulls
in unsafe/libyaml behavior that is not needed for the selected seam.

Vendored PyYAML source remains the semantic reference:

- `third_party/sources/pyyaml-6.0.3/lib/yaml/resolver.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/error.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/loader.py`

## Writer Handoff

The writer may implement exactly `hfa_pyyaml_safe_load_contract` as an
independent Rust library unit. It must use `saphyr-parser` only for events and
locations, then prove the adapter with the tagged fixture matrix:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
```

The writer must not claim parity through `serde_yaml_ng`, `rust-yaml`,
`yaml-rust2`, `saphyr`, or any generic YAML `Value` alone.

## Deferred Promotion Work

Resource-limit fixtures for large aliases, deeply nested inputs, and
scanner/parser limits are deferred to production-facing promotion planning. That
is acceptable for writer start because no production owner switch is happening
in this unit. Promotion must revisit these limits before Rust can become the
runtime owner for config loading.

## Rollback

Rollback remains keeping `config_utils.load_yaml`, PyYAML 6.0.3, and all HFA
config loading paths Python-owned. Removing the pinned parser dependency before
writer work would not change current user workflows.
