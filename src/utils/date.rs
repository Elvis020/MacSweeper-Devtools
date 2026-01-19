// Date formatting utilities
use chrono::{DateTime, Utc, Local};

pub fn format_datetime(dt: &DateTime<Utc>) -> String {
    let local: DateTime<Local> = DateTime::from(*dt);
    local.format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn days_since(dt: &DateTime<Utc>) -> u32 {
    let now = Utc::now();
    let duration = now.signed_duration_since(*dt);
    duration.num_days() as u32
}

pub fn format_days_ago(days: u32) -> String {
    if days == 0 {
        "Today".to_string()
    } else if days == 1 {
        "Yesterday".to_string()
    } else if days < 7 {
        format!("{} days ago", days)
    } else if days < 30 {
        format!("{} weeks ago", days / 7)
    } else if days < 365 {
        format!("{} months ago", days / 30)
    } else {
        format!("{} years ago", days / 365)
    }
}
