use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Algorithm, Argon2, Params, Version,
};
use rand_core::OsRng;

use crate::phase05::error::{Phase05Error, Phase05Result};

pub const PASSWORD_MIN_CHARACTERS: usize = 10;
pub const PASSWORD_MAX_CHARACTERS: usize = 128;
pub const PRODUCTION_MEMORY_KIB: u32 = 19_456;
pub const PRODUCTION_ITERATIONS: u32 = 2;
pub const PRODUCTION_PARALLELISM: u32 = 1;

#[derive(Clone)]
pub struct PasswordEngine {
    memory_kib: u32,
    iterations: u32,
    parallelism: u32,
}

impl PasswordEngine {
    pub fn production() -> Self {
        Self {
            memory_kib: PRODUCTION_MEMORY_KIB,
            iterations: PRODUCTION_ITERATIONS,
            parallelism: PRODUCTION_PARALLELISM,
        }
    }

    #[cfg(test)]
    pub fn test() -> Self {
        Self {
            memory_kib: 32,
            iterations: 1,
            parallelism: 1,
        }
    }

    pub fn runtime() -> Self {
        #[cfg(test)]
        {
            Self::test()
        }
        #[cfg(not(test))]
        {
            Self::production()
        }
    }

    fn argon2(&self) -> Phase05Result<Argon2<'static>> {
        let params = Params::new(self.memory_kib, self.iterations, self.parallelism, Some(32))
            .map_err(|_| Phase05Error::internal())?;
        Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
    }

    pub fn validate(password: &str) -> Phase05Result<()> {
        let characters = password.chars().count();
        if !(PASSWORD_MIN_CHARACTERS..=PASSWORD_MAX_CHARACTERS).contains(&characters) {
            return Err(Phase05Error::new(
                "PASSWORD_POLICY_VIOLATION",
                "The password must contain 10 to 128 characters.",
            ));
        }
        Ok(())
    }

    pub fn hash(&self, password: &str) -> Phase05Result<String> {
        Self::validate(password)?;
        let salt = SaltString::generate(&mut OsRng);
        self.argon2()?
            .hash_password(password.as_bytes(), &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| Phase05Error::internal())
    }

    pub fn dummy_hash(&self) -> Phase05Result<String> {
        let salt = SaltString::generate(&mut OsRng);
        self.argon2()?
            .hash_password(b"POSMAN dummy credential", &salt)
            .map(|hash| hash.to_string())
            .map_err(|_| Phase05Error::internal())
    }

    pub fn verify(&self, candidate: &str, phc: &str) -> bool {
        PasswordHash::new(phc).ok().is_some_and(|parsed| {
            self.argon2().ok().is_some_and(|argon2| {
                argon2
                    .verify_password(candidate.as_bytes(), &parsed)
                    .is_ok()
            })
        })
    }

    #[cfg(test)]
    fn parameters(&self) -> (u32, u32, u32) {
        (self.memory_kib, self.iterations, self.parallelism)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_parameters_and_unicode_are_supported() {
        let engine = PasswordEngine::production();
        assert_eq!(engine.parameters(), (19_456, 2, 1));
        let value = "كلمة مرور محلية 123";
        let hash = engine.hash(value).expect("hash password");
        assert!(hash.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"));
        assert!(engine.verify(value, &hash));
        assert!(!engine.verify("mot de passe incorrect", &hash));
    }

    #[test]
    fn password_length_uses_unicode_characters() {
        assert!(PasswordEngine::validate("كلمةمرور12").is_ok());
        assert_eq!(
            PasswordEngine::validate("قصيرة")
                .expect_err("too short")
                .code,
            "PASSWORD_POLICY_VIOLATION"
        );
    }
}
