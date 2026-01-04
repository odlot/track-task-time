# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [v0.3.1] - 2026-01-04

### Changed

- Updated README, PRD, DDD, and AGENTS with current CLI behavior and best practices.

## [v0.3.0] - 2026-01-04

### Added

- `list` command to show tasks with totals and IDs, with optional date filters.

## [v0.2.0] - 2026-01-03

### Added

- Pre-commit hooks for `cargo fmt`, `cargo check`, and `cargo clippy`.

### Changed

- Release workflow now validates fmt, clippy, and tests before building artifacts.
- Refactored the CLI into modules for maintainability.

## [v0.1.2] - 2026-01-03

### Fixed

- Report ordering now uses the latest end time for most-recent-first output.

### Changed

- Report output now prints times before the task name and duration.
- Report output includes a date header and shows time-only ranges.

## [v0.1.1] - 2026-01-03

### Added

- `edit` command to rename tasks and adjust time segments.
- `location` command to show the data file path.

## [v0.1.0] - 2026-01-03

### Added

- Initial `ttt` CLI with start/stop/pause/resume/status/report commands.
- Local JSON data store with daily reporting.
