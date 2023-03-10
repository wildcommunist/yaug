use anyhow::Context;
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, Version};
use argon2::password_hash::SaltString;
use rand::Rng;
use secrecy::{ExposeSecret, Secret};
use crate::telemetry::spawn_blocking_with_tracing;

const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";

#[derive(Debug)]
pub struct AccountPassword(Secret<String>);

impl AsRef<Secret<String>> for AccountPassword {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

impl AccountPassword {
    pub fn parse(v: Secret<String>) -> Result<Self, String> {
        let pass = v.expose_secret();

        let empty_or_whitespace = pass.trim().is_empty();
        let correct_length = match pass.len() {
            8..=120 => true,
            _ => false
        };

        let required_characters = ['~', '`', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '-', '+', '=', '{', '[', '}', ']', '|', '\\', ':', ';', '"', '\'', '<', ',', '>', '.', '?', '/'];
        let contains_required_characters = pass.chars().any(|c| required_characters.contains(&c));

        if empty_or_whitespace || !correct_length || !contains_required_characters {
            return Err("Password does not meet minimum requirements".to_string());
        }

        Ok(AccountPassword(Secret::new(pass.to_string())))
    }

    pub fn new(length: u8) -> Self {
        AccountPassword(AccountPassword::generate_password(length))
    }

    pub fn generate_password(length: u8) -> Secret<String> {
        let mut rng = rand::thread_rng();
        let required_characters = ['~', '`', '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '_', '-', '+', '=', '{', '[', '}', ']', '|', '\\', ':', ';', '"', '\'', '<', ',', '>', '.', '?', '/'];
        let mut password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..PASSWORD_CHARSET.len());
                PASSWORD_CHARSET[idx] as char
            })
            .collect();
        if !password.chars().any(|c| required_characters.contains(&c)) {
            // we dont have any special symbols, replace the first one with a random special character
            let rand_special_char = rng.gen_range(0..required_characters.len());
            let rand_char = required_characters[rand_special_char];
            password.replace_range(0..1, &format!("{}", rand_char));
        }

        Secret::new(password)
    }

    pub async fn compute_hash(&self) -> Result<Secret<String>, anyhow::Error> {
        let secret = self.0.clone();
        let hash =
            spawn_blocking_with_tracing(move || compute_password_hash(secret))
                .await?
                .context("Failed to hash password")?;
        Ok(hash)
    }
}

fn compute_password_hash(password: Secret<String>) -> Result<Secret<String>, anyhow::Error> {
    let salt = SaltString::generate(&mut rand::thread_rng());
    let hash = Argon2::new(
        Algorithm::Argon2id,
        Version::V0x13,
        Params::new(15000, 2, 1, None).unwrap(),
    ).hash_password(password.expose_secret().as_bytes(), &salt)?
        .to_string();
    Ok(Secret::new(hash))
}

#[cfg(test)]
mod tests {
    use claim::assert_err;
    use quickcheck::Gen;
    use rand::Rng;
    use secrecy::{ExposeSecret, Secret};
    use crate::domain::AccountPassword;

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: Gen>(_g: &mut G) -> Self {
            Self(Secret::from(generate_valid_password()))
        }
    }

    fn generate_password(length: u8) -> String {
        AccountPassword::generate_password(length).expose_secret().to_string()
    }

    #[quickcheck_macros::quickcheck]
    fn valid_password_is_accepted(pass: ValidPasswordFixture) -> bool {
        AccountPassword::parse(pass.0).is_ok()
    }

    fn generate_valid_password() -> String {
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(8..120);
        generate_password(length)
    }

    #[test]
    fn short_password_length_is_rejected() {
        let password = Secret::new(generate_password(7));
        assert_err!(AccountPassword::parse(password));
    }

    #[test]
    fn long_password_length_is_rejected() {
        let password = Secret::new(generate_password(121));
        assert_err!(AccountPassword::parse(password));
    }

    #[test]
    fn password_is_rejected_if_no_special_symbols_are_present() {
        let password = Secret::new("12345678".to_string());
        assert_err!(AccountPassword::parse(password));
    }

    #[test]
    fn whitespace_are_rejected() {
        let password = Secret::new(" ".to_string());
        assert_err!(AccountPassword::parse(password));
    }
}