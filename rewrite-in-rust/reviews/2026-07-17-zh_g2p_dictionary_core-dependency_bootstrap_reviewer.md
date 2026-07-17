# zh_g2p_dictionary_core - dependency_bootstrap_reviewer

Date: 2026-07-17
Decision: pass

Unit: `zh_g2p_dictionary_core`
Role: `dependency_bootstrap_reviewer`

## Findings

No findings for the dependency/bootstrap scope.

Evidence:

- `rewrite-in-rust/manifest.yaml:1181` marks exactly this unit as `zh_g2p_dictionary_core`, with status `reimplemented`, inventory status `confirmed`, legacy current owner, and Rust target owner.
- `inference/LyricFA/tools/ZhG2p.py:71` imports only `os`; dictionary loading is local file IO at `inference/LyricFA/tools/ZhG2p.py:80`, and the deterministic conversion path is `convert`, `convert_list`, `split_string`, `split_string_no_regex`, and `tone_to_normal` at `inference/LyricFA/tools/ZhG2p.py:13`, `inference/LyricFA/tools/ZhG2p.py:48`, `inference/LyricFA/tools/ZhG2p.py:132`, and `inference/LyricFA/tools/ZhG2p.py:153`.
- Production callers are accurately identified: `inference/LyricFA/tools/language_processors.py:6` imports `ZhG2p` and module `split_string`; `inference/API/lfa_api.py:1` imports `ZhG2p` and uses the Mandarin singleton at `inference/API/lfa_api.py:181`.
- `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:13` models injected maps as `ZhG2pDictionaries`; `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:28` constructs from those maps; the module does not introduce dictionary file IO, a Python bridge, model runtime integration, or new Chinese G2P crate dependency.
- `pyproject.toml:7`, `requirements.txt:7`, `requirements-linux.txt:3`, `requirements-web.txt:3`, `uv.lock:4292`, `third_party/sources/manifest.json:1`, and `third_party/native_sources/manifest.json:1` support the dependency record's conclusion: project dependencies cover UI, Web, model, numeric, Qwen, Japanese, and vendored source/runtime packages, but no third-party Chinese G2P package is imported by `ZhG2p.py`.

## Boundary Decision

Boundary confirmed. Do not split, merge, defer, or replace this unit for the dependency/bootstrap gate.

The confirmed unit boundary is justified because `ZhG2p.py` separates into one independently verifiable deterministic core plus adjacent runtime/file surfaces:

- In boundary: token splitting, tone normalization, traditional-to-simplified lookup, dictionary-map conversion, phrase-window precedence, ASCII number conversion, include-tone behavior, and unknown-token reset. This matches `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:8` and `rewrite-in-rust/dependencies/zh_g2p_dictionary_core.yaml:3`.
- Kept legacy: dictionary file IO and full bundled dictionary ownership, language processor orchestration, `LyricMatcher` and `lfa_api` orchestration, Japanese G2P, model execution, GUI/Web/CLI routing, and production bridge wiring. This matches `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:22` and `rewrite-in-rust/dependencies/zh_g2p_dictionary_core.yaml:35`.
- The injected-map seam is appropriate while full dictionary loading remains legacy-owned. Python loads `phrases_map`, `phrases_dict`, `user_dict`, `word`, and `trans_word` at `inference/LyricFA/tools/ZhG2p.py:87`; Rust accepts the already loaded map shapes at `rewrite-in-rust/rust/crates/v2m-core/src/zh_g2p.rs:13`.

The hand-written Rust replacement is justified. There is no package-level Chinese G2P dependency to map to a Rust crate; the behavior is local string and dictionary control flow. This follows the rewrite policy at `rewrite-in-rust/README.md:65` and `rewrite-in-rust/README.md:66`, and the dependency evidence in `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:29`.

## Artifact Specificity

The dependency/bootstrap artifacts are specific enough for this role:

- `rewrite-in-rust/dependencies/zh_g2p_dictionary_core.yaml:4` names each covered capability and its legacy/Rust reference.
- `rewrite-in-rust/bootstrap/zh_g2p_dictionary_core.md:52` defines the independent Rust library seam and explicitly excludes bridge and packaging work.
- `rewrite-in-rust/records/0057-confirm-zh-g2p-dictionary-boundary.md:38` records the boundary decision, reversal path, and expected verification commands.
- `rewrite-in-rust/fixtures/zh_g2p_dictionary_core.jsonl:1` provides map-injected fixture cases for the selected behavior, and `rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py:51` verifies the same injected-map seam against legacy Python by constructing `ZhG2p` without calling dictionary file loading.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_zh_g2p_dictionary_core.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml zh_g2p`: pass, 1 `v2m-core` fixture-table test passed and 0 bridge tests selected.
- `git diff --check`: pass.

## Residual Risk

This dependency/bootstrap review does not prove full behavior parity across the bundled Mandarin and Cantonese dictionary files. It proves the boundary and injected-map seam are appropriate and that the fixture harness covers representative map-driven behavior. Behavior and data/algorithm reviewers should still evaluate edge-case parity and phrase-window semantics before coordinator promotion.

Runtime promotion remains unplanned in this unit. A future promotion record must define dictionary asset packaging, payload validation, Python-facing error mapping, logging text, and bridge ownership before production callers import Rust G2P helpers.

## Promotion Note

This role does not block coordinator state update for the dependency/bootstrap gate. The unit is ready for the next required review roles listed in `rewrite-in-rust/manifest.yaml:1191`: stage behavior review and data/algorithm review. The manifest should not be marked promoted by this report alone.
