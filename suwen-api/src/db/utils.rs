use sha2::{Digest, Sha256};

pub(crate) fn sha256_hash(input: &str) -> String {
    format!("{:x}", Sha256::digest(input.as_bytes()))
}
