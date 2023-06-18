use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher,
        SaltString,
        PasswordHash
    },
    Argon2, PasswordVerifier
};


pub fn hash_password(password: &String) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    return Ok(argon2.hash_password(password.as_bytes(), &salt)?.to_string());
}

pub fn verify_password(password: &String, hash: &String) -> Result<bool, argon2::password_hash::Error> {
    let hash = PasswordHash::new(&hash)?;
    
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
} 