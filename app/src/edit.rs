use chrono::{DateTime, Utc};

use crate::model::{SegmentEdit, Store, Task};
use crate::prompt::{prompt_line, prompt_optional};
use crate::tasks::{task_status, total_elapsed};
use crate::time::{format_datetime_local, format_duration};

pub fn resolve_task_index(
    store: &Store,
    now: DateTime<Utc>,
    id: Option<String>,
    index: Option<usize>,
) -> Result<usize, String> {
    if store.tasks.is_empty() {
        return Err("No tasks to edit.".into());
    }

    if id.is_some() && index.is_some() {
        return Err("Use either --id or --index, not both.".into());
    }

    if let Some(id) = id {
        return store
            .tasks
            .iter()
            .position(|task| task.id == id)
            .ok_or_else(|| format!("No task found with id \"{}\".", id));
    }

    if let Some(index) = index {
        if index == 0 || index > store.tasks.len() {
            return Err(format!(
                "Task index must be between 1 and {}.",
                store.tasks.len()
            ));
        }
        return Ok(index - 1);
    }

    prompt_task_selection(store, now)
}

fn prompt_task_selection(store: &Store, now: DateTime<Utc>) -> Result<usize, String> {
    println!("Select a task to edit:");
    for (idx, task) in store.tasks.iter().enumerate() {
        let id_short = short_id(&task.id);
        let status = task_status(task);
        let elapsed = format_duration(total_elapsed(task, now));
        println!(
            "{:>3}) [{}] {} ({}) total {}",
            idx + 1,
            status,
            task.name,
            id_short,
            elapsed
        );
    }

    let input = prompt_line("Enter task number (or 'q' to cancel): ")?;
    if input.is_empty() || input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
        return Err("Canceled.".into());
    }
    let selection: usize = input
        .parse()
        .map_err(|_| "Invalid selection. Enter a number from the list.".to_string())?;
    if selection == 0 || selection > store.tasks.len() {
        return Err(format!(
            "Task index must be between 1 and {}.",
            store.tasks.len()
        ));
    }
    Ok(selection - 1)
}

fn short_id(id: &str) -> &str {
    if id.len() > 8 { &id[..8] } else { id }
}

pub fn edit_task_interactive(task: &mut Task, now: DateTime<Utc>) -> Result<(), String> {
    println!("Editing task: {}", task.name);

    if let Some(input) = prompt_optional(&format!("Name [{}]: ", task.name))? {
        task.name = input;
    }

    let created_label = format_datetime_local(task.created_at);
    if let Some(input) =
        prompt_optional(&format!("Created at [{}] (RFC3339/now): ", created_label))?
    {
        task.created_at = parse_datetime_input(&input, now, "created at")?;
    }

    let closed_label = match task.closed_at {
        Some(closed_at) => format_datetime_local(closed_at),
        None => "open".to_string(),
    };
    if let Some(input) = prompt_optional(&format!(
        "Closed at [{}] (RFC3339/now/open): ",
        closed_label
    ))? {
        task.closed_at = parse_optional_datetime_input(&input, now, "closed at")?;
    }

    if task.segments.is_empty() {
        println!("No segments to edit.");
        return Ok(());
    }

    println!("Segments:");
    for (idx, segment) in task.segments.iter_mut().enumerate() {
        let start_label = format_datetime_local(segment.start_at);
        if let Some(input) = prompt_optional(&format!(
            "Segment {} start [{}] (RFC3339/now): ",
            idx + 1,
            start_label
        ))? {
            segment.start_at = parse_datetime_input(&input, now, "segment start")?;
        }

        let end_label = match segment.end_at {
            Some(end_at) => format_datetime_local(end_at),
            None => "open".to_string(),
        };
        if let Some(input) = prompt_optional(&format!(
            "Segment {} end [{}] (RFC3339/now/open): ",
            idx + 1,
            end_label
        ))? {
            segment.end_at = parse_optional_datetime_input(&input, now, "segment end")?;
        }
    }

    Ok(())
}

pub fn apply_task_edits(
    task: &mut Task,
    name: Option<String>,
    created_at: Option<String>,
    closed_at: Option<String>,
    segment_edits: Vec<String>,
    now: DateTime<Utc>,
) -> Result<(), String> {
    if let Some(name) = name {
        if name.trim().is_empty() {
            return Err("Task name cannot be empty.".into());
        }
        task.name = name;
    }

    if let Some(created_at) = created_at {
        task.created_at = parse_datetime_input(&created_at, now, "created at")?;
    }

    if let Some(closed_at) = closed_at {
        task.closed_at = parse_optional_datetime_input(&closed_at, now, "closed at")?;
    }

    for edit in segment_edits {
        let (index, start_at, end_at) = parse_segment_edit(&edit, now)?;
        if index == 0 || index > task.segments.len() {
            return Err(format!(
                "Segment index must be between 1 and {}.",
                task.segments.len()
            ));
        }
        let segment = &mut task.segments[index - 1];
        segment.start_at = start_at;
        segment.end_at = end_at;
    }

    Ok(())
}

fn parse_segment_edit(input: &str, now: DateTime<Utc>) -> Result<SegmentEdit, String> {
    let parts: Vec<&str> = input.splitn(3, ',').collect();
    if parts.len() != 3 {
        return Err("Segment edit must be in the form INDEX,START,END.".into());
    }
    let index: usize = parts[0]
        .parse()
        .map_err(|_| "Segment index must be a number.".to_string())?;
    let start_at = parse_datetime_input(parts[1], now, "segment start")?;
    let end_at = parse_optional_datetime_input(parts[2], now, "segment end")?;
    Ok((index, start_at, end_at))
}

fn parse_datetime_input(
    input: &str,
    now: DateTime<Utc>,
    label: &str,
) -> Result<DateTime<Utc>, String> {
    if input.eq_ignore_ascii_case("now") {
        return Ok(now);
    }
    DateTime::parse_from_rfc3339(input)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|err| format!("Invalid {} timestamp: {}", label, err))
}

fn parse_optional_datetime_input(
    input: &str,
    now: DateTime<Utc>,
    label: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    if input.eq_ignore_ascii_case("open") || input.eq_ignore_ascii_case("none") {
        return Ok(None);
    }
    parse_datetime_input(input, now, label).map(Some)
}
