use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Store {
    pub version: u32,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub segments: Vec<Segment>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Segment {
    pub start_at: DateTime<Utc>,
    pub end_at: Option<DateTime<Utc>>,
}

#[derive(Debug)]
pub struct ReportEntry {
    pub name: String,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
    pub seconds: i64,
}

pub type SegmentEdit = (usize, DateTime<Utc>, Option<DateTime<Utc>>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Active,
    Paused,
}
