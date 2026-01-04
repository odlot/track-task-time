use chrono::{DateTime, Duration, Local, TimeZone, Utc};

use crate::model::{ReportEntry, Segment, Store};

pub fn report_today(store: &Store, now: DateTime<Utc>) -> Vec<ReportEntry> {
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

    let mut entries = Vec::new();

    for task in &store.tasks {
        let mut seconds = 0i64;
        let mut earliest: Option<DateTime<Utc>> = None;
        let mut latest: Option<DateTime<Utc>> = None;

        for segment in &task.segments {
            let Some((start, end)) = overlap_window(segment, start_utc, end_utc, now) else {
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

        let Some(start_at) = earliest else {
            continue;
        };
        let Some(end_at) = latest else {
            continue;
        };

        entries.push(ReportEntry {
            name: task.name.clone(),
            start_at,
            end_at,
            seconds,
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

pub fn overlap_window(
    segment: &Segment,
    window_start: DateTime<Utc>,
    window_end: DateTime<Utc>,
    now: DateTime<Utc>,
) -> Option<(DateTime<Utc>, DateTime<Utc>)> {
    let segment_end = segment.end_at.unwrap_or(now);
    if segment_end <= window_start || segment.start_at >= window_end {
        return None;
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
    if end <= start {
        return None;
    }
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn overlap_window_handles_window_edges() {
        let seg_start = Utc.with_ymd_and_hms(2025, 1, 1, 9, 0, 0).unwrap();
        let seg_end = Utc.with_ymd_and_hms(2025, 1, 1, 10, 0, 0).unwrap();
        let window_start = Utc.with_ymd_and_hms(2025, 1, 1, 9, 30, 0).unwrap();
        let window_end = Utc.with_ymd_and_hms(2025, 1, 1, 10, 30, 0).unwrap();
        let segment = Segment {
            start_at: seg_start,
            end_at: Some(seg_end),
        };

        let result = overlap_window(&segment, window_start, window_end, window_end).unwrap();
        assert_eq!(result.0, window_start);
        assert_eq!(result.1, seg_end);
    }
}
