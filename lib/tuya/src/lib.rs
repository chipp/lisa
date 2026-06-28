mod client;
mod discovery;
mod error;
mod profile;
mod protocol;

pub use client::{Client, DeviceConfig};
pub use discovery::{resolve_devices, DiscoveredDevice, ResolvedDevice};
pub use error::{Error, Result};
pub use profile::{dps_from_action, state_from_dps};
