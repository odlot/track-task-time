use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Duration, Local, TimeZone, Utc};
use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Parser)]
#[command(
    name = "ttt",
    about = "Track task time from the command line",
    after_help = "Examples:\n  ttt start \"Write docs\"\n  ttt pause\n  ttt resume\n  ttt status\n  ttt report\n  ttt stop"
)]
struct Cli {
    #[arg(
        long = "data-file",
        value_name = "PATH",
        help = "Override the default data file location"
    )]
    data_file: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Start tracking a task")]
    Start {
        #[arg(value_name = "TASK", help = "Task name to track")]
        task: String,
    },
    #[command(about = "Stop the active or paused task")]
    Stop,
    #[command(about = "Pause the active task")]
    Pause,
    #[command(about = "Resume the paused task")]
    Resume,
    #[command(about = "Show the current task and elapsed time")]
    Status,
    #[command(about = "Show the data file location")]
    Location,
    #[command(about = "Show today's totals (default)")]
    Report {
        #[arg(long, help = "Report today's totals (default)")]
        today: bool,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct Store {
    version: u32,
    tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    id: String,
    name: String,
    created_at: DateTime<Utc>,
    closed_at: Option<DateTime<Utc>>,
    segments: Vec<Segment>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Segment {
    start_at: DateTime<Utc>,
    end_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TaskState {
    Active,
    Paused,
}

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
        Command::Report { today: _ } => {
            let report = report_today(&store, now);
            if report.is_empty() {
                println!("No entries for today.");
                return;
            }
            for (name, seconds) in report {
                println!("{} — {}", name, format_duration(seconds));
            }
        }
    }
}

fn data_file_path(custom: Option<PathBuf>) -> PathBuf {
    if let Some(path) = custom {
        return path;
    }

    if let Some(dirs) = ProjectDirs::from("com", "ttt", "ttt") {
        return dirs.data_dir().join("ttt.json");
    }

    PathBuf::from("ttt.json")
}

fn load_store(path: &Path) -> Result<Store, String> {
    if !path.exists() {
        return Ok(Store {
            version: 1,
            tasks: Vec::new(),
        });
    }

    let contents = fs::read_to_string(path).map_err(|err| err.to_string())?;
    let store: Store = serde_json::from_str(&contents).map_err(|err| err.to_string())?;
    Ok(store)
}

fn save_store(path: &Path, store: &Store) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }

    let payload = serde_json::to_string_pretty(store).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

fn current_task_state(store: &Store) -> Option<(usize, TaskState)> {
    for (idx, task) in store.tasks.iter().enumerate() {
        if task.closed_at.is_some() {
            continue;
        }
        let has_open_segment = task.segments.iter().any(|seg| seg.end_at.is_none());
        if has_open_segment {
            return Some((idx, TaskState::Active));
        }
        if !task.segments.is_empty() {
            return Some((idx, TaskState::Paused));
        }
    }
    None
}

fn active_task_name(store: &Store) -> Option<String> {
    current_task_state(store)
        .and_then(|(idx, state)| {
            if state == TaskState::Active {
                Some(idx)
            } else {
                None
            }
        })
        .map(|idx| store.tasks[idx].name.clone())
}

fn start_task(store: &mut Store, name: String, now: DateTime<Utc>) {
    let task = Task {
        id: Uuid::new_v4().to_string(),
        name,
        created_at: now,
        closed_at: None,
        segments: vec![Segment {
            start_at: now,
            end_at: None,
        }],
    };
    store.tasks.push(task);
}

fn stop_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    if let Some(segment) = task.segments.iter_mut().find(|seg| seg.end_at.is_none()) {
        segment.end_at = Some(now);
    }
    task.closed_at = Some(now);
}

fn pause_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    if let Some(segment) = task.segments.iter_mut().find(|seg| seg.end_at.is_none()) {
        segment.end_at = Some(now);
    }
}

fn resume_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    task.segments.push(Segment {
        start_at: now,
        end_at: None,
    });
}

fn total_elapsed(task: &Task, now: DateTime<Utc>) -> i64 {
    task.segments
        .iter()
        .map(|seg| segment_duration(seg, now))
        .sum()
}

fn segment_duration(segment: &Segment, now: DateTime<Utc>) -> i64 {
    let end = segment.end_at.unwrap_or(now);
    let duration = end - segment.start_at;
    duration.num_seconds().max(0)
}

fn report_today(store: &Store, now: DateTime<Utc>) -> Vec<(String, i64)> {
    let now_local = now.with_timezone(&Local);
    let date = now_local.date_naive();
    let start_local = date.and_hms_opt(0, 0, 0).unwrap();
    let end_local = start_local + Duration::days(1);

    let start_utc = Local
        .from_local_datetime(&start_local)
        .unwrap()
        .with_timezone(&Utc);
    let end_utc = Local
        .from_local_datetime(&end_local)
        .unwrap()
        .with_timezone(&Utc);

    let mut totals: HashMap<String, (String, i64)> = HashMap::new();

    for task in &store.tasks {
        let mut seconds = 0i64;
        for segment in &task.segments {
            seconds += overlap_seconds(segment, start_utc, end_utc, now);
        }
        if seconds == 0 {
            continue;
        }
        let key = task.name.to_lowercase();
        let entry = totals.entry(key).or_insert_with(|| (task.name.clone(), 0));
        entry.1 += seconds;
    }

    let mut output: Vec<(String, i64)> = totals.into_values().collect();
    output.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));
    output
}

fn overlap_seconds(
    segment: &Segment,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> i64 {
    let segment_end = segment.end_at.unwrap_or(now);
    if segment_end <= window_start || segment.start_at >= window_end {
        return 0;
    }
    let start = if segment.start_at > window_start {
        segment.start_at
    } else {
        window_start
    };
    let end = if segment_end < window_end {
        segment_end
    } else {
        window_end
    };
    (end - start).num_seconds().max(0)
}

fn format_duration(seconds: i64) -> String {
    let total = seconds.max(0);
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

fn prompt_yes_no(message: &str) -> bool {
    print!("{}", message);
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("{}", message);
    std::process::exit(2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_duration_hhmmss() {
        assert_eq!(format_duration(0), "00:00:00");
        assert_eq!(format_duration(59), "00:00:59");
        assert_eq!(format_duration(60), "00:01:00");
        assert_eq!(format_duration(3661), "01:01:01");
    }

    #[test]
    fn current_task_state_active_and_paused() {
        let now = Utc::now();
        let active = Task {
            id: "active".into(),
            name: "Active".into(),
            created_at: now,
            closed_at: None,
            segments: vec![Segment {
                start_at: now,
                end_at: None,
            }],
        };
        let paused = Task {
            id: "paused".into(),
            name: "Paused".into(),
            created_at: now,
            closed_at: None,
            segments: vec![Segment {
                start_at: now,
                end_at: Some(now),
            }],
        };

        let store = Store {
            version: 1,
            tasks: vec![active],
        };
        let state = current_task_state(&store);
        assert_eq!(state, Some((0, TaskState::Active)));

        let store = Store {
            version: 1,
            tasks: vec![paused],
        };
        let state = current_task_state(&store);
        assert_eq!(state, Some((0, TaskState::Paused)));
    }

    #[test]
    fn total_elapsed_counts_open_segment() {
        let start = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let now = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap();
        let task = Task {
            id: "task".into(),
            name: "Task".into(),
            created_at: start,
            closed_at: None,
            segments: vec![Segment {
                start_at: start,
                end_at: None,
            }],
        };
        assert_eq!(total_elapsed(&task, now), 1800);
    }

    #[test]
    fn overlap_seconds_handles_window_edges() {
        let seg_start = Utc.with_ymd_and_hms(2025, 1, 1, 9, 0, 0).unwrap();
        let seg_end = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let window_start = Utc.with_ymd_and_hms(2025, 1, 1, 9, 30, 0).unwrap();
        let window_end = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap();
        let segment = Segment {
            start_at: seg_start,
            end_at: Some(seg_end),
        };

        assert_eq!(
            overlap_seconds(&segment, window_start, window_end, window_end),
            1800
        );
    }
}
