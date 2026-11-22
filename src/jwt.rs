use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use jsonwebtoken::{Algorithm, DecodingKey, crypto};

use crate::routes::message_routes::ApiError;

const TOKEN_PREFIX: &str = "eyJhbGciOiJIUzI1NiJ9.";

pub fn decode_string_jwt(token: &str, decode_key: &DecodingKey) -> Result<String, ApiError> {
    let invalid = || ApiError::unauthorized("Invalid Token");

    let token_with_prefix = format!("{TOKEN_PREFIX}{token}");

    let mut parts = token_with_prefix.split('.');
    let (Some(header), Some(payload), Some(signature), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(invalid());
    };

    let message = format!("{header}.{payload}");
    let valid = crypto::verify(signature, message.as_bytes(), decode_key, Algorithm::HS256)
        .map_err(|_| invalid())?;

    if !valid {
        return Err(invalid());
    }

    let bytes = URL_SAFE_NO_PAD.decode(payload).map_err(|_| invalid())?;
    String::from_utf8(bytes).map_err(|_| invalid())
}
