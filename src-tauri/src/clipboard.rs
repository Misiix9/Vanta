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

    Ok(())
}

fn save_item(content: &str) -> Result<()> {
    let path = get_db_path();
    let conn = Connection::open(path)?;

    // Check if content is already the most recent item to avoid duplicates
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
    conn.execute(
        "INSERT INTO clipboard (content, timestamp) VALUES (?1, ?2)",
        params![content, now],
    )?;

    // Keep only last 50 items
    conn.execute(
        "DELETE FROM clipboard WHERE id NOT IN (SELECT id FROM clipboard ORDER BY id DESC LIMIT 50)",
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
                let content = String::from_utf8_lossy(&output.stdout).to_string();

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

    let mut stmt = conn.prepare("SELECT id, content, timestamp FROM clipboard ORDER BY id DESC")?;
    let rows = stmt.query_map([], |row| {
        Ok(ClipboardItem {
            id: row.get(0)?,
            content: row.get(1)?,
            timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    })?;

    let mut history = Vec::new();
    for row in rows {
        history.push(row?);
    }

    Ok(history)
}

// Ensure Rusqlite's optional trait is imported
use rusqlite::OptionalExtension;
