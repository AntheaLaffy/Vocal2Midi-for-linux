# quantization_pipeline_promotion Bootstrap

## Boundary

`quantization_pipeline_promotion` is confirmed as the final quantization caller
routing and runtime-ownership unit.

It covers:

- the production quantization seam in
  `inference/pipeline/auto_lyric_hybrid.py` after GAME note extraction and
  before all exports;
- wrapper selection through `inference/quant/rust_bridge.py`;
- bridge executable configuration and packaging expectations;
- rollback to legacy Python quantization;
- cancellation and timeout behavior around the bridge subprocess;
- preservation, or intentional prior update, of the verified GUI/Web
  quantization setting contract.

It does not cover:

- quantization algorithm parity, already covered by the verified quantization
  algorithm units;
- the JSON bridge proof, already covered by verified
  `quantization_bridge_bootstrap`;
- caller/default discovery, already covered by verified
  `quantization_caller_defaults_contract`;
- GUI rewrite, Flask/Web rewrite, model inference, slicing, ASR, forced
  alignment, GAME, RMVPE, or export format internals.

## Current Source Facts

The live pipeline currently sorts notes, writes the optional ASR match log, then
uses the legacy activation gate and legacy quantizer:

```text
all_notes.sort(key=lambda x: x.onset)
if should_apply_quantization(quantization_mode, quantization_step):
    quantize_notes(all_notes, tempo, quantization_step, mode=quantization_mode)
```

The same mutated `all_notes` list is then exported to MIDI, TXT, CSV, and USTX.

`inference/quant/rust_bridge.py` already provides the accepted bridge wrapper:

- `backend="legacy"` calls the legacy Python quantizer and is the current
  default;
- `backend="rust-json"` sends one JSON payload to `v2m-quant-bridge` and applies
  timing mutations to the original Python note objects;
- `V2M_QUANT_BACKEND` can select the backend;
- `V2M_QUANT_BRIDGE_BIN` or an explicit executable path is required for
  `rust-json`;
- current wrapper timeout is `30.0` seconds, but it does not yet accept
  `cancel_checker`.

`PipelineConfig` defaults remain `quantization_step=16` and
`quantization_mode="bayes"`. Desktop AutoLyric still passes `0/simple` from
disabled controls. Web still persists disabled `none/dev` values but does not
map those fields into `PipelineConfig`, so effective Web runtime remains
`bayes/16`.

## Seam Decision

Use the existing CLI/subprocess JSON bridge for promotion. Do not switch this
unit to PyO3/maturin, ctypes/cdylib, an HTTP service, or a Python runtime router
without a new record.

The future writer should route production quantization through the wrapper, not
duplicate bridge logic in the pipeline:

```python
if should_apply_quantization(quantization_mode, quantization_step):
    quantize_notes_with_backend(
        all_notes,
        tempo,
        quantization_step,
        mode=quantization_mode,
        backend=quantization_backend,
        executable=quantization_bridge_bin or None,
        timeout_sec=quantization_timeout_sec,
        cancel_checker=cancel_checker,
    )
```

The exact names may follow local style, but the production seam must expose
these concepts:

- backend selection with a legacy rollback value;
- bridge executable path;
- bridge timeout;
- cancel checker propagation.

During the reimplemented-but-unreviewed state, keep the effective default owner
as legacy unless the bridge binary is explicitly configured. Moving the default
runtime owner to Rust is the promotion step and requires the review gates below.

## Bridge Executable And Packaging

`v2m-quant-bridge` must be built outside application runtime:

```bash
cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge
```

The application must not run `cargo build` dynamically. The bridge executable
path must come from explicit configuration or `V2M_QUANT_BRIDGE_BIN`.

Promotion tests must cover:

- configured existing executable;
- missing executable;
- directory or non-executable path;
- child process nonzero exit;
- non-JSON stdout;
- timeout.

Rollback is selecting `legacy` backend. Do not silently retry legacy after a
Rust bridge failure unless an explicit fallback option is added and tested; a
silent retry would hide bridge errors and weaken parity evidence.

## Cancellation And Performance

Production promotion needs a cancel-aware bridge call. The current wrapper's
`subprocess.run(..., timeout=...)` is enough for bootstrap proof, but not enough
for Web/GUI cancellation because it cannot observe `cancel_checker` until the
process exits or times out.

Before the pipeline routes production calls to `rust-json`, the writer must:

- check cancellation before starting quantization;
- start the bridge in a form that can be polled;
- kill the child process when `cancel_checker` becomes true;
- map cancellation back to the existing pipeline cancellation path;
- check cancellation again after quantization and before export;
- keep a bounded timeout;
- keep one bridge process per quantization operation, never one per note;
- keep stdout reserved for the JSON response.

Performance evidence should use deterministic generated note sets for simple,
smart, bayes, and dp modes. The first hard limits are architectural:

- one bridge process per quantization operation;
- no runtime build step;
- no unbounded subprocess wait;
- cancellation can terminate a running child process promptly.

Do not promote on microbenchmarks alone; behavior and rollback evidence remain
required.

## GUI And Web Follow-Ups

The verified caller/default contract intentionally preserves current mismatch:

- Desktop AutoLyric controls are disabled and pass `quantization_step=0`,
  `quantization_mode="simple"`.
- `gui/fluent_utils.py` parser mappings exist but are not the active AutoLyric
  config path.
- Web controls are disabled and submit/persist `quantize_precision="none"` and
  `quantize_algorithm="dev"`.
- `web_task_manager.py` ignores those Web fields and effective
  `PipelineConfig` defaults remain `bayes/16`.

Promotion must preserve those semantics unless a separate product change updates
`quantization_caller_defaults_contract`, its tests, and product ergonomics
review first.

## Implemented Writer Artifacts

The first writer pass added wrapper-based production routing while keeping the
effective default rollbackable to legacy:

- `application/config.py` now carries optional `quantization_backend`,
  `quantization_bridge_bin`, and `quantization_timeout_sec` fields through
  `PipelineConfig.to_kwargs`.
- `inference/pipeline/auto_lyric_hybrid.py` routes the existing post-GAME,
  pre-export quantization seam through `quantize_notes_with_backend`; an empty
  backend defers to wrapper/environment selection, while explicit `legacy`
  remains rollback.
- `inference/quant/rust_bridge.py` now supports `cancel_checker`, bounded
  timeout validation, and killing an in-flight bridge process. Timeout and
  cancellation cover the subprocess communication boundary, including stdin
  payload delivery to a child process that starts but does not read stdin.
- `tests/test_quantization_pipeline_promotion.py` covers the promotion seam.

Covered cases:

- legacy default keeps existing fake-note pipeline behavior;
- explicit `rust-json` backend mutates/reorders the same original note objects
  before every export;
- explicit legacy backend rolls back even when Rust bridge configuration exists;
- unsupported backend surfaces a boundary error;
- disabled non-dp quantization does not call the wrapper;
- `dp` with step zero still routes through quantization;
- unknown positive modes still use simple fallback through the bridge;
- missing/non-executable bridge paths are errors, not silent behavior changes;
- timeout is bounded;
- an in-flight fake bridge is killed when `cancel_checker` flips;
- non-reading bridge executables cannot block indefinitely while receiving a
  JSON payload;
- Web quantization fields remain ignored unless the caller/default contract is
  intentionally changed.

Existing required checks remain:

```bash
uv run pytest tests/test_quantization_pipeline_promotion.py
cargo build --manifest-path rewrite-in-rust/rust/Cargo.toml --bin v2m-quant-bridge
uv run python rewrite-in-rust/bootstrap/check_quantization_bridge_bootstrap.py
uv run pytest tests/test_quantization_caller_defaults_contract.py
uv run pytest tests/test_web_api.py
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml
uv run python scripts/audit_vendored_sources.py
```

## Review Gates

Do not mark `quantization_pipeline_promotion` verified or promoted without:

- stage behavior review covering pipeline routing/export parity and fallback;
- architecture review covering owner boundary, bridge executable packaging,
  backend selection, and rollback;
- product ergonomics review covering GUI/Web setting semantics and user-visible
  failure/cancellation behavior.

## Rollback

Rollback is selecting the legacy backend and keeping
`inference.quant.quantization` available. A rollback must not require changing
GUI/Web settings, model inference paths, or export code.
