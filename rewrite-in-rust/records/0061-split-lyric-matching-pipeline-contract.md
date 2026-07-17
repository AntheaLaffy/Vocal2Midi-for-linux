# 0061 - Split Lyric Matching Pipeline Contract

Date: 2026-07-17

## Context

The next manifest unit after `ja_g2p_fallback_core` was
`lyric_matching_pipeline_contract`. It was still `planned` and `provisional`, so
dependency/bootstrap discovery was required before writer work.

The selected sources were:

- `inference/LyricFA/tools/lyric_matcher.py`
- `inference/LyricFA/tools/language_processors.py`

## Discovery

The original unit mixed several independently verifiable responsibilities:

- language processor selection and clean/split/phonetic flow;
- Chinese and Japanese G2P calls;
- sequence alignment and difference counting;
- lyric file reads and lab file reads;
- lab-to-lyric filename mapping;
- missing lyric tracking and pipeline counters;
- no-match and threshold-exceeded handling;
- JSON result persistence;
- console display text and summary printing.

The dependency expansion found that several dependencies are already verified
or have their own confirmed seams:

- `lyric_sequence_alignment_core` owns `SequenceAligner`,
  `SmartHighlighter`, and `calculate_difference_count`;
- `zh_g2p_dictionary_core` owns Chinese G2P over injected dictionary maps;
- `ja_g2p_fallback_core` owns Japanese fallback G2P with `pyopenjtalk` absent.

Keeping the original unit intact would either duplicate these verified
behaviors or force a broad file-IO/language/G2P/alignment unit that is harder to
review independently.

## Decision

Split `lyric_matching_pipeline_contract` into two units:

1. `lyric_matching_file_contract_core`
   - current unit;
   - covers `LyricMatchingPipeline` file/state behavior, result JSON schema,
     filename mapping, missing lyric de-duplication, empty-ASR skip, no-match
     handling, and diff-threshold routing;
   - uses injected matcher results in fixtures so language processor and
     alignment behavior stay behind their own seams.
2. `lyric_language_processor_contract`
   - later unit;
   - covers `ProcessorFactory`, base clean-text behavior, zh/en/ja
     clean/split/phonetic flow, and Japanese `build_reference_lyric`.

Do not introduce PyO3, subprocess, HTTP, CLI, runtime-router, model-runtime, or
production bridge dependencies in either unit.

## Verification

For `lyric_matching_file_contract_core`, dependency/bootstrap artifacts are:

```text
rewrite-in-rust/dependencies/lyric_matching_file_contract_core.yaml
rewrite-in-rust/bootstrap/lyric_matching_file_contract_core.md
rewrite-in-rust/fixtures/lyric_matching_file_contract_core.jsonl
rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py
```

Current Python parity command:

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py
```

Expected Rust command after writer implementation:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file
```

## Reversal

Rollback remains keeping `LyricMatcher`, `LyricMatchingPipeline`, and
`ProcessorFactory` in Python as runtime owners. No production bridge was
introduced.
