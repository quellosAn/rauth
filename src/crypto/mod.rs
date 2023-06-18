use std::{path::Path, io::{self, BufReader}, fs::File};
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHasher,
        SaltString,
        PasswordHash
    },
    Argon2, PasswordVerifier
};
use tokio_rustls::rustls::{Certificate, PrivateKey};
use rustls_pemfile::{certs, pkcs8_private_keys};

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

pub fn load_cert(path: &Path) -> io::Result<Vec<Certificate>> {
    certs(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid cert"))
        .map(|mut certs| certs.drain(..).map(Certificate).collect())
}

pub fn load_key(path: &Path) -> io::Result<Vec<PrivateKey>> {
    pkcs8_private_keys(&mut BufReader::new(File::open(path)?))
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid key"))
        .map(|mut keys| keys.drain(..).map(PrivateKey).collect())
}