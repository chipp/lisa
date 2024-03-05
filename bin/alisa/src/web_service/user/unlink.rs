use axum::{http::HeaderMap, response::Result};

use crate::web_service::auth::validate_autorization;

pub async fn unlink(headers: HeaderMap) -> Result<()> {
    validate_autorization(&headers, "unlink")?;
    Ok(())
}
