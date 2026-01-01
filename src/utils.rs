use sha2::{Digest, Sha256, Sha512};

pub fn to_sha512(input: impl AsRef<[u8]> + Clone) -> String {
    let mut hasher = Sha512::new();
    hasher.update(input.clone());
    hex::encode(hasher.finalize())
}

pub fn to_sha256(input: impl AsRef<[u8]> + Clone) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.clone());
    hex::encode(hasher.finalize())
}

pub fn hash_bcrypt(password: &str, cost: u32) -> Result<String, bcrypt::BcryptError> {
    bcrypt::hash(password, cost)
}

pub fn verify_bcrypt(password: &str, hash: &str) -> Result<bool, bcrypt::BcryptError> {
    bcrypt::verify(password, hash)
}
