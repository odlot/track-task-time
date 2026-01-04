mod cli;
mod edit;
mod list;
mod model;
mod prompt;
mod report;
mod storage;
mod tasks;
mod time;

use chrono::{Local, Utc};
use clap::Parser;

use crate::cli::{Cli, Command};
use crate::edit::{apply_task_edits, edit_task_interactive, resolve_task_index};
use crate::list::{ListWindow, list_header, list_tasks};
use crate::model::TaskState;
use crate::prompt::prompt_yes_no;
use crate::report::report_today;
use crate::storage::{data_file_path, load_store, save_store};
use crate::tasks::{
    active_task_name, current_task_state, pause_task, resume_task, start_task, stop_task,
    total_elapsed,
};
use crate::time::{format_duration, format_time_local_display};

fn main() {
    let cli = Cli::parse();
    let data_file = data_file_path(cli.data_file);

    let mut store = match load_store(&data_file) {
        Ok(store) => store,
        Err(err) => exit_with_error(&err),
    };

    let now = Utc::now();

    match cli.command {
        Command::Start { task } => {
            if let Some((idx, state)) = current_task_state(&store) {
                let existing_name = store.tasks[idx].name.clone();
                let prompt = match state {
                    TaskState::Active => format!(
                        "Active task \"{}\" is running. Stop it and start \"{}\"? [y/N] ",
                        existing_name, task
                    ),
                    TaskState::Paused => format!(
                        "Task \"{}\" is paused. Abandon it and start \"{}\"? [y/N] ",
                        existing_name, task
                    ),
                };
                if !prompt_yes_no(&prompt) {
                    exit_with_error("Canceled.");
                }
                stop_task(&mut store, idx, now);
            }
            start_task(&mut store, task, now);
            save_store(&data_file, &store).unwrap_or_else(|err| exit_with_error(&err));
        }
        Command::Stop => {
            if let Some((idx, _)) = current_task_state(&store) {
                stop_task(&mut store, idx, now);
                save_store(&data_file, &store).unwrap_or_else(|err| exit_with_error(&err));
            } else {
                exit_with_error("No active or paused task. Start one with \"ttt start <task>\".");
            }
        }
        Command::Pause => {
            if let Some((idx, state)) = current_task_state(&store) {
                if state == TaskState::Active {
                    pause_task(&mut store, idx, now);
                    save_store(&data_file, &store).unwrap_or_else(|err| exit_with_error(&err));
                } else {
                    exit_with_error("Task is already paused. Resume it with \"ttt resume\".");
                }
            } else {
                exit_with_error("No active task. Start one with \"ttt start <task>\".");
            }
        }
        Command::Resume => match current_task_state(&store) {
            Some((idx, TaskState::Paused)) => {
                resume_task(&mut store, idx, now);
                save_store(&data_file, &store).unwrap_or_else(|err| exit_with_error(&err));
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
                println!("Active: {} — {}", task.name, format_duration(elapsed));
            }
            Some((idx, TaskState::Paused)) => {
                let task = &store.tasks[idx];
                let elapsed = total_elapsed(task, now);
                println!("Paused: {} — {}", task.name, format_duration(elapsed));
            }
            None => println!("No active task."),
        },
        Command::Location => {
            println!("{}", data_file.display());
        }
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
        }
        Command::Report { today: _ } => {
            let report = report_today(&store, now);
            if report.is_empty() {
                println!("No entries for today.");
                return;
            }
            let report_date = now.with_timezone(&Local).date_naive();
            println!("{}", report_date);
            for entry in report {
                println!(
                    "{} - {} - {} ({})",
                    format_time_local_display(entry.start_at),
                    format_time_local_display(entry.end_at),
                    entry.name,
                    format_duration(entry.seconds)
                );
            }
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

            save_store(&data_file, &store).unwrap_or_else(|err| exit_with_error(&err));
        }
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2);
}
