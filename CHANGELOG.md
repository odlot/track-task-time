# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
