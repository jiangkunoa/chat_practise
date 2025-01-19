use anyhow::{Context, Result};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher, SaltString
    },
    Argon2, PasswordHash, PasswordVerifier
};



pub fn password_hash(password: &str) -> Result<String> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!(e))
        .context("密码哈希失败")?
        .to_string();
    Ok(password_hash)
}

pub fn password_verify(password: &str, password_hash: &str) -> Result<bool> {
    let password_hash = PasswordHash::new(password_hash)
        .map_err(|e| anyhow::anyhow!(e))
        .context("密码哈希解析失败")?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &password_hash).is_ok())
}



#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_hash() -> anyhow::Result<()> {
        let password = "123456";
        let password_hash = password_hash(password)?;
        println!("password_hash: {}", password_hash);
        let is_valid = password_verify("123456", &password_hash)?;
        assert!(is_valid);
        println!("is_valid: {}", is_valid);
        Ok(())
    }
}