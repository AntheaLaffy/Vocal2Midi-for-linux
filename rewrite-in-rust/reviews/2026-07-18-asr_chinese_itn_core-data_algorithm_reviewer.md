# asr_chinese_itn_core - data_algorithm_reviewer

Date: 2026-07-18
Decision: fail

## Findings

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:593
- Issue: The Rust pure-number predicate accepts malformed decimal spans that Python rejects and leaves unchanged. `is_pure_num` treats `点` as always valid and does not require a digit after it, so `一二点` is accepted; `convert_pure_num` then maps it to `12.`. Python's `pure_num` regex requires `点` to be followed by one or more Chinese digits, and the fallback replacement path returns the original span when no converter matches.
- Evidence: `../inference/qwen3asr_dml/chinese_itn.py:228`, `../inference/qwen3asr_dml/chinese_itn.py:447`, and `../inference/qwen3asr_dml/chinese_itn.py:482` define the stricter regex/fallback behavior. `uv run python - <<'PY' ... PY` from `/home/fuurin/code/Vocal2Midi-for-linux` returned `一点 => 一点`, `一二点 => 一二点`, and `点三 => 点三`. The fixture file has one valid decimal case at `fixtures/asr_chinese_itn_core.jsonl:3`, but no trailing-dot or malformed decimal no-op cases.
- Required fix: Tighten `is_pure_num` to mirror the Python decimal grammar, then add golden fixtures for rejected decimal-like spans such as `一点`, `一二点`, and `点三`.

- Severity: high
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:117
- Issue: The Rust span scanner retries shorter suffixes after a longer candidate fails, while Python `re.sub` consumes the matched span once and returns it unchanged when conversion fails. This can convert valid prefixes inside spans that Python intentionally leaves unchanged.
- Evidence: Rust loops from the longest candidate down to shorter candidates at `rust/crates/v2m-core/src/asr_chinese_itn.rs:117`, and accepts a shorter conversion when `converted != candidate` at `rust/crates/v2m-core/src/asr_chinese_itn.rs:125`. Python calls `pattern.sub(replace, original)` once per regex match at `../inference/qwen3asr_dml/chinese_itn.py:508`; unsupported replacements fall through to `final = original` at `../inference/qwen3asr_dml/chinese_itn.py:482` or exception fallback at `../inference/qwen3asr_dml/chinese_itn.py:494`. The Python reference check returned `十点二十 => 十点二十`, `二点十 => 二点十`, and `百分之十点二十 => 百分之十点二十`; those rejected longer spans are not represented in `fixtures/asr_chinese_itn_core.jsonl:1`.
- Required fix: Make the Rust scanner preserve the Python one-shot match/no-op behavior for rejected longer spans, or add documented fixture evidence proving a narrower accepted contract. Add fixtures for invalid time/percent/decimal spans with valid numeric prefixes.

- Severity: medium
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:145
- Issue: Idiom overlap uses a byte-window approximation, `start.saturating_sub(6)`, for Python's `l_pos - 2` character-index check. That assumes the two-character lookback is six UTF-8 bytes and can diverge for ASCII heads, spaces, or non-BMP Unicode adjacent to candidate spans.
- Evidence: Python computes the idiom lookback in string indices at `../inference/qwen3asr_dml/chinese_itn.py:417` and `../inference/qwen3asr_dml/chinese_itn.py:428`; Rust compares byte offsets at `rust/crates/v2m-core/src/asr_chinese_itn.rs:335`. Fixtures cover standalone idiom/no-op cases at `fixtures/asr_chinese_itn_core.jsonl:20`, `fixtures/asr_chinese_itn_core.jsonl:31`, and `fixtures/asr_chinese_itn_core.jsonl:32`, but not idiom adjacency with ASCII heads or mixed-width Unicode.
- Required fix: Convert the lookback to character-index accounting, or add fixtures that lock the intended byte-level approximation.

- Severity: low
- Location: rust/crates/v2m-core/src/asr_chinese_itn.rs:117
- Issue: The handwritten scanner has avoidable nested work: for each candidate start it tries many suffixes, and each replacement can sort unit lists and scan every idiom across the full context. That is acceptable for short ASR lines, but the complexity is undocumented and the repeated allocation/sorting makes the algorithm harder to maintain.
- Evidence: Suffix retry is at `rust/crates/v2m-core/src/asr_chinese_itn.rs:117`, idiom scanning is at `rust/crates/v2m-core/src/asr_chinese_itn.rs:335`, and repeated unit sorting occurs at `rust/crates/v2m-core/src/asr_chinese_itn.rs:438` and `rust/crates/v2m-core/src/asr_chinese_itn.rs:856`.
- Required fix: Document the expected input-size bound, and consider static pre-sorted unit tables plus tests for long malformed numeric spans if this helper becomes production-routed.

## Checks

- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_chinese_itn_core`: passed, 1 targeted fixture test passed.
- `uv run python rewrite-in-rust/bootstrap/check_asr_chinese_itn_core.py`: passed, 37 Python golden cases still match the current Python implementation.
- `cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml --all-targets --all-features -- -D warnings`: passed.
- `uv run python - <<'PY' ... PY`: sampled Python rejected-span behavior; `十点二十`, `二点十`, `百分之十点二十`, `一点`, `一二点`, and `点三` remain unchanged in Python.

## Residual Risk

The unit boundary is confirmed: `manifest.yaml:1725` limits this child to Chinese ITN, and the umbrella split is recorded in `dependencies/asr_text_postprocess_contract.yaml:112` and `records/0102-bootstrap-asr-text-postprocess-contract.md:18`. Writer/reviewer separation is preserved by this read-only review. The remaining risk is not unrelated ASR behavior; it is the unverified regex-rejection surface inside this unit. The current 37 fixtures cover successful conversion branches well, but they do not cover enough converter-failure/no-op branches to validate the handwritten scanner approximation.

## Promotion Note

This role blocks promotion. The unit should not be marked verified until malformed decimal/time/percent spans and regex no-op behavior are fixture-backed and the Rust scanner/predicates are adjusted or explicitly re-scoped.
