# 0126 - Fix ASR Resample Poly Clippy MSRV

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Trigger

Stage-close verification ran `cargo clippy -p v2m-core --all-targets
--all-features -- -D warnings` and found style/MSRV issues in the verified
resample implementation.

## Change

Applied behavior-preserving Rust style fixes:

- removed redundant `f32 -> f32` casts
- replaced `usize::is_multiple_of`, which is newer than the workspace MSRV, with
  a modulo check
- rewrote accumulation as `+=`
- removed explicit loop-counter/range-index patterns in `upfirdn` accumulation

No public seam, fixture, dependency, or runtime owner changed.

## Verification

```bash
cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml asr_resample_poly_contract
cargo clippy --manifest-path rewrite-in-rust/rust/Cargo.toml -p v2m-core --all-targets --all-features -- -D warnings
```

## Rollback

Keep `scipy.signal.resample_poly` calls in Python-owned ASR helpers as runtime
owners.
