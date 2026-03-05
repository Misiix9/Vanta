use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
// use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct ClipboardItem {
    pub id: i64,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub pinned: bool,
    pub content_type: String,
}

fn get_db_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    path.push("vanta");
    let _ = fs::create_dir_all(&path);
    path.push("clipboard.db");
    path
}

pub fn init_db() -> Result<()> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard (
            id INTEGER PRIMARY KEY,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL
        )",
        [],
    )?;

    conn.execute_batch(
        "ALTER TABLE clipboard ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0;",
    ).ok();

    conn.execute_batch(
        "ALTER TABLE clipboard ADD COLUMN content_type TEXT NOT NULL DEFAULT 'text';",
    ).ok();

    Ok(())
}

fn detect_content_type(content: &str) -> &'static str {
    let trimmed = content.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        return "url";
    }
    if trimmed.contains('@') && !trimmed.contains(' ') && trimmed.contains('.') {
        return "email";
    }
    if trimmed.starts_with('#') && trimmed.len() >= 4 && trimmed.len() <= 9
        && trimmed[1..].chars().all(|c| c.is_ascii_hexdigit())
    {
        return "color";
    }
    if (trimmed.starts_with("rgb(") || trimmed.starts_with("rgba(") || trimmed.starts_with("hsl("))
        && trimmed.ends_with(')')
    {
        return "color";
    }
    if trimmed.starts_with('/') || trimmed.starts_with("~/") || trimmed.starts_with("./") {
        return "path";
    }
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if serde_json::from_str::<serde_json::Value>(trimmed).is_ok() {
            return "json";
        }
    }
    if trimmed.lines().count() > 3
        && (trimmed.contains("fn ") || trimmed.contains("function ")
            || trimmed.contains("def ") || trimmed.contains("class ")
            || trimmed.contains("const ") || trimmed.contains("let ")
            || trimmed.contains("import ") || trimmed.contains("pub "))
    {
        return "code";
    }
    "text"
}

fn save_item(content: &str) -> Result<()> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare("SELECT content FROM clipboard ORDER BY id DESC LIMIT 1")?;
    let exists = stmt
        .query_row([], |row| row.get::<_, String>(0))
        .optional()?;

    if let Some(last_content) = exists {
        if last_content == content {
            return Ok(());
        }
    }

    let now = Utc::now().to_rfc3339();
    let ct = detect_content_type(content);
    conn.execute(
        "INSERT INTO clipboard (content, timestamp, pinned, content_type) VALUES (?1, ?2, 0, ?3)",
        params![content, now, ct],
    )?;

    conn.execute(
        "DELETE FROM clipboard WHERE pinned = 0 AND id NOT IN (SELECT id FROM clipboard ORDER BY pinned DESC, id DESC LIMIT 100)",
        [],
    )?;

    Ok(())
}

pub fn start_watcher() {
    thread::spawn(|| {
        println!("Clipboard watcher started");
        let mut last_content = String::new();

        loop {
            // Run wl-paste once (request text/plain to avoid binary/images)
            if let Ok(output) = Command::new("wl-paste")
                .args(["--type", "text/plain"])
                .output()
            {
                // Truncate output to prevent massive memory/DB spikes (e.g. 100,000 chars limit)
                let raw_content = String::from_utf8_lossy(&output.stdout);
                let content = if raw_content.len() > 100_000 {
                    format!("{}... [Truncated by Vanta]", &raw_content[..100_000])
                } else {
                    raw_content.to_string()
                };

                if !content.is_empty() && content != last_content {
                    println!(
                        "Clipboard changed: {}",
                        content.chars().take(20).collect::<String>()
                    );
                    if let Err(e) = save_item(&content) {
                        eprintln!("Failed to save clipboard: {}", e);
                    }
                    last_content = content;
                }
            } else {
                eprintln!("Failed to run wl-paste");
            }

            thread::sleep(Duration::from_millis(1000));
        }
    });
}

pub fn get_history() -> Result<Vec<ClipboardItem>> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    let mut stmt = conn.prepare(
        "SELECT id, content, timestamp, pinned, content_type FROM clipboard ORDER BY pinned DESC, id DESC"
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            pinned: row.get::<_, i64>(3).unwrap_or(0) != 0,
            content_type: row.get::<_, String>(4).unwrap_or_else(|_| "text".to_string()),
        })
    })?;

    let mut history = Vec::new();
    for row in rows {
        history.push(row?);
    }

    Ok(history)
}

pub fn delete_item(id: i64) -> Result<()> {
    let path = get_db_path();
    let conn = Connection::open(path)?;
    conn.execute("DELETE FROM clipboard WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn toggle_pin(id: i64) -> Result<bool> {
    let path = get_db_path();
    let conn = Connection::open(path)?;
    let current: i64 = conn.query_row(
        "SELECT pinned FROM clipboard WHERE id = ?1",
        params![id],
        |row| row.get(0),
    )?;
    let new_val = if current == 0 { 1 } else { 0 };
    conn.execute(
        "UPDATE clipboard SET pinned = ?1 WHERE id = ?2",
        params![new_val, id],
    )?;
    Ok(new_val != 0)
}

use rusqlite::OptionalExtension;
