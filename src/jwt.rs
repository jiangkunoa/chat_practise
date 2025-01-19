use std::time::Duration;

use anyhow::Result;
use serde::{Serialize, Deserialize};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use sqlx::types::chrono::Local;

const SECRET_KEY: &[u8] = b"chat_jwt_secret";


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: u64,
    pub exp: usize,
}
pub fn build_token(id: u64) -> Result<String> {
    let exp = Local::now() + Duration::from_secs(60 * 60 * 24 * 14);
    let my_claims = Claims {
        sub: id,
        exp: exp.timestamp() as usize,
    };
    Ok(encode(&Header::default(), &my_claims, &EncodingKey::from_secret(SECRET_KEY.as_ref()))?)
}

pub fn validate_jwt(token: &str) -> Result<Claims> {
    let validation = Validation::default();
    let token_data = decode::<Claims>(token, &DecodingKey::from_secret(SECRET_KEY), &validation)?;
    if token_data.claims.exp < Local::now().timestamp() as usize {
        return Err(anyhow::anyhow!("token expired"));
    }
    Ok(token_data.claims)
}