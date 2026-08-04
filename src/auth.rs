use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier, password_hash::{SaltString, rand_core::OsRng}};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use uuid::Uuid;

static SECRET: Lazy<String> = Lazy::new(|| {
    let s = std::env::var("JWT_SECRET").expect("Krytyczny błąd: Zmienna środowiskowa JWT_SECRET nie została ustawiona!");
    assert!(s.len()>=32, "Zagrożenie bezpieczeństwa: JWT_SECRET musi mieć co najmniej 32 znaki!");
    s
});

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims { pub sub: String, pub role: String, pub exp: usize, pub iat: usize, pub jti: String, pub iss: String, pub aud: String }

#[derive(Deserialize)] pub struct LoginRequest { pub username: String, pub password: String }

pub fn hash_password(p: &str) -> anyhow::Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default().hash_password(p.as_bytes(), &salt).map_err(|e| anyhow::anyhow!(e.to_string()))?.to_string())
}
pub fn verify_password(h: &str, p: &str) -> bool {
    PasswordHash::new(h).map(|ph| Argon2::default().verify_password(p.as_bytes(), &ph).is_ok()).unwrap_or(false)
}
pub fn create_access_token(u: &str, role: &str) -> String {
    let now = chrono::Utc::now();
    let c = Claims{ sub: u.into(), role: role.into(), iat: now.timestamp() as usize, exp: (now+chrono::Duration::minutes(15)).timestamp() as usize, jti: Uuid::new_v4().to_string(), iss: "webx-metrics-pro".into(), aud: "webx-client".into() };
    encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET.as_bytes())).unwrap()
}
pub fn create_refresh_token(u: &str) -> (String, String) {
    let jti = Uuid::new_v4().to_string(); let now = chrono::Utc::now();
    let c = Claims{ sub: u.into(), role: "refresh".into(), iat: now.timestamp() as usize, exp: (now+chrono::Duration::days(7)).timestamp() as usize, jti: jti.clone(), iss: "webx-metrics-pro".into(), aud: "webx-client".into() };
    (encode(&Header::default(), &c, &EncodingKey::from_secret(SECRET.as_bytes())).unwrap(), jti)
}
pub fn verify_jwt(t: &str) -> Result<Claims, String> {
    let mut v = Validation::default(); v.set_issuer(&["webx-metrics-pro"]); v.set_audience(&["webx-client"]);
    decode::<Claims>(t, &DecodingKey::from_secret(SECRET.as_bytes()), &v).map(|d| d.claims).map_err(|e| e.to_string())
}
