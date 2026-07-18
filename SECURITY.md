# Security Policy

This document is for users and contributors reporting security problems in
Vocal2Midi.

## Supported Versions

Security fixes target the current `main` branch. Historical tags and local
model bundles are not maintained as separate security release lines.

## Reporting a Vulnerability

Do not open a public issue for an unpatched vulnerability. Use the repository's
[private vulnerability reporting
page](https://github.com/AntheaLaffy/Vocal2Midi-for-linux/security/advisories/new).
Include the affected revision, platform, reproduction steps, impact, and any
known mitigation. Do not include real user audio, credentials, access tokens,
or private filesystem contents.

If private reporting is unavailable, contact a repository maintainer through
their GitHub profile and ask for a private channel before sharing details.

## Security Boundaries

- The Web backend has no authentication and exposes local filesystem and model
  management operations. Bind it only on trusted networks.
- Model downloads and some workflows execute subprocesses and access the
  network.
- Model files, audio, lyrics, generated outputs, and local settings may contain
  sensitive data. Keep them out of issue reports and commits.
- The Rust migration does not become a production security boundary merely by
  reaching behavior parity. Runtime ownership changes require the reviews and
  rollback evidence named in `rewrite-in-rust/manifest.yaml`.

The current HTTP and download constraints are documented in
[`docs/web-api.md`](docs/web-api.md).
