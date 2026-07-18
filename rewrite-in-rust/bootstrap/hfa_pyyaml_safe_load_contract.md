# hfa_pyyaml_safe_load_contract Bootstrap

## Boundary

Preserve `config_utils.load_yaml`: open one path as UTF-8 text and call PyYAML
6.0.3 `safe_load`, including the returned Python value type and exact failure
class/message. This is the full loader phase split from deterministic config
validation. The Rust library seam is now reimplemented, while production
runtime ownership remains Python/PyYAML.

## Authoritative Candidate Matrix

The 2026-07-17 dependency probe established these compatibility facts:

- PyYAML 6.0.3 constructs YAML 1.1 `yes`/`on` booleans, `077` octal,
  sexagesimal numbers, dates/datetimes, `!!binary` bytes, `!!set`, `!!omap`,
  `!!pairs`, merge keys, and last-value duplicate replacement; it rejects
  custom tags and multiple documents through `safe_load`.
- `serde_yaml_ng` 0.10.0 direct `Value` differs on `yes`/`on`, `077`,
  sexagesimal, `1e3`, dates, binary/set/merge, duplicates, and custom tags.
- active pure-Rust `rust-yaml` 1.1.0 with forced `%YAML 1.1` is closer for
  booleans, binary, merge, and duplicates, but still differs on octal,
  sexagesimal, float resolution, timestamps, set/omap/pairs, custom tags, and
  multiple documents. Its 1.1.0 source marks key `Yaml` loading paths as
  placeholders and its timestamp constructor as incomplete.

Therefore neither crate is a direct high-level capability replacement. This does
not rule out partial crate reuse: the next bootstrap should evaluate whether a
parser/event crate, especially `rust-yaml`, can own the syntax layer while a
small compatibility adapter implements PyYAML's resolver, constructor,
duplicate/merge, single-document, tag-rejection, and error-projection deltas
from vendored PyYAML source.

The follow-up 2026-07-18 dependency pass selected `saphyr-parser` 0.0.11 as
the lower-layer parser/event crate. It is pinned in
`rewrite-in-rust/rust/crates/v2m-core/Cargo.toml` as:

```toml
saphyr-parser = { version = "=0.0.11", default-features = false }
```

This pin is not a high-level YAML replacement. `saphyr-parser` owns only the
syntax/event/span substrate: parser events, tags, aliases, and source
locations. The writer must implement the PyYAML 6.0.3 compatibility adapter over
that event stream:

- YAML 1.1 implicit resolver rules from
  `third_party/sources/pyyaml-6.0.3/lib/yaml/resolver.py`;
- SafeConstructor value construction, merge flattening, duplicate replacement,
  bytes, sets, omap, pairs, timestamps, and alias identity from
  `third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py`;
- single-document, duplicate-anchor, undefined-alias, and tag behavior from
  `third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py`;
- Python-style structured error projection from
  `third_party/sources/pyyaml-6.0.3/lib/yaml/error.py` and the 56-case tagged
  fixture matrix.

Rejected alternatives for this unit:

- `saphyr` 0.0.11: high-level YAML 1.2 value loading is broader than the
  selected adapter surface.
- `yaml-rust2` 0.11.0: event APIs exist, but its high-level loader rejects
  duplicate keys and `saphyr-parser` exposes a narrower direct parser/span
  surface.
- `rust-yaml` 1.1.0: broader loader/emitter/resolver surface with no advantage
  over the event-only crate for this compatibility adapter.
- `serde_yaml_ng` 0.10.0: high-level serde/libyaml path differs from PyYAML and
  pulls in unsafe/libyaml behavior that this seam does not need.

## Tagged Fixture Harness

`rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py` now executes
the real `config_utils.load_yaml` against
`rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl`.

The checker projects Python runtime values into tagged JSON so fixtures do not
erase PyYAML-specific types:

- call result tags: `ok: true` for values and `ok: false` for structured
  errors;
- scalar tags: `null`, `bool`, decimal-string `int`, structured `float` with
  kind/repr/hex/sign, `str`, `bytes` with hex/base64, `date`, and `datetime`
  with timezone offset seconds;
- container tags: `list`, `tuple`, insertion-ordered `dict`, and sorted
  projected `set`;
- repeated-reference tag: `ref`, used for alias identity and recursive aliases;
- error shape: phase, exception class/message, `context`, `problem`, `note`,
  `context_mark`, `problem_mark`, file `errno`/filename fields, and
  `UnicodeDecodeError` encoding/span/object fields with temporary paths
  normalized.

The 56-case table covers empty and comment-only input, explicit empty document,
nulls, YAML 1.1 boolean and numeric resolution, quoted strings, PyYAML's `1e3`
and `1.0e3` string behavior, negative zero, NaN sign category, timestamps,
spaced and no-space timezone offsets, timestamp field-width and range behavior,
fractional truncation, binary data including non-ASCII constructor rejection,
sets, omap/pairs and malformed variants, unhashable keys, alias identity,
recursive aliases, repeated anchor-shaped text in comments/quoted/block
scalars, merge flattening and merge errors, duplicate last-wins,
bool/int/float key collision, non-string mapping keys, custom
local/global/python-tag rejection, non-specific tag resolution,
multi-document rejection, scanner/parser/composer/constructor errors, invalid
UTF-8, filesystem errors, and repeated stateless loads.

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
```

## Writer Status

Dependency/bootstrap is closed for writer handoff. The writer may implement a
single Rust library unit using `saphyr-parser` only for syntax events and source
spans. Record 0085 adds `v2m-core::hfa_pyyaml` and fixture-driven Rust parity
tests for the tagged projection. Record 0086 expands that projection to 56 rows
after independent review found duplicate-anchor, timestamp, and binary edge
gaps.

The Rust implementation does not promote a generic YAML Value API as PyYAML
parity. It owns the PyYAML 6.0.3 resolver, SafeConstructor, merge/duplicate,
alias identity, single-document/tag, and structured error projection behavior
above the parser event layer.

Independent behavior and error/tracing reruns passed after record 0086 fixed
the initial review findings. The unit is verified as an independent Rust library
seam. Runtime promotion remains separate.

Resource-limit fixtures for large aliases, deeply nested inputs, and
scanner/parser limits are explicitly deferred to production-facing promotion
planning. Current production remains Python/PyYAML-owned and the writer target
is an independent library seam, so resource policy can be specified before an
owner switch without blocking the adapter implementation.

## Kept Legacy And Rollback

PyYAML remains the sole loader/runtime owner. `hfa_config_validation_core` does
not parse YAML and cannot be substituted for this unit. No production route or
bridge changes during bootstrap.
