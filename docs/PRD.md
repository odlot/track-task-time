# Product Requirements Document (PRD)

## Overview

`ttt` is a small, cross-platform CLI for tracking task time. It supports explicit start/stop and pause/resume commands, stores data locally in JSON, reports daily totals, and lets users edit task names and times.

## Problem Statement

People who work across multiple tasks need a lightweight way to track time without web accounts, background daemons, or complex UI. Existing tools often require network access or lack clear pause/resume behavior.

## Target Users

- Individual developers, writers, and makers who want a local CLI tool.
- Power users who prefer terminal workflows and scripts.
- Privacy-focused users who want local-only storage.

## Goals

- Make it fast to start and stop tracking a task from the terminal.
- Keep data local, portable, and human-readable.
- Provide a simple daily report aggregated by task name.

## Non-Goals

- Multi-user or collaborative tracking.
- Cloud sync, accounts, or remote storage.
- Automatic task detection or background tracking.

## Key Features

- Start a single active task with a name.
- Pause and resume without splitting into separate tasks.
- Stop and close a task explicitly.
- Show current status and elapsed time.
- Generate a daily report listing tasks with start/end times (most recent first).
- Store data locally in a JSON file with an override flag.
- Edit task names and timestamps after the fact.
- Show the resolved data file location.
- List tasks with totals and IDs, with optional date filters.

## User Flows

- Start a new task
  - Run `ttt start <task>`.
  - If another task is active or paused, confirm stopping it before continuing.
- Pause and resume
  - Run `ttt pause` to pause the active task.
  - Run `ttt resume` to continue a paused task.
- Stop a task
  - Run `ttt stop` to close the active or paused task.
- Check status
  - Run `ttt status` to view the current task and elapsed time.
- Get a daily report
  - Run `ttt report` to see the date header and entries with start/end times for today.
- Edit a task
  - Run `ttt edit` and select a task from the list.
  - Update the task name and timestamps interactively, or use flags.
- Find the data file
  - Run `ttt location` to print the data file path.
- List tasks
  - Run `ttt list` for all tasks or filter with `--today` / `--week`.

## Success Criteria

- Users can start and stop tracking in under 10 seconds from a clean terminal.
- Data is stored locally in a single JSON file with no network access.
- Daily report output is stable and deterministic for the same input.
- Command errors are actionable and guide users to the next step.
- Users can correct task names and timing without editing JSON by hand.
- Users can quickly find task IDs via the list output.

## Assumptions and Constraints

- Only one active or paused task at a time.
- Local machine clock is used for timestamps.
- Data is stored in a per-user data directory unless overridden.
