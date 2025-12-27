use crate::{Event, MacAddr};
use std::io;
use tokio::sync::mpsc::Receiver;

#[derive(Debug)]
pub struct Error {
    context: &'static str,
    source: io::Error,
}

impl Error {
    pub fn new(context: &'static str, source: io::Error) -> Self {
        Self { context, source }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait ScannerTrait {
    fn new() -> Result<Self>
    where
        Self: Sized;
    fn start_scan(&mut self) -> Result<Receiver<(MacAddr, Event)>>;
}

#[cfg(test)]
mod tests {
    use super::Error;
    use std::io;

    #[test]
    fn error_includes_context_and_source() {
        let err = Error::new("scan failed", io::Error::new(io::ErrorKind::Other, "boom"));
        let message = err.to_string();

        assert!(message.contains("scan failed"));
        assert!(message.contains("boom"));
    }
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;
