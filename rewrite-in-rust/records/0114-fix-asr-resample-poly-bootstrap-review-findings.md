# 0114 - Fix ASR Resample Poly Bootstrap Review Findings

Date: 2026-07-18

## Unit

`asr_resample_poly_contract`

## Trigger

The first `dependency_bootstrap_reviewer` report found three blockers:

- short fixtures did not exercise SciPy's steady-state polyphase behavior
- the checker bypassed SciPy validation for equal source/target rates
- the dependency source path omitted `get_window -> kaiser -> special.i0`

## Fixes

Expanded `fixtures/asr_resample_poly_contract.jsonl` from 12 to 18 cases.
Added long steady-state dual-sine inputs for:

- 44100 -> 16000 with 9000 input samples
- 48000 -> 16000 with 256 input samples
- 22050 -> 16000 with 5000 input samples
- 8000 -> 16000 with 256 input samples

Each long case asserts output shape, selected head/mid/tail samples,
`finite_sum`, and `finite_abs_sum` instead of dumping the full output array.

Updated `bootstrap/check_asr_resample_poly_contract.py` so every contract case
calls SciPy `resample_poly`; identity is no longer a checker shortcut. Added
equal invalid-rate cases for `(0, 0)` and `(-16000, -16000)` to pin this
validation order.

Updated dependency and bootstrap records to include:

- `scipy/signal/windows/_windows.py::get_window`
- `scipy/signal/windows/_windows.py::kaiser`
- the `kaiser -> scipy.special.i0` numeric dependency for default window weights

## Verification

```bash
uv run python rewrite-in-rust/bootstrap/check_asr_resample_poly_contract.py
```
