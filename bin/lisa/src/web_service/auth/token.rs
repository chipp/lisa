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

pub fn is_valid_token<T: AsRef<str>>(token: T, token_type: TokenType) -> bool {
    use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

    let mut validation = Validation::new(Algorithm::HS512);
    validation.sub = Some("yandex".to_owned());
    validation.set_audience(&[token_type.to_string()]);

    let secret = extract_secret_from_env();
    let decoded = decode::<Claims>(
        token.as_ref(),
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    );

    decoded.is_ok()
}

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: String,
    exp: usize,
    aud: Vec<String>,
}

pub fn create_token_with_expiration_in(expiration: Duration, token_type: TokenType) -> String {
    use chrono::Utc;
    use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};

    let expiration = Utc::now()
        .checked_add_signed(expiration)
        .expect("valid timestamp")
        .timestamp();

    let claims = Claims {
        sub: "yandex".to_owned(),
        exp: expiration as usize,
        aud: vec![token_type.to_string()],
    };

    let header = Header::new(Algorithm::HS512);
    let secret = extract_secret_from_env();

    encode(
        &header,
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .unwrap()
}

fn extract_secret_from_env() -> String {
    std::env::var("JWT_SECRET").expect("Set JWT_SECRET env variable")
}
