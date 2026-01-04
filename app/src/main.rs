mod cli;
mod crypto;
mod edit;
mod list;
mod model;
mod prompt;
mod report;
mod storage;
mod tasks;
mod time;

use chrono::{DateTime, Local, Utc};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::crypto::read_passphrase;
use crate::edit::{apply_task_edits, edit_task_interactive, resolve_task_index};
use crate::list::{ListWindow, list_header, list_tasks};
use crate::model::{Task, TaskState};
use crate::prompt::{prompt_line, prompt_required, prompt_yes_no};
use crate::report::report_today;
use crate::storage::{data_file_path, list_backups, load_store, save_store};
use crate::tasks::{
    active_task_name, current_task_state, pause_task, resume_task, start_task, stop_task,
    total_elapsed,
};
use crate::time::{format_duration, format_time_local_display};

fn main() {
    let cli = Cli::parse();
    let data_file = data_file_path(cli.data_file);

    let now = Utc::now();
    let command = cli.command;

    let data_exists = data_file.exists();

    if matches!(&command, Command::Location) {
        println!("{}", data_file.display());
        return;
    }
    if matches!(&command, Command::Version) {
        println!("ttt {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    if matches!(&command, Command::Restore) {
        let backups = list_backups(&data_file);
        if backups.is_empty() {
            exit_with_error("No backups found.");
        }
        println!("Available backups:");
        for (idx, entry) in backups.iter().enumerate() {
            println!("{:>3}) {}", idx + 1, format_backup_entry(entry));
        }
        let input = prompt_line("Select backup number (or 'q' to cancel): ")
            .unwrap_or_else(|err| exit_with_error(&err));
        if input.is_empty() || input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit")
        {
            exit_with_error("Canceled.");
        }
        let selection: usize = input
            .parse()
            .map_err(|_| "Invalid selection. Enter a number from the list.".to_string())
            .unwrap_or_else(|err| exit_with_error(&err));
        if selection == 0 || selection > backups.len() {
            exit_with_error(&format!(
                "Backup selection must be between 1 and {}.",
                backups.len()
            ));
        }
        let entry = &backups[selection - 1];
        let label = format_backup_entry(entry);
        if !prompt_yes_no(&format!("Restore {}? [y/N] ", label)) {
            exit_with_error("Canceled.");
        }
        let passphrase = read_passphrase(false).unwrap_or_else(|err| exit_with_error(&err));
        let store = match load_store(&entry.path, &passphrase) {
            Ok(store) => store,
            Err(err) => exit_with_error(&err),
        };
        save_store(&data_file, &store, &passphrase).unwrap_or_else(|err| exit_with_error(&err));
        println!("Restored backup {}", entry.path.display());
        return;
    }
    if matches!(&command, Command::Rekey) {
        if !data_exists {
            exit_with_error("No data file found. Start tracking with \"ttt start\" first.");
        }
        let current_passphrase = read_passphrase(false).unwrap_or_else(|err| exit_with_error(&err));
        let store = match load_store(&data_file, &current_passphrase) {
            Ok(store) => store,
            Err(err) => exit_with_error(&err),
        };
        let new_passphrase = read_passphrase(true).unwrap_or_else(|err| exit_with_error(&err));
        save_store(&data_file, &store, &new_passphrase).unwrap_or_else(|err| exit_with_error(&err));
        println!("Passphrase updated for {}", data_file.display());
        return;
    }

    let will_write = matches!(
        &command,
        Command::Start { .. }
            | Command::Stop
            | Command::Pause
            | Command::Resume
            | Command::Edit { .. }
    );
    let is_new_store = !data_exists;
    let confirm_passphrase = will_write && is_new_store;
    let passphrase =
        read_passphrase(confirm_passphrase).unwrap_or_else(|err| exit_with_error(&err));
    let mut store = match load_store(&data_file, &passphrase) {
        Ok(store) => store,
        Err(err) => exit_with_error(&err),
    };

    match command {
        Command::Start { task } => {
            let task_name = match task {
                Some(name) if !name.trim().is_empty() => name,
                Some(_) => exit_with_error("Task name cannot be empty."),
                None => prompt_required("Task name: ", "Task name")
                    .unwrap_or_else(|err| exit_with_error(&err)),
            };
            if let Some((idx, state)) = current_task_state(&store) {
                let existing_name = store.tasks[idx].name.clone();
                let prompt = match state {
                    TaskState::Active => format!(
                        "Active task \"{}\" is running. Stop it and start \"{}\"? [y/N] ",
                        existing_name, task_name
                    ),
                    TaskState::Paused => format!(
                        "Task \"{}\" is paused. Abandon it and start \"{}\"? [y/N] ",
                        existing_name, task_name
                    ),
                };
                if !prompt_yes_no(&prompt) {
                    exit_with_error("Canceled.");
                }
                stop_task(&mut store, idx, now);
            }
            start_task(&mut store, task_name.clone(), now);
            save_store(&data_file, &store, &passphrase).unwrap_or_else(|err| exit_with_error(&err));
            println!(
                "Started: {} at {}",
                task_name,
                format_time_local_display(now)
            );
            if is_new_store {
                println!("Created encrypted data file at {}", data_file.display());
            }
        }
        Command::Stop => {
            if let Some((idx, _)) = current_task_state(&store) {
                let task_name = store.tasks[idx].name.clone();
                stop_task(&mut store, idx, now);
                let elapsed = total_elapsed(&store.tasks[idx], now);
                save_store(&data_file, &store, &passphrase)
                    .unwrap_or_else(|err| exit_with_error(&err));
                println!(
                    "Stopped: {} at {} (total {})",
                    task_name,
                    format_time_local_display(now),
                    format_duration(elapsed)
                );
                if is_new_store {
                    println!("Created encrypted data file at {}", data_file.display());
                }
            } else {
                exit_with_error("No active or paused task. Start one with \"ttt start <task>\".");
            }
        }
        Command::Pause => {
            if let Some((idx, state)) = current_task_state(&store) {
                if state == TaskState::Active {
                    let task_name = store.tasks[idx].name.clone();
                    pause_task(&mut store, idx, now);
                    let elapsed = total_elapsed(&store.tasks[idx], now);
                    save_store(&data_file, &store, &passphrase)
                        .unwrap_or_else(|err| exit_with_error(&err));
                    println!(
                        "Paused: {} at {} (total {})",
                        task_name,
                        format_time_local_display(now),
                        format_duration(elapsed)
                    );
                    if is_new_store {
                        println!("Created encrypted data file at {}", data_file.display());
                    }
                } else {
                    exit_with_error("Task is already paused. Resume it with \"ttt resume\".");
                }
            } else {
                exit_with_error("No active task. Start one with \"ttt start <task>\".");
            }
        }
        Command::Resume => match current_task_state(&store) {
            Some((idx, TaskState::Paused)) => {
                let task_name = store.tasks[idx].name.clone();
                resume_task(&mut store, idx, now);
                save_store(&data_file, &store, &passphrase)
                    .unwrap_or_else(|err| exit_with_error(&err));
                println!(
                    "Resumed: {} at {}",
                    task_name,
                    format_time_local_display(now)
                );
                if is_new_store {
                    println!("Created encrypted data file at {}", data_file.display());
                }
            }
            Some((_, TaskState::Active)) => {
                let active_name = active_task_name(&store).unwrap_or_default();
                exit_with_error(&format!(
                    "Task \"{}\" is already running. Pause it with \"ttt pause\".",
                    active_name
                ));
            }
            None => {
                exit_with_error("No paused task. Start one with \"ttt start <task>\".");
            }
        },
        Command::Status => match current_task_state(&store) {
            Some((idx, TaskState::Active)) => {
                let task = &store.tasks[idx];
                let elapsed = total_elapsed(task, now);
                let started_at = active_segment_start(task).unwrap_or(task.created_at);
                println!(
                    "Active: {} - {} (since {})",
                    task.name,
                    format_duration(elapsed),
                    format_time_local_display(started_at)
                );
            }
            Some((idx, TaskState::Paused)) => {
                let task = &store.tasks[idx];
                let elapsed = total_elapsed(task, now);
                let paused_at = last_segment_end(task).unwrap_or(task.created_at);
                println!(
                    "Paused: {} - {} (paused at {})",
                    task.name,
                    format_duration(elapsed),
                    format_time_local_display(paused_at)
                );
            }
            None => println!("No active task. Start one with \"ttt start\"."),
        },
        Command::List { today, week } => {
            if today && week {
                exit_with_error("Use either --today or --week, not both.");
            }
            let window = if today {
                ListWindow::Today
            } else if week {
                ListWindow::Week
            } else {
                ListWindow::All
            };
            let entries = list_tasks(&store, now, window);
            if entries.is_empty() {
                println!("No matching tasks.");
                return;
            }
            if let Some(header) = list_header(now, window) {
                println!("{}", header);
            }
            let total_seconds: i64 = entries.iter().map(|entry| entry.seconds).sum();
            for (idx, entry) in entries.iter().enumerate() {
                println!(
                    "{:>3}) [{}] {} ({}) total {}",
                    idx + 1,
                    entry.status,
                    entry.name,
                    entry.id,
                    format_duration(entry.seconds)
                );
            }
            println!("Total: {}", format_duration(total_seconds));
        }
        Command::Report { today: _ } => {
            let report = report_today(&store, now);
            if report.is_empty() {
                println!("No entries for today.");
                return;
            }
            let report_date = now.with_timezone(&Local).date_naive();
            println!("{}", report_date);
            let total_seconds: i64 = report.iter().map(|entry| entry.seconds).sum();
            for entry in report {
                println!(
                    "{} - {} - {} ({})",
                    format_time_local_display(entry.start_at),
                    format_time_local_display(entry.end_at),
                    entry.name,
                    format_duration(entry.seconds)
                );
            }
            println!("Total: {}", format_duration(total_seconds));
        }
        Command::Edit {
            id,
            index,
            name,
            created_at,
            closed_at,
            segment_edit,
        } => {
            let idx = match resolve_task_index(&store, now, id, index) {
                Ok(idx) => idx,
                Err(err) => exit_with_error(&err),
            };

            let task = &mut store.tasks[idx];
            let has_edits = name.is_some()
                || created_at.is_some()
                || closed_at.is_some()
                || !segment_edit.is_empty();

            if has_edits {
                apply_task_edits(task, name, created_at, closed_at, segment_edit, now)
                    .unwrap_or_else(|err| exit_with_error(&err));
            } else {
                edit_task_interactive(task, now).unwrap_or_else(|err| exit_with_error(&err));
            }

            save_store(&data_file, &store, &passphrase).unwrap_or_else(|err| exit_with_error(&err));
            if is_new_store {
                println!("Created encrypted data file at {}", data_file.display());
            }
        }
        Command::Location => {}
        Command::Rekey => {}
        Command::Restore => {}
        Command::Version => {}
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2);
}

fn active_segment_start(task: &Task) -> Option<chrono::DateTime<Utc>> {
    task.segments
        .iter()
        .find(|segment| segment.end_at.is_none())
        .map(|segment| segment.start_at)
}

fn last_segment_end(task: &Task) -> Option<chrono::DateTime<Utc>> {
    task.segments
        .iter()
        .rev()
        .find_map(|segment| segment.end_at)
}

fn format_backup_entry(entry: &crate::storage::BackupEntry) -> String {
    let name = entry
        .path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("backup");
    let modified = entry
        .modified
        .map(|time| {
            DateTime::<Local>::from(time)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_else(|| "unknown".to_string());
    format!("{} (modified {}, {} bytes)", name, modified, entry.size)
}
