# 0079 - Re-cut HFA Config File Contract

Date: 2026-07-17

## Context

Record 0074 separated HFA config behavior from G2P and export, but left one
provisional `hfa_config_file_contract_core` containing both
`config_utils.load_yaml` and `check_configs`. Dependency expansion now proves
that these phases have different capability and fixture boundaries.

`check_configs` is a small deterministic filesystem validator after a vocab
loader outcome exists. `load_yaml` delegates to PyYAML 6.0.3 `safe_load`, whose
YAML 1.1 resolution, Python-specific constructed values, tags, document rules,
source-marked errors, and file/decode behavior cannot be represented honestly as
the same small validator.

## Dependency Evidence

The project declares PyYAML in `pyproject.toml` and `requirements.txt`, locks
6.0.3 in `uv.lock`, and vendors its source at
`third_party/sources/pyyaml-6.0.3`. `SafeLoader` composes the resolver,
constructor, and single-document composer under that tree.

The authoritative 2026-07-17 candidate probe found:

- PyYAML constructs YAML 1.1 `yes`/`on` booleans, legacy octal and sexagesimal
  numbers, dates/datetimes, bytes, sets, ordered mappings/pairs, aliases/merge
  keys, and last-value duplicate keys, while rejecting custom tags and multiple
  documents through `safe_load`;
- `serde_yaml_ng` 0.10.0 direct `Value` disagrees on booleans, numeric and float
  resolution, dates, binary/set/merge, duplicates, and custom tags;
- pure-Rust `rust-yaml` 1.1.0 forced to YAML 1.1 is closer for a subset, but
  still disagrees on octal, sexagesimal, float resolution, timestamps,
  set/omap/pairs, custom tags, and multiple documents; inspected 1.1.0 source
  also labels key load paths as placeholders and timestamp construction as
  incomplete.

Neither crate is a drop-in high-level capability replacement. That does not mean
the final loader should be fully hand-written. The later loader bootstrap should
evaluate whether a Rust parser/event crate can own the lower syntax layer, with
the PyYAML-specific resolver, constructor, duplicate/merge behavior, tag
rejection, single-document enforcement, and error projection implemented as a
compatibility adapter from vendored Python source.

## Decision

Replace the provisional mixed manifest unit with two explicit units:

1. `hfa_config_validation_core` is confirmed and writer-ready. It covers the
   real `check_configs` control flow over an injected JSON-compatible loader
   value or structured error. Its 34-case JSONL checker monkeypatches the real
   loader and proves suffix rendering, existence/loader order, dynamic shape
   errors, dictionary path validation, unparsed config contents, exact errors,
   and repeated calls.
2. `hfa_pyyaml_safe_load_contract` remains planned and provisional. It retains
   the complete file, resolver, constructor, tag, document, Python value, and
   error contract. It requires a tagged value projection, executable full
   golden matrix, and a selected parser/compatibility strategy before writer
   work. A partial Rust parser crate plus compatibility adapter is a valid
   strategy when fixtures prove the lower layer is trustworthy.

The validation child is an intermediate independently verifiable phase, not
final config completion. It must not be promoted as `load_yaml` parity, and
cross-unit composition must later expand beyond JSON-compatible values for
dates, datetimes, bytes, sets, ordered pairs, and arbitrary mapping keys.

## Writer Route

The next writer may implement exactly `hfa_config_validation_core` in the
independent Rust workspace. It must consume the existing 34-case fixture, keep
Python as runtime owner, avoid adding a YAML dependency or bridge, and request
dependency/bootstrap, behavior, and error/tracing review.

No writer may start `hfa_pyyaml_safe_load_contract` until its bootstrap record's
tagged value/error fixture and parser compatibility requirements are satisfied.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
uv run python -m py_compile inference/HubertFA/tools/config_utils.py rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
uv run python -c "import yaml; assert yaml.__version__ == '6.0.3'"
uv run python scripts/audit_vendored_sources.py
git diff --check
```

## Reversal

Keep `config_utils.load_yaml`, `check_configs`, and
`InferenceOnnx.load_config` as runtime owners. The re-cut changes only rewrite
inventory and executable fixtures; no Python import, Rust production module,
bridge, Cargo dependency, model path, GUI, Web, or CLI route changes.
