//! Utility functions

use leptos::*;
use std::fmt::Write;

pub fn format_bytes(bytes: i64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;
    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }
    format!("{:.1} {}", size, UNITS[unit_index])
}

pub fn format_date(date: chrono::DateTime<chrono::Utc>) -> String {
    date.format("%Y-%m-%d %H:%M").to_string()
}

pub fn get_file_icon(mime_type: &str, is_folder: bool) -> &'static str {
    if is_folder { "folder" }
    else if mime_type.starts_with("image/") { "image" }
    else if mime_type.starts_with("video/") { "video" }
    else if mime_type.starts_with("audio/") { "audio" }
    else if mime_type == "application/pdf" { "pdf" }
    else { "file" }
}
