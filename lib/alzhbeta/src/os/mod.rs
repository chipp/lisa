use crate::{Event, MacAddr};
use tokio::sync::mpsc::Receiver;

pub trait CommonScanner {
    fn new() -> Self;
    fn start_scan(&mut self) -> Receiver<(MacAddr, Event)>;
}

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "macos")]
pub mod macos;
