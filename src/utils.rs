use sha2::{Digest, Sha512};

pub fn to_sha512(input: impl AsRef<[u8]> + Clone) -> String {
    let mut hasher = Sha512::new();
    hasher.update(input.clone());
    hex::encode(hasher.finalize())
}
