mod data;
mod event;
mod os;

pub use event::{Event, MacAddr};
pub use os::ScannerTrait;

#[cfg(target_os = "linux")]
pub use os::linux::Scanner;

#[cfg(target_os = "macos")]
pub use os::macos::Scanner;
