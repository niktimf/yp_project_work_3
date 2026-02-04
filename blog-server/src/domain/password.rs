use std::{fmt, ops::Deref};

use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
        rand_core::OsRng,
    },
};

#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    // OWASP recommended parameters (2023)
    // https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html
    const ARGON2_MEMORY_KIB: u32 = 65536; // 64 MiB
    const ARGON2_ITERATIONS: u32 = 3;
    const ARGON2_LANES: u32 = 4;
    const ARGON2_OUTPUT_LEN: usize = 32;

    pub fn hash<S: AsRef<str>>(input: S) -> Result<Self, password_hash::Error> {
        let params = Params::new(
            Self::ARGON2_MEMORY_KIB,
            Self::ARGON2_ITERATIONS,
            Self::ARGON2_LANES,
            Some(Self::ARGON2_OUTPUT_LEN),
        )?;

        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let salt = SaltString::generate(&mut OsRng);

        let hash = argon2.hash_password(input.as_ref().as_bytes(), &salt)?;
        Ok(Self(hash.to_string()))
    }

    pub fn verify(&self, password: &str) -> bool {
        PasswordHash::new(&self.0)
            .map(|hash| {
                Argon2::default()
                    .verify_password(password.as_bytes(), &hash)
                    .is_ok()
            })
            .unwrap_or(false)
    }

    pub const fn from_hash(hash: String) -> Self {
        Self(hash)
    }
}

impl Deref for Password {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Password {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password(\"********\")")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hash_and_verify() {
        let password = Password::hash("secret123").unwrap();
        assert!(password.verify("secret123"));
        assert!(!password.verify("wrong_password"));
    }

    #[test]
    fn test_password_debug_hides_hash() {
        let password = Password::hash("secret123").unwrap();
        let debug_output = format!("{:?}", password);
        assert_eq!(debug_output, "Password(\"********\")");
    }
}
