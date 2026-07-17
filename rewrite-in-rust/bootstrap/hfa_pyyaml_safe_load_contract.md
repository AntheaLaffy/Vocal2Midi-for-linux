# hfa_pyyaml_safe_load_contract Bootstrap

## Boundary

Preserve `config_utils.load_yaml`: open one path as UTF-8 text and call PyYAML
6.0.3 `safe_load`, including the returned Python value type and exact failure
class/message. This is the full loader phase split from deterministic config
validation; it remains planned and Python-owned.

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

No dependency is added during this bootstrap pass.

## Required Next Bootstrap

Before writer work:

1. define a tagged canonical value projection that does not erase Python
   date/datetime/bytes/set/omap/pairs distinctions or mapping order/keys;
2. create a Python 3.12/PyYAML 6.0.3 real-loader golden checker for the complete
   resolver/constructor/error matrix in the dependency record;
3. select and pin either a partial parser/event crate plus an explicit
   compatibility layer, or a fixture-bound hand-written replacement justified
   against vendored PyYAML resolver/constructor/composer sources;
4. prove single-document and safe-tag rejection, source marks, UTF-8/file errors,
   repeated loads, and resource limits;
5. update this unit from provisional only after that evidence closes the writer
   gate.

## Kept Legacy And Rollback

PyYAML remains the sole loader/runtime owner. `hfa_config_validation_core` does
not parse YAML and cannot be substituted for this unit. No production route or
bridge changes during bootstrap.
