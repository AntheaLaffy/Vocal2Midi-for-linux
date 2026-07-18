# 0083 - Bootstrap HFA PyYAML Safe Load Contract

Date: 2026-07-18

## Context

Record 0079 split deterministic HFA config validation from full PyYAML loader
behavior. Record 0082 closed the validation child. The next manifest unit,
`hfa_pyyaml_safe_load_contract`, remains the full `config_utils.load_yaml`
contract: open one UTF-8 file and return or raise exactly what PyYAML 6.0.3
`safe_load` exposes to Python callers.

This unit is broader than the JSON-compatible `check_configs` seam. PyYAML can
construct non-JSON Python values, preserve alias identity, create recursive
objects, accept non-string mapping keys, flatten merges, replace duplicate keys,
and raise marked scanner/parser/composer/constructor errors.

## Bootstrap Added

Added a Python-only golden harness:

- `rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py`
- `rewrite-in-rust/fixtures/hfa_pyyaml_safe_load_contract.jsonl`

The checker calls the real `inference.HubertFA.tools.config_utils.load_yaml`
under Python 3.12 and PyYAML 6.0.3. It projects runtime values into tagged JSON
instead of losing type information through a generic JSON value model.

The projection records:

- call results through `ok: true` and `ok: false`;
- scalar values: `null`, `bool`, decimal-string `int`, categorized/repr/hex/sign
  `float`, `str`, hex/base64 `bytes`, `date`, and `datetime` with timezone
  offset seconds;
- containers: `list`, `tuple`, insertion-ordered `dict`, and sorted projected
  `set`;
- identity and cycles through `id` and `ref`;
- errors with phase, exception class/message, `context`, `problem`, `note`,
  `context_mark`, `problem_mark`, file `errno`/filename fields, and
  `UnicodeDecodeError` encoding/span/object fields.

The fixture table currently contains 47 golden cases covering empty and
comment-only input, explicit empty document, nulls, YAML 1.1 booleans, quoted
strings, binary/octal/hex/decimal/sexagesimal integers, PyYAML float resolver
behavior including `1e3` and `1.0e3` remaining strings, negative zero, NaN sign
category, timestamps, fractional truncation, binary data, sets, omap, pairs,
malformed omap/pairs, unhashable keys, alias identity, recursive aliases, merge
flattening and merge errors, duplicate last-wins, bool/int/float key collision,
non-string mapping keys, custom local/global/python-tag rejection, non-specific
tag resolution, multi-document rejection, scanner/parser/composer/constructor
errors, invalid UTF-8, filesystem errors, and repeated stateless loads.

## Dependency Evidence

PyYAML is a direct first-layer dependency in `pyproject.toml` and
`requirements.txt`, locked as 6.0.3 in `uv.lock`, and indexed in
`third_party/sources/manifest.json` at
`third_party/sources/pyyaml-6.0.3`.

The relevant vendored sources are:

- `third_party/sources/pyyaml-6.0.3/lib/yaml/loader.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/resolver.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/constructor.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/composer.py`
- `third_party/sources/pyyaml-6.0.3/lib/yaml/error.py`

No current Rust workspace crate or lockfile entry provides a YAML parser for
this unit. Existing third-party cargo vendor references to `serde_yaml` and
`unsafe-libyaml` are incidental dev-dependency lock evidence inside unrelated
vendored Python-package crates; they are not selected or available as current
workspace dependencies.

## Decision

At this checkpoint, keep `hfa_pyyaml_safe_load_contract` active for
dependency/bootstrap work but still `inventory_status: provisional` and not
writer-ready. Record 0084 supersedes this checkpoint after selecting the
parser/event strategy.

The tagged fixture matrix closes the first bootstrap gap from record 0079. The
remaining gate is choosing and pinning a Rust strategy:

1. a parser/event crate that can own syntax/source-mark handling, plus a
   PyYAML-compatible resolver/constructor/merge/tag/document/error adapter; or
2. a fixture-bound hand-written parser path justified against the vendored
   PyYAML sources if crate reuse is less verifiable.

Do not add a generic Rust YAML `Value` replacement, and do not promote this as
config loading parity.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py
uv run python -m py_compile rewrite-in-rust/bootstrap/check_hfa_pyyaml_safe_load_contract.py inference/HubertFA/tools/config_utils.py
git diff --check
```

The checker validates all 47 cases under PyYAML 6.0.3.

## Reversal

Rollback remains keeping `config_utils.load_yaml` and PyYAML 6.0.3 as runtime
owners. No Rust crate, bridge, production import, model config loading path,
GUI, Web, CLI, or inference route changed.

## Follow-up

Record 0084 closes the parser/event strategy gate left open here by pinning
`saphyr-parser` 0.0.11 as the lower-layer syntax dependency and keeping PyYAML
resolver/constructor/error behavior adapter-owned.
