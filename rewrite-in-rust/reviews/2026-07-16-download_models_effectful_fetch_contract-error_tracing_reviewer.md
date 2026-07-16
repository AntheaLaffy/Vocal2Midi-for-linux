# download_models_effectful_fetch_contract - error_tracing_reviewer

Date: 2026-07-16
Decision: fail

## Findings

- Severity: medium
- Location: rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:42
- Issue: The mocked effect boundary does not prove diagnostics for non-HTTP stream
  failures during GitHub asset downloads. The bootstrap says this unit covers
  download-facing helpers through mocked network effects and lists
  `urllib.error.URLError`, `TimeoutError`, and JSON decoding errors as mocked
  boundaries at `rewrite-in-rust/bootstrap/download_models_effectful_fetch_contract.md:51`.
  The checker only injects `URLError` for `github_api_asset_sizes` at
  `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:115`
  and `HTTPError` for `download_github_model` at
  `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:197`.
  It never injects `URLError` or `TimeoutError` from `stream_download`, and the
  Rust model only accepts `stream_result.http_error` or `stream_result.bytes` at
  `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:191`
  and `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:209`.
  Legacy `download_github_model` catches `HTTPError` at `download_models.py:368`
  but allows other stream exceptions from `download_models.py:342` to escape.
  Without a fixture, the Rust contract cannot say whether a later bridge should
  preserve that exception/traceback behavior, convert it to a user-visible
  `stderr_lines` entry, or intentionally narrow the contract.
- Evidence: Fixture lines 7 through 12 cover GitHub success, size mismatch,
  bad zip, unsafe layout, marker missing, and HTTP 404 only
  (`rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:7`).
  A manual legacy probe with `stream_download` patched to raise
  `urllib.error.URLError("offline")` printed the initial download lines and then
  raised `URLError: <urlopen error offline>` rather than returning `False`.
- Required fix: Add fixture/checker/Rust coverage for stream `URLError` and
  timeout failures, or add a control-plane record that explicitly excludes
  non-HTTP stream exceptions from this unit and assigns them to a later
  promotion/bridge error contract.

- Severity: low
- Location: rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:352
- Issue: Qwen cleanup unlink failures are swallowed by legacy Python but are not
  representable in the Rust fixture model. Legacy `_cleanup_qwen_artifacts`
  ignores `OSError` while removing `.gitattributes` and immediate
  `*.incomplete` files at `download_models.py:440` through
  `download_models.py:452`. The current fixture only covers successful cleanup at
  `rewrite-in-rust/fixtures/download_models_effectful_fetch_contract.jsonl:16`,
  and the Rust helper always removes matching immediate entries and logs success
  at `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:356`
  through `rewrite-in-rust/rust/crates/v2m-core/src/download_models_effectful.rs:369`.
  If cleanup fails in production, rollback diagnosis depends on knowing whether
  Python's silent retention is preserved or deliberately improved.
- Evidence: The dependency record includes Qwen artifact cleanup in this unit at
  `rewrite-in-rust/dependencies/download_models_effectful_fetch_contract.yaml:16`,
  but the Python checker uses real temp-file cleanup only and does not simulate
  unlink failure at
  `rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py:343`.
- Required fix: Add a mocked cleanup-failure fixture that proves the retained
  file/log behavior, or explicitly document cleanup unlink failures as a
  legacy-owned production IO risk outside this mocked Rust state machine.

## Checks

- `uv run python rewrite-in-rust/bootstrap/check_download_models_effectful_fetch_contract.py`: pass.
- `cargo test --manifest-path rewrite-in-rust/rust/Cargo.toml download_models_effectful`: pass; `download_models_effectful::tests::download_models_effectful_fetch_fixtures_match` passed.
- `cargo fmt --manifest-path rewrite-in-rust/rust/Cargo.toml --all -- --check`: pass.
- `git diff --check -- rewrite-in-rust/manifest.yaml rewrite-in-rust/rust/crates/v2m-core/src/lib.rs`: pass.
- Untracked-unit whitespace scan with `git diff --check --no-index /dev/null <file>` output checks for the target record, dependency doc, bootstrap doc, fixture JSONL, Python checker, and Rust implementation: pass with no whitespace diagnostics.
- `rg -n "proxy|7890|token|password|secret|Authorization|Bearer" ...`: no proxy, 7890, token, password, secret, authorization, or bearer-token matches in the target unit artifacts; one unrelated `git-lfs` prose match in `download_models.py:436`.
- Manual legacy probe for `download_github_model` with mocked `stream_download`
  raising `urllib.error.URLError("offline")`: raised `URLError` after initial
  download output, confirming this stream-error shape is not represented by the
  fixture/Rust contract.

## Residual Risk

The current fixture set proves user-visible messages for GitHub API size-cache
fallback, stream progress success, HTTP 404 mapping, archive/marker failures,
CLI missing/install/run failures, Qwen weight-marker failures, source fallback,
and unknown Qwen source. It does not prove Python traceback text for escaping
download exceptions, subprocess exceptions beyond missing executable,
package-install return-code detail, or real filesystem cleanup errors.

This unit has no proxy URL or port-redaction surface. Proxy redaction, including
manual proxy URLs such as port 7890, belongs to the Web model download execution
contract rather than `download_models_effectful_fetch_contract`.

## Promotion Note

This error-tracing gate blocks coordinator state update for
`download_models_effectful_fetch_contract` until the non-HTTP stream failure
diagnostic boundary is fixture-backed or explicitly re-scoped. Python remains
the runtime owner and rollback route.
