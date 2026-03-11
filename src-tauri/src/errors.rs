use serde::ser::SerializeStruct;
use thiserror::Error;

/// Unified error type for all Vanta operations.
///
/// Each variant carries a human-readable message and maps to a numeric error
/// code that the frontend can match on programmatically.
#[derive(Debug, Error)]
pub enum VantaError {
    // ── System ──────────────────────────────────────────
    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Json(#[from] serde_json::Error),

    // ── Domain ──────────────────────────────────────────
    #[error("{0}")]
    Config(String),

    #[error("{0}")]
    Scanner(String),

    #[error("{0}")]
    Matcher(String),

    #[error("{0}")]
    Launcher(String),

    #[error("{0}")]
    Window(String),

    #[error("{0}")]
    Script(String),

    #[error("{0}")]
    Extension(String),

    #[error("{0}")]
    Store(String),

    #[error("{0}")]
    Workflow(String),

    #[error("{0}")]
    Clipboard(String),

    #[error("{0}")]
    Permission(String),

    #[error("{0}")]
    Community(String),

    #[error("{0}")]
    Theme(String),

    // ── Catch-all for gradual migration ─────────────────
    #[error("{0}")]
    Other(String),
}

impl VantaError {
    /// Numeric error code for frontend error-type discrimination.
    pub fn code(&self) -> u32 {
        match self {
            Self::Io(_) => 1000,
            Self::Json(_) => 1001,
            Self::Config(_) => 2000,
            Self::Scanner(_) => 2001,
            Self::Matcher(_) => 2002,
            Self::Launcher(_) => 2003,
            Self::Window(_) => 3000,
            Self::Script(_) => 3001,
            Self::Workflow(_) => 3002,
            Self::Extension(_) => 4000,
            Self::Store(_) => 4001,
            Self::Clipboard(_) => 5000,
            Self::Permission(_) => 6000,
            Self::Community(_) => 7000,
            Self::Theme(_) => 8000,
            Self::Other(_) => 9999,
        }
    }

    /// Machine-readable error kind tag.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Io(_) => "io",
            Self::Json(_) => "json",
            Self::Config(_) => "config",
            Self::Scanner(_) => "scanner",
            Self::Matcher(_) => "matcher",
            Self::Launcher(_) => "launcher",
            Self::Window(_) => "window",
            Self::Script(_) => "script",
            Self::Workflow(_) => "workflow",
            Self::Extension(_) => "extension",
            Self::Store(_) => "store",
            Self::Clipboard(_) => "clipboard",
            Self::Permission(_) => "permission",
            Self::Community(_) => "community",
            Self::Theme(_) => "theme",
            Self::Other(_) => "other",
        }
    }
}

/// Serialize as `{ "code": N, "kind": "...", "message": "..." }` so the
/// Tauri IPC layer delivers structured errors to the frontend.
impl serde::Serialize for VantaError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut s = serializer.serialize_struct("VantaError", 3)?;
        s.serialize_field("code", &self.code())?;
        s.serialize_field("kind", self.kind())?;
        s.serialize_field("message", &self.to_string())?;
        s.end()
    }
}

/// Enables `?` on functions that still return `Result<T, String>`.
impl From<String> for VantaError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

/// Enables `Err("literal")?` and `return Err("literal".into())`.
impl From<&str> for VantaError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_owned())
    }
}
