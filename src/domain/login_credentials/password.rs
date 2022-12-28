use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct LoginPassword(Secret<String>);

impl LoginPassword {
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

        Ok(LoginPassword(Secret::new(pass.to_string())))
    }
}

#[cfg(test)]
mod tests {
    use claim::assert_err;
    use quickcheck::Gen;
    use rand::Rng;
    use secrecy::Secret;
    use crate::domain::LoginPassword;

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: Gen>(_g: &mut G) -> Self {
            Self(Secret::from(generate_valid_password()))
        }
    }

    const PASSWORD_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";

    fn generate_password(length: u8) -> String {
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

        password
    }

    #[quickcheck_macros::quickcheck]
    fn valid_password_is_accepted(pass: ValidPasswordFixture) -> bool {
        LoginPassword::parse(pass.0).is_ok()
    }

    fn generate_valid_password() -> String {
        let mut rng = rand::thread_rng();
        let length = rng.gen_range(8..120);
        generate_password(length)
    }

    #[test]
    fn short_password_length_is_rejected() {
        let password = Secret::new(generate_password(7));
        assert_err!(LoginPassword::parse(password));
    }

    #[test]
    fn long_password_length_is_rejected() {
        let password = Secret::new(generate_password(121));
        assert_err!(LoginPassword::parse(password));
    }

    #[test]
    fn password_is_rejected_if_no_special_symbols_are_present() {
        let password = Secret::new("12345678".to_string());
        assert_err!(LoginPassword::parse(password));
    }
}