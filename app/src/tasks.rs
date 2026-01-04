use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::model::{Segment, Store, Task, TaskState};

pub fn current_task_state(store: &Store) -> Option<(usize, TaskState)> {
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

pub fn active_task_name(store: &Store) -> Option<String> {
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

pub fn start_task(store: &mut Store, name: String, now: DateTime<Utc>) {
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

pub fn stop_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    if let Some(segment) = task.segments.iter_mut().find(|seg| seg.end_at.is_none()) {
        segment.end_at = Some(now);
    }
    task.closed_at = Some(now);
}

pub fn pause_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    if let Some(segment) = task.segments.iter_mut().find(|seg| seg.end_at.is_none()) {
        segment.end_at = Some(now);
    }
}

pub fn resume_task(store: &mut Store, idx: usize, now: DateTime<Utc>) {
    let task = &mut store.tasks[idx];
    task.segments.push(Segment {
        start_at: now,
        end_at: None,
    });
}

pub fn total_elapsed(task: &Task, now: DateTime<Utc>) -> i64 {
    task.segments
        .iter()
        .map(|seg| segment_duration(seg, now))
        .sum()
}

pub fn task_status(task: &Task) -> &'static str {
    if task.segments.iter().any(|seg| seg.end_at.is_none()) {
        "active"
    } else if task.closed_at.is_none() && !task.segments.is_empty() {
        "paused"
    } else {
        "stopped"
    }
}

fn segment_duration(segment: &Segment, now: DateTime<Utc>) -> i64 {
    let end = segment.end_at.unwrap_or(now);
    let duration = end - segment.start_at;
    duration.num_seconds().max(0)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

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
}
