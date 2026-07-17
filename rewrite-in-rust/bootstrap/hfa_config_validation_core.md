# hfa_config_validation_core Bootstrap

## Boundary

Cover only `config_utils.check_configs` after separating the loader result from
YAML parsing. The legacy checker remains the executable reference: the harness
monkeypatches `config_utils.load_yaml` to return or raise the fixture-supplied
outcome, then calls the real `check_configs` against a temporary model folder.

Include:

- Python f-string suffix rendering for strings and `None`;
- vocab then config existence order and exact assertion paths;
- loader invocation only after both existence checks;
- JSON-compatible top-level and `dictionaries` value shapes with exact
  representative Python attribute/type errors;
- ordered `None` skipping and dictionary path validation over empty, file,
  directory, nested, absolute, relative, and Unicode values;
- the fact that config content is never parsed;
- loader error pass-through and uncached repeated calls.

Exclude YAML text/byte loading, implicit resolvers, tags, aliases/merges,
Python-specific YAML value construction, YAML parser errors, VERSION/config JSON
assignment, ONNX/model behavior, and all production routing.

## Seam

- kind: independent Rust library
- runtime owner: legacy Python
- bridge dependencies: none
- input: model directory, nullable/string suffix, and an injected loader outcome
- loaded value domain for this first seam: JSON-compatible null, bool, number,
  string, list, and string-key mapping
- loader failure: opaque structured exception type/message, propagated unchanged
- output: unit or structured compatibility error retaining operation and path
- repeated calls: stateless and uncached

This domain is intentionally not the final config/YAML contract. Before
promotion, composition with `hfa_pyyaml_safe_load_contract` must model the
additional Python values PyYAML can construct and rerun cross-unit fixtures.

## Fixture Harness

`rewrite-in-rust/fixtures/hfa_config_validation_core.jsonl` contains 34 cases.
Each case declares suffix, temporary files/directories, injected loader value or
error, repeat count, exact call results, and loader paths. The checker restores
the real loader after every case and normalizes only the temporary root:

```bash
uv run python rewrite-in-rust/bootstrap/check_hfa_config_validation_core.py
```

The table covers default/json/empty/None suffixes; vocab/config first-failure
order; loader call timing and error pass-through; top-level and `dictionaries`
shape failures; null, empty, file, directory, nested, Unicode, relative and
absolute dictionary paths; first missing entry; non-path values; invalid config
content that passes; and repeat/no-cache behavior.

## Writer Gate

The unit is confirmed and fixture-complete. A writer may implement exactly this
injected-outcome validator in the independent Rust workspace. The writer must
not add a YAML crate, parse YAML, expand this seam to full `safe_load`, or route
production callers.

## Rollback

Keep `config_utils.check_configs` and `load_yaml` as runtime owners. There is no
bridge or production import to reverse during implementation.
