use chrono::{DateTime, Datelike, Duration, Local, TimeZone, Utc};

use crate::model::{Segment, Store};
use crate::report::overlap_window;
use crate::tasks::task_status;

pub struct TaskListEntry {
    pub name: String,
    pub id: String,
    pub status: &'static str,
    pub seconds: i64,
    pub start_at: Option<DateTime<Utc>>,
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ListWindow {
    All,
    Today,
    Week,
}

pub fn list_tasks(store: &Store, now: DateTime<Utc>, window: ListWindow) -> Vec<TaskListEntry> {
    let bounds = window_bounds(now, window);
    let mut entries = Vec::new();

    for task in &store.tasks {
        let mut seconds = 0i64;
        let mut earliest: Option<DateTime<Utc>> = None;
        let mut latest: Option<DateTime<Utc>> = None;

        for segment in &task.segments {
            let Some((start, end)) = segment_bounds(segment, bounds, now) else {
                continue;
            };
            let duration = (end - start).num_seconds().max(0);
            if duration == 0 {
                continue;
            }
            seconds += duration;
            earliest = Some(match earliest {
                Some(value) => value.min(start),
                None => start,
            });
            latest = Some(match latest {
                Some(value) => value.max(end),
                None => end,
            });
        }

        if seconds == 0 {
            continue;
        }

        entries.push(TaskListEntry {
            name: task.name.clone(),
            id: task.id.clone(),
            status: task_status(task),
            seconds,
            start_at: earliest,
            end_at: latest,
        });
    }

    entries.sort_by(|a, b| {
        b.end_at
            .cmp(&a.end_at)
            .then_with(|| b.start_at.cmp(&a.start_at))
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    entries
}

pub fn list_header(now: DateTime<Utc>, window: ListWindow) -> Option<String> {
    match window {
        ListWindow::All => None,
        ListWindow::Today => Some(now.with_timezone(&Local).date_naive().to_string()),
        ListWindow::Week => {
            let (start, end) = week_bounds(now);
            let end_date = end - Duration::days(1);
            Some(format!(
                "Week {} to {}",
                start.date_naive(),
                end_date.date_naive()
            ))
        }
    }
}

fn window_bounds(now: DateTime<Utc>, window: ListWindow) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    match window {
        ListWindow::All => None,
        ListWindow::Today => Some(today_bounds(now)),
        ListWindow::Week => Some(week_bounds(now)),
    }
}

fn segment_bounds(
    segment: &Segment,
    bounds: Option<(DateTime<Utc>, DateTime<Utc>)>,
    now: DateTime<Utc>,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    match bounds {
        Some((start, end)) => overlap_window(segment, start, end, now),
        None => {
            let end_at = segment.end_at.unwrap_or(now);
            if end_at <= segment.start_at {
                None
            } else {
                Some((segment.start_at, end_at))
            }
        }
    }
}

fn today_bounds(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
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

    (start_utc, end_utc)
}

fn week_bounds(now: DateTime<Utc>) -> (DateTime<Utc>, DateTime<Utc>) {
    let now_local = now.with_timezone(&Local);
    let date = now_local.date_naive();
    let weekday = date.weekday().num_days_from_monday() as i64;
    let start_date = date - Duration::days(weekday);
    let end_date = start_date + Duration::days(7);

    let start_local = start_date.and_hms_opt(0, 0, 0).unwrap();
    let end_local = end_date.and_hms_opt(0, 0, 0).unwrap();

    let start_utc = Local
        .from_local_datetime(&start_local)
        .unwrap()
        .with_timezone(&Utc);
    let end_utc = Local
        .from_local_datetime(&end_local)
        .unwrap()
        .with_timezone(&Utc);

    (start_utc, end_utc)
}
