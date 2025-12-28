use std::fmt;
use std::net::Ipv4Addr;

use crate::vacuum::BinType;

#[derive(Debug)]
pub enum Error {
    DevicesNotFound(Ipv4Addr),
    InvalidChecksum,
    DeviceResponse(i16),
    UnsupportedBinType(BinType),
    Io(std::io::Error),
    Json(serde_json::Error),
    Timeout(tokio::time::error::Elapsed),
    CryptoEncrypt(cipher::inout::PadError),
    CryptoDecrypt(cipher::block_padding::UnpadError),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<tokio::time::error::Elapsed> for Error {
    fn from(err: tokio::time::error::Elapsed) -> Self {
        Self::Timeout(err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DevicesNotFound(ip) => {
                if ip.is_broadcast() {
                    write!(f, "devices not found")
                } else {
                    write!(f, "device {ip} not found")
                }
            }
            Self::InvalidChecksum => write!(f, "invalid data checksum"),
            Self::DeviceResponse(code) => write!(f, "device error code {code}"),
            Self::UnsupportedBinType(bin_type) => {
                write!(f, "unsupported bin type for start: {bin_type}")
            }
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Timeout(err) => write!(f, "timeout error: {err}"),
            Self::CryptoEncrypt(err) => write!(f, "crypto encrypt error: {err}"),
            Self::CryptoDecrypt(err) => write!(f, "crypto decrypt error: {err}"),
        }
    }
}

impl std::error::Error for Error {}
