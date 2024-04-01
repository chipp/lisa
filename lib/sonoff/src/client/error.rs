#[derive(Debug)]
pub enum Error {
    Disconnected,
    Dns(dns_parser::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    UrlParse(chipp_http::UrlParseError),
    HttpError(chipp_http::Error),
    UnknownDevice(String),
    MissingHostname,
    MissingService,
    MissingAddr,
    MissingInfo,
    MissingInfoField(&'static str),
}

impl From<dns_parser::Error> for Error {
    fn from(err: dns_parser::Error) -> Self {
        Self::Dns(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<chipp_http::UrlParseError> for Error {
    fn from(err: chipp_http::UrlParseError) -> Self {
        Self::UrlParse(err)
    }
}

impl From<chipp_http::Error> for Error {
    fn from(err: chipp_http::Error) -> Self {
        Self::HttpError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Dns(err) => write!(f, "DNS error: {err}"),
            Self::Json(err) => write!(f, "JSON error: {err}"),
            Self::Io(err) => write!(f, "IO error: {err}"),
            Self::UrlParse(err) => write!(f, "URL parse error: {err}"),
            Self::HttpError(err) => write!(f, "HTTP error: {err}"),
            Self::UnknownDevice(device) => write!(f, "Unknown device: {device}"),
            Self::MissingHostname => write!(f, "Missing hostname"),
            Self::MissingService => write!(f, "Missing service"),
            Self::MissingAddr => write!(f, "Missing ip address and port"),
            Self::MissingInfo => write!(f, "Missing info"),
            Self::MissingInfoField(field) => write!(f, "Missing info field: {field}"),
        }
    }
}

impl std::error::Error for Error {}
