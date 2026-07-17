# lyric_matching_file_contract_core Bootstrap

## Boundary

`lyric_matching_file_contract_core` covers the deterministic file/state/JSON
behavior in `inference/LyricFA/tools/lyric_matcher.py`.

The unit covers:

- `ProcessResult` shape;
- `LyricMatchingPipeline._extract_filename_without_extension`;
- lab-to-lyric name mapping through `lab_name.rsplit("_", 1)[0]`;
- missing lyric de-duplication;
- lab file read-and-strip behavior after lyric lookup succeeds;
- empty ASR phonetic skip behavior;
- no-match handling;
- zh-vs-non-zh diff-threshold source/target sequence choice;
- success, diff, no-match, missing-lyric, and total-file counters;
- result JSON schema from `LyricMatcher.save_to_json`;
- single-file execute behavior with injected matcher outputs.

The unit explicitly does not cover:

- `ProcessorFactory` and language processor selection;
- Chinese/Japanese G2P behavior;
- `pyopenjtalk.run_frontend`;
- sequence alignment algorithm internals;
- `SmartHighlighter` display rendering;
- exact console output text;
- full directory glob ordering across multiple files;
- model execution;
- GUI/Web/CLI routing;
- production Rust routing.

## Dependency Expansion

`lyric_matcher.py` imports only standard-library `glob`, `json`, `os`,
`dataclasses`, and `typing`, plus local helpers:

- `ProcessorFactory` and `LyricData` from `language_processors.py`;
- `SequenceAligner`, `calculate_difference_count`, and `SmartHighlighter` from
  `sequence_aligner.py`.

Dependency evidence:

- `lyric_sequence_alignment_core` already verifies `SequenceAligner`,
  `SmartHighlighter`, and `calculate_difference_count`.
- `zh_g2p_dictionary_core` and `ja_g2p_fallback_core` already verify the
  deterministic G2P helper seams needed by language processors.
- `pyproject.toml`, `requirements*.txt`, and `uv.lock` include heavy model/UI
  packages, but this file-contract slice does not call model runtimes or need a
  third-party package replacement.
- `third_party/source_audit.json` reports source coverage without binary
  artifacts for the installed dependency set.

The selected Rust seam therefore uses injected matcher outputs in fixtures. It
tests `LyricMatchingPipeline` file/state behavior without reimplementing
language processor or alignment internals in this unit.

## Seam

Use the default rewrite seam:

- kind: independent Rust library
- crate: `rewrite-in-rust/rust/crates/v2m-core`
- module: `lyric_matching_file`
- runtime owner: legacy Python
- bridge dependencies: none

No production caller imports Rust lyric matching helpers in this unit.

## Fixture Harness

Python/Rust parity uses:

```text
rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl
```

The legacy Python side is checked by:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py
```

The fixture table uses a fake matcher to inject:

- lyric file processing outputs;
- ASR text/phonetic outputs;
- alignment outputs;
- JSON save behavior through the real `LyricMatcher.save_to_json` static
  method.

The fixture table covers:

- filename extraction and lab-to-lyric mapping;
- missing lyric de-duplication;
- successful lab processing;
- empty-ASR skip;
- no-match JSON output and counters;
- zh phonetic diff-threshold routing;
- non-zh text diff-threshold routing;
- result JSON schema;
- single-file execute state and JSON output.

The Rust side should be checked by:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file
```

## Repeated-Call Behavior

The selected state transitions are deterministic for fixed file contents,
fixture-injected matcher outputs, language, threshold, and output directory.
The Rust seam does not depend on model state, network state, GUI/Web state, or
global caches.

## Rollback

Rollback is keeping production imports unchanged:

```text
inference.LyricFA.tools.lyric_matcher.LyricMatcher
inference.LyricFA.tools.lyric_matcher.LyricMatchingPipeline
```

No production caller should import Rust lyric matching file-contract helpers
until a promotion record defines processor payloads, logging text, Python-facing
error mapping, directory glob ordering, and rollback.
