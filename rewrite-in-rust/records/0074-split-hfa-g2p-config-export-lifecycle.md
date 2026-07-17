# 0074 - Split HFA G2P, Config, And Export Lifecycle

Date: 2026-07-17

## Context

The provisional `hfa_g2p_export_config_core` grouped three source modules and
unrelated lifecycle phases: upstream text-to-phoneme conversion, model-folder
validation, and downstream effectful label export. Dependency expansion found
separate standard-library, PyYAML, and textgrid 1.6.1 boundaries plus independent
caller/rollback points. Keeping the mixed unit would hide state and package
contracts rather than reduce implementation complexity.

## Decision

Replace it, in implementation order, with:

1. `hfa_phoneme_mora_g2p_core` (confirmed): pure Base/phoneme/Japanese-mora
   G2P and the shared output/prefix contract. This is the first writer route.
2. `hfa_dictionary_g2p_core` (confirmed): dictionary file parsing, lookup, edge
   SP/missing-word warnings, and the same output contract.
3. `hfa_config_file_contract_core` (provisional): PyYAML safe-load and file/path
   validation; it stays provisional until a maintained Rust YAML parser is
   selected and compatibility limits are recorded.
4. `hfa_htk_label_export_core` (confirmed): in-memory HTK planned files,
   paths, scaling, and cumulative prediction state.
5. `hfa_textgrid_export_core` (confirmed): the narrow textgrid 1.6.1 long-format
   serialization/path subset.
6. `hfa_export_dispatch_contract` (confirmed): format membership, fixed call
   order, caller default/status behavior, and composition of the two exporters.

The first two share a G2P output type and therefore stay ordered, but file and
warning ownership remains separate. HTK and TextGrid share verified HFA Word
inputs, not algorithms. Dispatch follows both. Config is independent of those
data paths and remains legacy-owned while its parser dependency is unresolved.

## Discovery Evidence

- `g2p.py` otherwise uses only pathlib/warnings. Python 3.12 probes confirmed
  empty and literal-space tokens, control tokens, mora fallbacks, and exact index
  shapes; these need fixtures rather than normalization.
- `check_configs` parses vocab through PyYAML but merely checks config-file
  existence. Missing `dictionaries` defaults to a list and then raises
  `AttributeError`; invalid config contents can pass. These are compatibility
  facts, not a reason to hand-roll YAML.
- `save_htk` initializes `w_out`/`ph_out` before its prediction loop. A two-file
  probe confirmed the second file contains first plus second prediction labels.
  Treat this likely bug as current contract until a product decision changes it.
- TextGrid output is owned by pinned `textgrid==1.6.1`; its writer fills blank
  gaps, doubles quotes, uses UTF-8/Python float text, and rejects invalid or
  overlapping intervals. Vendored source is the hand-written reference.
- Export dispatch is membership-based, case-sensitive, and always invokes
  TextGrid before HTK regardless of requested order.

## Kept Legacy And Reversal

Keep wav/lab discovery, dataset mutation, aggregation, ONNX/model execution,
PyYAML parsing until aligned, directory creation/writes, status/warning
presentation, API artifact copying, CLI/GUI/Web routing, and all production
imports in Python. No bridge or Rust production code is introduced. Reversal is
restoring the single provisional manifest entry; runtime behavior is unchanged.
