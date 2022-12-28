use lazy_static::lazy_static;
use regex::Regex;
use secrecy::{ExposeSecret, Secret};

#[derive(Debug)]
pub struct LoginPassword(Secret<String>);

impl LoginPassword {
    pub fn parse(v: Secret<String>) -> Result<Self, String> {
        if v.expose_secret().len() < 8 || v.expose_secret().len() > 120 {
            return Err(format!("Password does not meet minimum criteria"));
        }

        Ok(LoginPassword(Secret::new(v.expose_secret().to_string())))
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
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.gen_range(0..PASSWORD_CHARSET.len());
                PASSWORD_CHARSET[idx] as char
            })
            .collect();
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
}