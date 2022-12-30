use std::fmt::Formatter;
use lazy_static::lazy_static;
use regex::Regex;
use validator::validate_email;

#[derive(Debug)]
pub struct AccountEmail(String);

impl AccountEmail {
    pub fn parse(v: String) -> Result<Self, String> {
        if validate_email(&v) {
            lazy_static! {
                static ref RE:Regex = Regex::new(
                    r#"^.*@.*\..*$"# // dont allow domains without extension
                ).unwrap();
            }

            if RE.is_match(&v) {
                Ok(AccountEmail(v))
            } else {
                Err(format!("{} is not a valid email address", v))
            }
        } else {
            Err(format!("{} is not a valid email address", v))
        }
    }
}

impl AsRef<str> for AccountEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccountEmail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use claim::{assert_err, assert_ok};
    use fake::Fake;
    use quickcheck::Gen;
    use fake::faker::internet::en::SafeEmail;
    use crate::domain::account_credentials::email::AccountEmail;

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    // This is used so we can run same test many times using the #[quickcheck_macros::quickcheck]
    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            let email = SafeEmail().fake_with_rng(g);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(email: ValidEmailFixture) -> bool {
        AccountEmail::parse(email.0).is_ok()
    }

    #[test]
    fn valid_long_domain_emails_are_parsed_successfully() {
        let email = format!("user@active.few.sub.domain");
        assert_ok!(AccountEmail::parse(email));
    }

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert_err!(AccountEmail::parse(email));
    }

    #[test]
    fn missing_at_symbol_rejected() {
        let email = "userdomain.com".to_string();
        assert_err!(AccountEmail::parse(email));
    }

    #[test]
    fn missing_subject_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(AccountEmail::parse(email));
    }

    #[test]
    fn missing_domain_rejected() {
        let email = "user@".to_string();
        assert_err!(AccountEmail::parse(email));
    }

    #[test]
    fn invalid_domain_rejected() {
        let email = "user@domain".to_string();
        assert_err!(AccountEmail::parse(email));
    }
}