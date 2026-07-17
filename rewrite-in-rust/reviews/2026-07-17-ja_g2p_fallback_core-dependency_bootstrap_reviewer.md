# ja_g2p_fallback_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass

## Findings

No findings.

Boundary decision: confirmed. The dependency/bootstrap evidence justifies keeping `ja_g2p_fallback_core` as one narrow manifest unit rather than splitting, merging, deferring, or replacing it.

Evidence:

- `rewrite-in-rust/manifest.yaml:1212` defines the unit as `ja_g2p_fallback_core`, status `reimplemented`, inventory status `confirmed`, with legacy runtime ownership and Rust target ownership.
- `rewrite-in-rust/manifest.yaml:1220` scopes the public interface to character classification, kana/romaji tables, number mapping, token splitting, kana mora splitting, long-vowel handling, fallback entry behavior, the `pyopenjtalk`-absent path, `convert_number`, and kana-vs-romaji output.
- `inference/LyricFA/tools/JaG2p.py:3` imports optional `pyopenjtalk`, and `inference/LyricFA/tools/JaG2p.py:5` sets `pyopenjtalk = None` on `ImportError`, so fallback behavior is an existing Python public path.
- `pyproject.toml:21`, `requirements.txt:20`, and `uv.lock:4304` declare `pyopenjtalk` behind a Windows marker; `uv run python -c "import importlib.util; print(importlib.util.find_spec('pyopenjtalk'))"` returned `None` in this Linux workspace.
- `rewrite-in-rust/dependencies/ja_g2p_fallback_core.yaml:23` records inventory impact as `confirmed`, with the reason tied to Windows-markered `pyopenjtalk` and Linux fallback behavior.
- `rewrite-in-rust/dependencies/ja_g2p_fallback_core.yaml:36` keeps `pyopenjtalk` frontend analysis legacy-owned, and `rewrite-in-rust/dependencies/ja_g2p_fallback_core.yaml:39` keeps language processors, LyricMatcher, `lfa_api` orchestration, ASR phoneme conversion, model execution, GUI/Web/CLI routing, and production bridge wiring legacy-owned.
- `rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md:26` excludes `pyopenjtalk.run_frontend`, OpenJTalk dictionary/model ownership, language processor orchestration, LyricMatcher, `lfa_api` orchestration, persistence, ASR post-processing, model execution, GUI/Web/CLI routing, and production Rust routing.
- `rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md:56` justifies a hand-written fallback over local `JaG2p.py` tables and explicitly avoids OpenJTalk, PyO3, subprocess, HTTP, CLI, and runtime-router dependencies.
- `rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py:21` forces `ja_module.pyopenjtalk = None`, so fixture generation/checking is specific to the intended fallback path.
- `rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl:1` through `rewrite-in-rust/fixtures/ja_g2p_fallback_core.jsonl:19` cover classification, normalization, token splitting, Unicode numeric behavior, kana splitting, long vowels, conversion, romaji output, kana output, and `convert_list`.
- `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:1` documents the Rust module as Japanese fallback G2P compatibility helpers; `rewrite-in-rust/rust/crates/v2m-core/src/ja_g2p.rs:4` states it mirrors `JaG2p.py` with `pyopenjtalk` unavailable.
- `rewrite-in-rust/rust/crates/v2m-core/src/lib.rs:17` exposes `ja_g2p` as an independent Rust module, and `rewrite-in-rust/bootstrap/ja_g2p_fallback_core.md:70` states no production caller imports Rust Japanese G2P helpers in this unit.
- `rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:38` confirms the boundary as the `pyopenjtalk`-absent fallback path, while `rewrite-in-rust/records/0059-confirm-ja-g2p-fallback-boundary.md:41` reserves frontend-backed Japanese G2P for a later package/runtime ownership decision.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_ja_g2p_fallback_core.py`: passed.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml ja_g2p`: passed; 1 `ja_g2p` test passed, 0 failed.
- `git diff --check`: passed.
- `uv run python -c "import importlib.util; print(importlib.util.find_spec('pyopenjtalk'))"`: returned `None`, confirming the optional frontend package is absent in this Linux uv environment.
- `rg -n '"name": "pyopenjtalk"|pyopenjtalk' third_party/sources/manifest.json third_party/source_audit.json third_party/native_sources/manifest.json third_party/sources/MISSING_SOURCES.md`: no matches, consistent with source vendoring for the installed Linux package set rather than the Windows-only optional dependency.

## Residual Risk

This review does not prove `pyopenjtalk.run_frontend` parity, Windows behavior, OpenJTalk dictionary/model ownership, or Python-facing bridge payload/error/log behavior. Those are correctly out of scope for this no-bridge Linux fallback unit and must be reviewed if a later promotion introduces frontend-backed Japanese G2P or production Rust routing.

## Promotion Note

This dependency/bootstrap role does not block moving the unit to the next required review gate. Do not mark the manifest unit verified from this report alone; the manifest still requires the behavior and data/algorithm review evidence listed for this unit.
