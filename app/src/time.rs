use chrono::{DateTime, Local, Utc};

pub fn format_duration(seconds: i64) -> String {
    let total = seconds.max(0);
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let secs = total % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

pub fn format_datetime_local(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).to_rfc3339()
}

pub fn format_time_local_display(dt: DateTime<Utc>) -> String {
    dt.with_timezone(&Local).format("%H:%M:%S").to_string()
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
}
