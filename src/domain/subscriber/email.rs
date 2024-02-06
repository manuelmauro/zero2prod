use derive_more::Display;
use validator::validate_email;

#[derive(Display)]
#[display(fmt = "{}", _0)]
pub struct Email(String);

impl TryFrom<String> for Email {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if validate_email(&value) {
            Ok(Self(value))
        } else {
            Err("invalid email".into())
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck::{Arbitrary, Gen};

    use super::Email;

    #[test]
    fn empty_string_is_rejected() {
        let email = "".to_string();
        assert!(Email::try_from(email).is_err());
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert!(Email::try_from(email).is_err());
    }

    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert!(Email::try_from(email).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmail(pub String);

    impl Arbitrary for ValidEmail {
        fn arbitrary(_g: &mut Gen) -> Self {
            // TODO use fake_with_rng
            let email = SafeEmail().fake();
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmail) -> bool {
        dbg!(&valid_email.0);
        Email::try_from(valid_email.0).is_ok()
    }
}
