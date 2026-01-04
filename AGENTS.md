# Repository Guidelines

## Project Structure & Module Organization

- Root contains project docs and tooling (`README.md`, `LICENSE`, `Dockerfile`, `.gitignore`).
- `rust-toolchain.toml` pins the Rust toolchain for local and CI usage.
- `app/` contains the Rust CLI crate (`Cargo.toml`, `Cargo.lock`, `src/`).
- `app/src/` modules: `cli.rs`, `model.rs`, `storage.rs`, `tasks.rs`, `report.rs`, `list.rs`, `edit.rs`, `prompt.rs`, `time.rs`, `main.rs`.
- `docs/` contains the product requirements and detailed design documents.
- `.github/workflows/ci.yml` runs Rust formatting and tests on pushes and PRs.
- `.github/workflows/release.yml` validates fmt/clippy/tests, builds binaries, and publishes a GitHub release on `v*.*.*` tags.
- `.pre-commit-config.yaml` runs Rust formatting, check, and clippy before commits.
- `app/target/` is build output and should not be committed.

## Build, Test, and Development Commands

- Build the CLI: `cd app && cargo build`
- Run tests: `cd app && cargo test`
- Check formatting: `cd app && cargo fmt --check`
- Run linting: `cd app && cargo clippy -- -D warnings`
- Pre-commit hooks: `python3 -m venv .venv`, then `. .venv/bin/activate`, `pip install pre-commit`, `pre-commit install`
- Build container: `docker build . -t track-task-time:0.1.3`
- Run container: `docker container run -d -it --rm --mount type=bind,src=./,dst=/app track-task-time:0.1.4 bash`
- Release process: update `CHANGELOG.md` and `app/Cargo.toml` version, then tag `vX.Y.Z` and push the tag.

## Coding Style & Naming Conventions

- Rust code lives under `app/src/`. Prefer `snake_case` for files and functions and Rust module conventions.
- Use `cargo fmt` (rustfmt) when available.

## Testing Guidelines

- Tests are standard Rust unit tests in the crate; run with `cd app && cargo test`.

## Commit & Pull Request Guidelines

- Git history is not available in this workspace, so no commit convention can be inferred.
- Use short, imperative commit messages (for example, "Add initial build script") and include scope when helpful.
- Pull requests should explain the change, list any new commands, and include screenshots only for UI changes.
- Never merge unless CI is green; wait for checks to complete.

## Best Practices

- Run pre-commit before pushing to catch fmt, check, and clippy issues.
- Keep `CHANGELOG.md`, `app/Cargo.toml`, and tags aligned for releases.
- Update README/PRD/DDD when CLI behavior or commands change.
- Prefer small, focused PRs to keep reviews fast and CI reliable.

## UX Guidelines

- Keep CLI help and README aligned with command arguments and flags.
- Report output starts with a date header, then `HH:MM:SS - HH:MM:SS - Task (HH:MM:SS)`, most recent first.
- List and report output include a total line at the end of the output.
- List output should include status, name, id, and total duration with stable ordering.
- Use interactive prompts when no flags are provided, but keep flags for power users.
- Surface the data file location via `ttt location` and document it.

## Agent-Specific Instructions

- Keep this document updated whenever you add new directories, tooling, or conventions.
