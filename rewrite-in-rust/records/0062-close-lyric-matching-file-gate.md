# 0062 - Close Lyric Matching File Gate

Date: 2026-07-17

## Context

`lyric_matching_file_contract_core` was split from the broader
`lyric_matching_pipeline_contract` in record 0061. The accepted unit boundary is
the deterministic file, state, and JSON behavior in
`inference/LyricFA/tools/lyric_matcher.py`, with language processor, G2P,
sequence alignment internals, display text, directory glob ownership, model
execution, GUI/Web/CLI routing, and production bridge wiring kept legacy-owned.

The unit now has current rerun review evidence:

- dependency/bootstrap review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_matching_file_contract_core-dependency_bootstrap_reviewer-rerun2.md`
- behavior review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_matching_file_contract_core-behavior_reviewer-rerun.md`
- error/tracing review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_matching_file_contract_core-error_tracing_reviewer-rerun.md`
- product/ergonomics review:
  `rewrite-in-rust/reviews/2026-07-17-lyric_matching_file_contract_core-product_ergonomics_reviewer-rerun.md`

The earlier signed-threshold finding is closed. Rust now stores the diff
threshold as `i64`, and the shared fixture table includes negative-threshold
parity.

## Decision

Accept `lyric_matching_file_contract_core` as verified for the current
legacy-owned, no-bridge Rust library seam.

The verified Rust unit preserves:

- filename extraction and lab-to-lyric stem mapping;
- missing lyric de-duplication;
- successful lab processing through injected matcher output;
- empty-ASR skip behavior;
- no-match empty JSON output and counters;
- zh phonetic diff-threshold routing;
- non-zh text diff-threshold routing;
- signed negative threshold behavior;
- three-field JSON result schema;
- single-file execute state with caller-supplied path lists.

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_lyric_matching_file_contract_core.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml lyric_matching_file
cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check
git diff --check
uv run python scripts/audit_vendored_sources.py
```

Reviewers also audited that no production Python GUI/Web/CLI caller imports the
Rust lyric matching file-contract seam.

## Residual Risk

The open findings are promotion-gate follow-ups only:

- add or assign parity coverage for lab read failure, lyric load failure, and
  JSON write failure counter/error ordering;
- define structured skip diagnostics if Rust becomes Python-facing;
- define Python-facing JSON write error mapping with path context;
- define exact console/Web log text parity or an accepted text-change policy;
- either keep directory glob/path discovery in Python or add promotion fixtures
  for extension filtering, glob ordering, duplicate stems, path encoding, and
  downstream log/export naming.

These do not block verification of the no-bridge library seam.

## Reversal

Rollback remains keeping `LyricMatcher`, `LyricMatchingPipeline`, and
`ProcessorFactory` in Python as runtime owners. No production bridge was
introduced.
