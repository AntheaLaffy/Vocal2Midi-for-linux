# 0044 - Confirm slice method and custom bounds boundary

Date: 2026-07-17

## Decision

Confirm `slice_method_and_bounds_contract` as a pure contract unit over slicing
method strings and optional custom duration bounds.

The unit covers:

- method normalization in `inference/API/slicer_api.py` and
  `scripts/slice_asr_cli.py`;
- CLI custom bounds in `scripts/slice_asr_cli.py::resolve_slice_bounds`;
- API custom bounds in
  `inference/API/slicer_api.py::_resolve_custom_slice_bounds`.

The unit does not reimplement `application.config.validate_slice_bounds`; that
0-60 second user-facing application validator is already verified by
`slice_bounds_validation`.

## Rationale

The selected behavior is deterministic stdlib-level string and float handling.
Although the owning modules import heavy audio, ASR, RMVPE, and slicer
dependencies, those capabilities are not needed to prove this contract.

Keeping this boundary narrow avoids re-expanding a small compatibility seam into
librosa, NumPy, SoundFile/libsndfile, FFmpeg, ONNX Runtime, Qwen ASR, RMVPE, or
the actual slicing algorithms.

## Verification

Legacy fixture check:

```bash
uv run python rewrite-in-rust/bootstrap/check_slice_method_and_bounds_contract.py
```

Future Rust writer check:

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml slice_method
```

## Rollback

Keep `inference.API.slicer_api`, `scripts.slice_asr_cli`, and
`application.config` as runtime owners until a promotion record introduces Rust
integration.
