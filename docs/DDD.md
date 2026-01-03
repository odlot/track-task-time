# Detailed Design Document (DDD)

## System Architecture

`ttt` is a single-binary CLI that reads and writes a local JSON file.

High-level components:

- CLI interface (argument parsing and command dispatch).
- Task state manager (start, stop, pause, resume).
- Task editor (interactive selection and edits via flags).
- Reporting engine (daily entries with start/end times).
- Storage layer (load/save JSON store).

Flow:

1) Parse command and resolve data file path.
2) Load JSON store (or initialize an empty store).
3) Apply command logic and update the store.
4) Persist the store and print output.

## Technology Stack

- Language: Rust 2024 edition.
- CLI parsing: `clap` (derive).
- Time handling: `chrono` with local and UTC conversions.
- JSON serialization: `serde` + `serde_json`.
- Data directory resolution: `directories`.
- IDs: `uuid` v4.

## Data Model

Store (root JSON object):

- `version` (u32): store version.
- `tasks` (array): list of tracked tasks.

Task:

- `id` (string): UUID v4.
- `name` (string): task name as entered.
- `created_at` (UTC timestamp).
- `closed_at` (optional UTC timestamp).
- `segments` (array of Segment).

Segment:

- `start_at` (UTC timestamp).
- `end_at` (optional UTC timestamp).

Notes:

- An active task has a segment with `end_at = null`.
- A paused task has no open segment but `closed_at = null`.
- A stopped task has `closed_at` set.

## API Design

CLI commands and flags:

- `ttt start <task>`
  - Prompts to stop an existing active or paused task.
- `ttt stop`
  - Stops the active or paused task.
- `ttt pause`
  - Pauses the active task.
- `ttt resume`
  - Resumes the paused task.
- `ttt status`
  - Shows current task and elapsed time.
- `ttt location`
  - Prints the resolved data file path.
- `ttt edit`
  - Interactive task selection with prompts to edit names and times.
  - Flags: `--id`, `--index`, `--name`, `--created-at`, `--closed-at`, `--segment-edit`.
- `ttt report`
  - Prints a date header and today's entries with start/end times (most recent first).
- Global flag: `--data-file <path>` overrides the default data location.

Exit behavior:

- Errors print a message to stderr and exit with code 2.

## Storage Design

- Default path: OS-specific user data directory via `directories`.
- Format: pretty-printed JSON for readability.
- Persistence: write file on state changes (start/stop/pause/resume).
- Edits update task metadata and segment timestamps in-place.

## Scalability Considerations

- Store size grows linearly with tasks and segments; all data is loaded into memory.
- Reporting is O(tasks * segments) for the current day.
- No file locking or concurrency control; concurrent runs could race.
- Potential future optimizations:
  - Incremental indexing by day for faster reports.
  - Archiving closed tasks to a separate file.
  - File locking to prevent concurrent writes.
