use std::fmt;

/// Unified error type for Vanta operations.
#[derive(Debug)]
pub enum VantaError {
    Config(String),
    Scanner(String),
    Matcher(String),
    Window(String),
    Script(String),
    Io(std::io::Error),
}

impl fmt::Display for VantaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VantaError::Config(msg) => write!(f, "Config error: {}", msg),
            VantaError::Scanner(msg) => write!(f, "Scanner error: {}", msg),
            VantaError::Matcher(msg) => write!(f, "Matcher error: {}", msg),
            VantaError::Window(msg) => write!(f, "Window error: {}", msg),
            VantaError::Script(msg) => write!(f, "Script error: {}", msg),
            VantaError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}

impl std::error::Error for VantaError {}

impl From<std::io::Error> for VantaError {
    fn from(e: std::io::Error) -> Self {
        VantaError::Io(e)
    }
}
