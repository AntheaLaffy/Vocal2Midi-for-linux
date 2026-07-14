# Vocal2Midi Rust Rewrite Skills

This directory is the source-of-truth copy of the project-specific rewrite
skills. Mirror these directories to `/home/fuurin/.claude/skills/` when the user
wants the skills available in future sessions.

## Skills

- `vocal2midi-rs-rewrite`: coordinate the rewrite loop, choose or re-cut the
  next unit, and route dependency discovery before implementation.
- `vocal2midi-rs-dep-bootstrap`: expand Python dependencies, align capability
  coverage, choose narrow Rust replacements, and update seam/bootstrap records.
- `vocal2midi-rs-unit-writer`: implement exactly one confirmed migration unit.
- `vocal2midi-rs-review-gate`: review exactly one unit and one review role.

## Project Difference

These skills are inspired by mvsep-rs, but Vocal2Midi is harder in a different
way: Python dependency expansion, native/FFI sources, and dependency mismatches
can change the rewrite boundary. The manifest unit list is provisional. Re-cut
planned units when discovery shows a better independently verifiable shape.

## Install Mirror

The repository copy is authoritative. The user skill root is a deployment
mirror. Do not edit the mirror directly unless the same change is also made
here.

Use these example prompts after mirroring:

```text
Use $vocal2midi-rs-rewrite to continue the next migration unit.
Use $vocal2midi-rs-rewrite to re-cut the provisional unit inventory.
Use $vocal2midi-rs-dep-bootstrap for runtime_device_normalization.
Use $vocal2midi-rs-unit-writer to implement slice_bounds_validation.
Use $vocal2midi-rs-review-gate to run behavior review for note_text_csv_export_core.
```
