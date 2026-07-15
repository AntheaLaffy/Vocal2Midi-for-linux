# quantization_caller_defaults_contract - behavior_reviewer

Date: 2026-07-15
Decision: pass

## Findings

No findings.

The behavior surface matches the unit contract: the manifest keeps the unit at
`reimplemented` with legacy as both current and target owner
(`rewrite-in-rust/manifest.yaml:305`, `rewrite-in-rust/manifest.yaml:307`,
`rewrite-in-rust/manifest.yaml:309`, `rewrite-in-rust/manifest.yaml:310`);
the contract fixture covers null, empty, simple, smart, bayes, dp, unknown,
uppercase, padded, positive, zero, and negative-step cases
(`rewrite-in-rust/fixtures/quantization_caller_defaults_contract.tsv:1`);
and the focused tests cover public dispatch, pipeline gating/export mutation,
configuration defaults/pass-through, GUI disabled/parser behavior, and Web
ignored quantization settings
(`tests/test_quantization_caller_defaults_contract.py:63`,
`tests/test_quantization_caller_defaults_contract.py:71`,
`tests/test_quantization_caller_defaults_contract.py:93`,
`tests/test_quantization_caller_defaults_contract.py:150`,
`tests/test_quantization_caller_defaults_contract.py:211`,
`tests/test_quantization_caller_defaults_contract.py:272`,
`tests/test_quantization_caller_defaults_contract.py:289`,
`tests/test_quantization_caller_defaults_contract.py:305`,
`tests/test_quantization_caller_defaults_contract.py:345`).

Source inspection confirms the tested behavior is the current legacy behavior:
`quantize_notes` and `should_apply_quantization` lowercase without trimming,
route `smart`/`bayes`/`dp`, fall back to simple, and make exact `dp` active even
with step zero (`inference/quant/quantization.py:793`,
`inference/quant/quantization.py:806`); the pipeline sorts, gates, mutates, then
exports the same note list (`inference/pipeline/auto_lyric_hybrid.py:435`,
`inference/pipeline/auto_lyric_hybrid.py:443`,
`inference/pipeline/auto_lyric_hybrid.py:449`); `PipelineConfig` defaults and
`to_kwargs` pass through bayes/16 (`application/config.py:60`,
`application/config.py:61`, `application/config.py:95`,
`application/config.py:96`); AutoLyric disables quantization controls and passes
simple/0 (`gui/auto_lyric_view.py:186`, `gui/auto_lyric_view.py:189`,
`gui/auto_lyric_view.py:194`, `gui/auto_lyric_view.py:197`,
`gui/auto_lyric_view.py:394`, `gui/auto_lyric_view.py:395`); parser mappings
match the test expectations (`gui/fluent_utils.py:1`,
`gui/fluent_utils.py:15`); and Web settings remain visible while
`TaskManager._build_config` omits quantization overrides so the dataclass
defaults apply (`web_server.py:94`, `web_server.py:101`,
`web_server.py:102`, `web_task_manager.py:399`,
`web_task_manager.py:429`).

Reviewer separation was preserved. I reviewed only and wrote this report; I did
not edit production code, tests, manifest, bootstrap, dependency records, or
Rust code.

## Checks

- `uv run pytest tests/test_quantization_caller_defaults_contract.py -q`: passed, 37 tests.
- `uv run pytest tests/test_web_api.py -q`: passed, 53 tests.
- `git diff --name-only -- inference/quant/quantization.py inference/pipeline/auto_lyric_hybrid.py application/config.py gui/auto_lyric_view.py gui/fluent_utils.py web_server.py web_task_manager.py`: no output; the source refs reviewed here have no unstaged diff.
- `rg -n "rust|bridge|quantize_notes_rust|rust-json|v2m-quant-bridge" inference/quant/quantization.py inference/pipeline/auto_lyric_hybrid.py application/config.py gui/auto_lyric_view.py gui/fluent_utils.py web_server.py web_task_manager.py`: no matches; no production caller in this behavior surface routes to Rust.
- `git diff --check`: passed.

## Residual Risk

This behavior review does not cover product ergonomics, live GUI interaction,
full model inference, algorithm internals, bridge error handling, or the later
runtime promotion route. The GUI disabled-control assertion is AST/source based,
not a live Qt interaction test.

## Promotion Note

This behavior role does not block promotion. Do not mark the manifest verified
from this report alone; `product_ergonomics_reviewer` is still required for
`quantization_caller_defaults_contract`.
