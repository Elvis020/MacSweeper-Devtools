// Output formatting for different formats (table, JSON, CSV)
use comfy_table::{Table, Cell, Color, Attribute};

pub fn create_table() -> Table {
    let mut table = Table::new();
    table.load_preset(comfy_table::presets::UTF8_FULL);
    table
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn format_days_ago(days: u32) -> String {
    if days == 0 {
        "Today".to_string()
    } else if days == 1 {
        "Yesterday".to_string()
    } else {
        format!("{} days ago", days)
    }
}
