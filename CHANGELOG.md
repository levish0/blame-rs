# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.8] - 2026-02-13

### Changed
- Updated CI build workflow to use:
  - `actions/checkout@v4`
  - `dtolnay/rust-toolchain@stable`
  - `Swatinem/rust-cache@v2`
  - Added `workflow_dispatch` trigger
- Updated crate metadata documentation URL to `https://docs.rs/blame-rs`.
- Updated README dependency example to `blame-rs = "0.1.8"`.
- Clarified README wording around moved/reordered lines behavior.

### Fixed
- Replaced panic paths in core blame logic with `BlameError::InvalidInput` error returns.
- Switched `DiffAlgorithm` and `BlameOptions` to derived `Default` implementations.
- Improved revision file sorting in `examples/multi_revision.rs` using numeric revision parsing.
- Cleaned clippy warnings in examples and tests (unused imports, format literals, expect formatting).

### Added
- Added workspace `xtask` crate with release helpers:
  - `cargo xtask publish-dry`
  - `cargo xtask publish`
- Added boundary-condition tests for:
  - Empty revisions
  - Single revision input
  - Trailing newline behavior
  - CRLF inputs
  - Reordered lines (no panic regression)
