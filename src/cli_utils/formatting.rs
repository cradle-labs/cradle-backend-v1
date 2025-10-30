use colored::Colorize;

/// Format a table with columns and rows
pub fn format_table(headers: Vec<&str>, rows: Vec<Vec<String>>) {
    let col_widths: Vec<usize> = headers
        .iter()
        .enumerate()
        .map(|(i, header)| {
            let mut width = header.len();
            for row in &rows {
                if i < row.len() {
                    width = width.max(row[i].len());
                }
            }
            width
        })
        .collect();

    // Print header
    let header_line = headers
        .iter()
        .enumerate()
        .map(|(i, h)| format!("{:width$}", h, width = col_widths[i]))
        .collect::<Vec<_>>()
        .join(" | ");

    println!("{}", header_line.bold());
    println!("{}", "-".repeat(header_line.len()));

    // Print rows
    for row in rows {
        let row_line = row
            .iter()
            .enumerate()
            .map(|(i, cell)| format!("{:width$}", cell, width = col_widths.get(i).copied().unwrap_or(20)))
            .collect::<Vec<_>>()
            .join(" | ");
        println!("{}", row_line);
    }
}

/// Format data as JSON
pub fn format_json<T: serde::Serialize>(data: &T) -> String {
    match serde_json::to_string_pretty(data) {
        Ok(json) => json,
        Err(_) => "Unable to format as JSON".to_string(),
    }
}

/// Format a single record as key-value pairs
pub fn format_record(data: Vec<(&str, String)>) {
    let max_key_len = data.iter().map(|(k, _)| k.len()).max().unwrap_or(20);

    for (key, value) in data {
        let padded_key = format!("{:width$}", key, width = max_key_len);
        println!("  {}: {}", padded_key.bright_cyan(), value);
    }
}

/// Format a list of items
pub fn format_list(items: Vec<String>) {
    for (i, item) in items.iter().enumerate() {
        println!("  {}. {}", i + 1, item);
    }
}

/// Format a header
pub fn print_header(text: &str) {
    println!();
    println!("{}", text.bold().bright_cyan());
    println!("{}", "=".repeat(text.len()));
    println!();
}

/// Format a section
pub fn print_section(text: &str) {
    println!();
    println!("{}", text.bold().bright_white());
    println!("{}", "-".repeat(text.len()));
}

/// Format UUID as short version
pub fn format_uuid_short(uuid: &uuid::Uuid) -> String {
    let s = uuid.to_string();
    format!("{}...", &s[..8])
}

/// Format datetime
pub fn format_datetime(dt: &chrono::NaiveDateTime) -> String {
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format BigDecimal with 2 decimal places
pub fn format_decimal(value: &bigdecimal::BigDecimal) -> String {
    let scale = 2;
    let rounded = value.with_scale_round(scale, bigdecimal::RoundingMode::HalfUp);
    rounded.to_string()
}

/// Format bool as Yes/No
pub fn format_bool(value: bool) -> String {
    if value {
        "Yes".green().to_string()
    } else {
        "No".red().to_string()
    }
}

/// Format status with color
pub fn format_status(status: &str) -> String {
    match status.to_lowercase().as_str() {
        "active" => status.green().to_string(),
        "inactive" => status.dimmed().to_string(),
        "pending" => status.yellow().to_string(),
        "cancelled" => status.red().to_string(),
        _ => status.white().to_string(),
    }
}

/// Format a count
pub fn format_count(label: &str, count: usize) -> String {
    format!("{}: {}", label, count.to_string().bright_cyan())
}

/// Format a key-value pair for display
pub fn format_kv(key: &str, value: &str) -> String {
    format!("{}: {}", key.bright_cyan(), value)
}
