use std::fmt;

use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenType {
    Code,
    Access,
    Refresh,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.serialize(f)
    }
}

#[derive(Debug)]
pub enum TokenError {
    InvalidExpiration,
    Encoding(jsonwebtoken::errors::Error),
}

impl fmt::Display for TokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidExpiration => write!(f, "invalid token expiration"),
            Self::Encoding(err) => write!(f, "token encoding failed: {err}"),
        }
    }
}

impl std::error::Error for TokenError {}

pub fn is_valid_token<T: AsRef<str>>(token: T, token_type: TokenType) -> bool {
    let secret = extract_secret_from_env();
    is_valid_token_with_secret(token, token_type, &secret)
}

fn is_valid_token_with_secret<T: AsRef<str>>(
    token: T,
    token_type: TokenType,
    secret: &str,
) -> bool {
    is_valid_token_with_secret_at(token, token_type, secret, current_timestamp())
}

fn is_valid_token_with_secret_at<T: AsRef<str>>(
    token: T,
    token_type: TokenType,
    secret: &str,
    now_timestamp: u64,
) -> bool {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let mut validation = Validation::new(Algorithm::HS512);
    // We validate `exp` ourselves to make tests deterministic with an injected clock.
    validation.validate_exp = false;
    validation.leeway = 0;
    validation.sub = Some("yandex".to_owned());
    validation.set_audience(&[token_type.to_string()]);

    let decoded = match decode::<Claims>(
        token.as_ref(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    ) {
        Ok(decoded) => decoded,
        Err(err) => {
            log::debug!("token decoding failed: {}", err);
            return false;
        }
    };

    decoded.claims.exp > now_timestamp
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: u64,
    aud: Vec<String>,
}

pub fn create_token_with_expiration_in(
    expiration: Duration,
    token_type: TokenType,
) -> Result<String, TokenError> {
    let secret = extract_secret_from_env();
    create_token_with_expiration_in_with_secret(expiration, token_type, &secret)
}

fn create_token_with_expiration_in_with_secret(
    expiration: Duration,
    token_type: TokenType,
    secret: &str,
) -> Result<String, TokenError> {
    create_token_with_expiration_in_with_secret_at(
        expiration,
        token_type,
        secret,
        current_timestamp() as i64,
    )
}

fn create_token_with_expiration_in_with_secret_at(
    expiration: Duration,
    token_type: TokenType,
    secret: &str,
    now_timestamp: i64,
) -> Result<String, TokenError> {
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let expiration = now_timestamp
        .checked_add(expiration.num_seconds())
        .ok_or(TokenError::InvalidExpiration)?;
    let expiration = u64::try_from(expiration).map_err(|_| TokenError::InvalidExpiration)?;

    let claims = Claims {
        sub: "yandex".to_owned(),
        exp: expiration,
        aud: vec![token_type.to_string()],
    };

    let header = Header::new(Algorithm::HS512);

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(TokenError::Encoding)
}

fn extract_secret_from_env() -> String {
    std::env::var("JWT_SECRET").expect("Set JWT_SECRET env variable")
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::{
        create_token_with_expiration_in_with_secret_at, is_valid_token_with_secret_at, TokenType,
    };
    use chrono::Duration;

    const SECRET: &str = "test-secret";
    const NOW: i64 = 1_700_000_000;

    #[test]
    fn access_token_roundtrip_is_valid() {
        let token = create_token_with_expiration_in_with_secret_at(
            Duration::minutes(1),
            TokenType::Access,
            SECRET,
            NOW,
        )
        .unwrap();

        assert!(is_valid_token_with_secret_at(
            token,
            TokenType::Access,
            SECRET,
            NOW as u64
        ));
    }

    #[test]
    fn token_type_mismatch_is_invalid() {
        let token = create_token_with_expiration_in_with_secret_at(
            Duration::minutes(1),
            TokenType::Access,
            SECRET,
            NOW,
        )
        .unwrap();

        assert!(!is_valid_token_with_secret_at(
            token,
            TokenType::Refresh,
            SECRET,
            NOW as u64
        ));
    }

    #[test]
    fn expired_token_is_invalid() {
        let token = create_token_with_expiration_in_with_secret_at(
            Duration::minutes(-1),
            TokenType::Access,
            SECRET,
            NOW,
        )
        .unwrap();

        assert!(!is_valid_token_with_secret_at(
            token,
            TokenType::Access,
            SECRET,
            NOW as u64
        ));
    }

    #[test]
    fn token_signed_with_different_secret_is_invalid() {
        let token = create_token_with_expiration_in_with_secret_at(
            Duration::minutes(1),
            TokenType::Access,
            "secret-a",
            NOW,
        )
        .unwrap();

        assert!(!is_valid_token_with_secret_at(
            token,
            TokenType::Access,
            "secret-b",
            NOW as u64
        ));
    }

    #[test]
    fn token_expires_exactly_at_expected_time() {
        let token = create_token_with_expiration_in_with_secret_at(
            Duration::seconds(30),
            TokenType::Access,
            SECRET,
            NOW,
        )
        .unwrap();

        assert!(is_valid_token_with_secret_at(
            &token,
            TokenType::Access,
            SECRET,
            (NOW + 29) as u64
        ));
        assert!(!is_valid_token_with_secret_at(
            token,
            TokenType::Access,
            SECRET,
            (NOW + 30) as u64
        ));
    }

    #[test]
    fn invalid_expiration_returns_error() {
        let result = create_token_with_expiration_in_with_secret_at(
            Duration::seconds(1),
            TokenType::Access,
            SECRET,
            i64::MAX,
        );

        assert!(matches!(result, Err(super::TokenError::InvalidExpiration)));
    }
}
