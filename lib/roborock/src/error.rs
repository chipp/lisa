use std::fmt;

#[derive(Debug)]
pub enum Error {
    Decode(DecodeError),
    Encode(EncodeError),
    Crypto(aes_gcm::Error),
    CryptoKeyLength,
    Io(std::io::Error),
    Json(serde_json::Error),
    Timeout(tokio::time::error::Elapsed),
    Rpc(RpcError),
    ConnectionClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    FrameTooShort,
    PayloadLengthMismatch,
    PayloadCrcMissing,
    CrcMismatch,
    PayloadLengthMissing,
    UnknownProtocol,
    MissingDps,
    MissingResponse,
    UnknownVersion,
    MissingAckNonce,
    GcmDecryptFailed(aes_gcm::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncodeError {
    GcmEncryptFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RpcError {
    UnknownMethod,
    UnexpectedResult,
    InvalidResultType,
    MissingResult,
    DeviceError,
}

impl From<DecodeError> for Error {
    fn from(err: DecodeError) -> Self {
        Self::Decode(err)
    }
}

impl From<EncodeError> for Error {
    fn from(err: EncodeError) -> Self {
        Self::Encode(err)
    }
}

impl From<RpcError> for Error {
    fn from(err: RpcError) -> Self {
        Self::Rpc(err)
    }
}

impl From<aes_gcm::Error> for Error {
    fn from(err: aes_gcm::Error) -> Self {
        Self::Crypto(err)
    }
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
            Self::Decode(err) => write!(f, "decode error: {err}"),
            Self::Encode(err) => write!(f, "encode error: {err}"),
            Self::Crypto(err) => write!(f, "crypto error: {err}"),
            Self::CryptoKeyLength => write!(f, "crypto key length error"),
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::Json(err) => write!(f, "json error: {err}"),
            Self::Timeout(err) => write!(f, "timeout error: {err}"),
            Self::Rpc(err) => write!(f, "rpc error: {err}"),
            Self::ConnectionClosed => write!(f, "connection closed"),
        }
    }
}

impl std::error::Error for Error {}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameTooShort => write!(f, "frame too short"),
            Self::PayloadLengthMismatch => write!(f, "payload length mismatch"),
            Self::PayloadCrcMissing => write!(f, "payload crc missing"),
            Self::CrcMismatch => write!(f, "crc mismatch"),
            Self::PayloadLengthMissing => write!(f, "payload length missing"),
            Self::UnknownProtocol => write!(f, "unknown protocol"),
            Self::MissingDps => write!(f, "missing dps"),
            Self::MissingResponse => write!(f, "missing response"),
            Self::UnknownVersion => write!(f, "unknown version"),
            Self::MissingAckNonce => write!(f, "missing ack nonce"),
            Self::GcmDecryptFailed(err) => write!(f, "gcm decrypt failed: {err}"),
        }
    }
}

impl std::error::Error for DecodeError {}

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GcmEncryptFailed => write!(f, "gcm encrypt failed"),
        }
    }
}

impl std::error::Error for EncodeError {}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownMethod => write!(f, "unknown method"),
            Self::UnexpectedResult => write!(f, "unexpected result"),
            Self::InvalidResultType => write!(f, "invalid result type"),
            Self::MissingResult => write!(f, "missing result"),
            Self::DeviceError => write!(f, "device error"),
        }
    }
}

impl std::error::Error for RpcError {}
