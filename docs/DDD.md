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

## Code Organization

- `cli.rs`: clap definitions and CLI help text.
- `main.rs`: command dispatch and wiring.
- `model.rs`: data structures for tasks and segments.
- `crypto.rs`: encryption, decryption, and passphrase handling.
- `storage.rs`: load/save encrypted JSON store.
- `tasks.rs`: task lifecycle (start/stop/pause/resume/status).
- `report.rs`: report formatting and overlap calculations.
- `list.rs`: list view for all/today/week summaries.
- `edit.rs`: task edits (interactive and flag-based).
- `prompt.rs`: interactive selection and prompts.
- `time.rs`: parsing and formatting for timestamps and durations.

## Technology Stack

- Language: Rust 2024 edition.
- CLI parsing: `clap` (derive).
- Time handling: `chrono` with local and UTC conversions.
- JSON serialization: `serde` + `serde_json`.
- Data directory resolution: `directories`.
- IDs: `uuid` v4.
- Encryption: `argon2` (KDF) and `chacha20poly1305` (AEAD).
- Passphrase input: `rpassword`.

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

- `ttt start [task]`
  - Prompts for a task name if omitted.
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
- `ttt list`
  - Lists tasks with totals and IDs (filters: `--today`, `--week`).
  - Prints a total line for the selected window.
- `ttt edit`
  - Interactive task selection with prompts to edit names and times.
  - Flags: `--id`, `--index`, `--name`, `--created-at`, `--closed-at`, `--segment-edit`.
- `ttt report`
  - Prints a date header and today's entries with start/end times (most recent first).
  - Output format: `HH:MM:SS - HH:MM:SS - Task Name (HH:MM:SS)`.
  - Prints a total line after the entries.
- `ttt rekey`
  - Re-encrypts the data file with a new passphrase.
- `ttt restore`
  - Restores the data file from a recent backup.
- `ttt version`
  - Prints the CLI version.
- Global flag: `--data-file <path>` overrides the default data location.

Exit behavior:

- Errors print a message to stderr and exit with code 2.

## Storage Design

- Default path: OS-specific user data directory via `directories`.
- Format: encrypted JSON envelope with salt, nonce, and ciphertext.
- Persistence: write file on state changes (start/stop/pause/resume).
- Edits update task metadata and segment timestamps in-place.
- Passphrase is required on every run.
- File permissions are set to owner-only when supported.
- Backups are kept in the same directory as `.bak1` through `.bak3`.

## Encryption

- KDF: Argon2id with per-file salt and stored parameters.
- Cipher: XChaCha20-Poly1305 with a random nonce per write.
- File layout: `{ version, kdf, cipher, salt, nonce, ciphertext }` in JSON.
- Keychain integration is out of scope but the KDF/cipher metadata is stored for future extensibility.

## Scalability Considerations

- Store size grows linearly with tasks and segments; all data is loaded into memory.
- Reporting is O(tasks * segments) for the current day.
- No file locking or concurrency control; concurrent runs could race.
- Potential future optimizations:
  - Incremental indexing by day for faster reports.
  - Archiving closed tasks to a separate file.
  - File locking to prevent concurrent writes.

## Quality Gates and Release Automation

- Pre-commit hooks run `cargo fmt`, `cargo check`, and `cargo clippy -D warnings`.
- CI enforces formatting, linting, and tests before merges.
- Releases are tagged with `vX.Y.Z` and publish binaries plus changelog notes.
