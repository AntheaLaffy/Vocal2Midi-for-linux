# 0026 - Close Web Filesystem Picker Behavior Findings

## Context

The first `web_filesystem_picker_contract` behavior review failed on two
fixture/model gaps:

- Rust sorted case-only name ties with an extra original-name tie-breaker, while
  Python's stable sort only keys on directory/file grouping and
  `name.lower()`;
- error response checks were subset-based, so extra fields could appear without
  failing the parity harness.

## Decision

Keep Python's legacy behavior:

- entry ordering is stable by `(item['type'] != 'directory',
  item['name'].lower())` only;
- invalid-mode, missing-path, and unreadable-directory error responses are
  exact response-shape checks.

Add a fixture that preserves enumeration order for case-only directory and file
name ties.

## Consequences

The picker contract remains `reimplemented`, not `verified`, until rerun review
or later coordinator review accepts the follow-up fixes.
