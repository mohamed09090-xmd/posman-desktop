use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use zeroize::Zeroizing;

use crate::phase05::error::{Phase05Error, Phase05Result};

pub struct SessionSecret {
    pub raw: Zeroizing<Vec<u8>>,
    pub hash: String,
}

pub fn generate_session_secret() -> SessionSecret {
    let mut raw = Zeroizing::new(vec![0_u8; 32]);
    OsRng.fill_bytes(raw.as_mut_slice());
    let hash = sha256_hex(raw.as_slice());
    SessionSecret { raw, hash }
}

pub fn generate_recovery_code() -> String {
    let mut bytes = [0_u8; 16];
    OsRng.fill_bytes(&mut bytes);
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut compact = String::with_capacity(39);
    for (index, byte) in bytes.iter().enumerate() {
        if index > 0 && index % 2 == 0 {
            compact.push('-');
        }
        compact.push(char::from(HEX[usize::from(byte >> 4)]));
        compact.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    compact
}

pub fn recovery_code_hash(code: &str) -> Phase05Result<String> {
    let normalized = code
        .chars()
        .filter(|character| !character.is_whitespace() && *character != '-')
        .flat_map(char::to_uppercase)
        .collect::<String>();
    if normalized.len() != 32
        || !normalized
            .chars()
            .all(|character| character.is_ascii_hexdigit())
    {
        return Err(Phase05Error::new(
            "RECOVERY_CODE_INVALID",
            "The recovery code is invalid or has already been used.",
        ));
    }
    Ok(sha256_hex(normalized.as_bytes()))
}

pub fn constant_time_hex_equal(left: &str, right: &str) -> bool {
    left.len() == right.len() && left.as_bytes().ct_eq(right.as_bytes()).into()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_session_and_recovery_secrets_have_required_entropy() {
        let session = generate_session_secret();
        assert_eq!(session.raw.len(), 32);
        assert_eq!(session.hash.len(), 64);
        let recovery = generate_recovery_code();
        assert_eq!(recovery.replace('-', "").len(), 32);
        assert_eq!(recovery_code_hash(&recovery).expect("hash").len(), 64);
    }

    #[test]
    fn recovery_hash_comparison_is_constant_work_for_equal_lengths() {
        assert!(constant_time_hex_equal(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ));
        assert!(!constant_time_hex_equal(
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
        ));
    }
}
