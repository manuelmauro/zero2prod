use derive_more::Display;
use serde::Deserialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Display)]
#[display(fmt = "{}", _0)]
pub struct SubscriberName(String);

impl TryFrom<String> for SubscriberName {
    type Error = String;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            return Err("name is empty".into());
        }

        if value.graphemes(true).count() > 256 {
            return Err("name is too long".into());
        }

        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        if value.chars().any(|g| forbidden_characters.contains(&g)) {
            return Err("name contains invalid characters".into());
        }

        Ok(Self(value))
    }
}

impl<'de> Deserialize<'de> for SubscriberName {
    fn deserialize<D>(deserializer: D) -> Result<SubscriberName, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        SubscriberName::try_from(s).map_err(serde::de::Error::custom)
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;

    #[test]
    fn a_256_grapheme_long_name_is_valid() {
        let name = "a̐".repeat(256);
        assert!(SubscriberName::try_from(name).is_ok());
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_rejected() {
        let name = "a".repeat(257);
        assert!(SubscriberName::try_from(name).is_err());
    }

    #[test]
    fn whitespace_only_names_are_rejected() {
        let name = " ".to_string();
        assert!(SubscriberName::try_from(name).is_err());
    }

    #[test]
    fn empty_string_is_rejected() {
        let name = "".to_string();
        assert!(SubscriberName::try_from(name).is_err());
    }

    #[test]
    fn names_containing_an_invalid_character_are_rejected() {
        for name in &['/', '(', ')', '"', '<', '>', '\\', '{', '}'] {
            let name = name.to_string();
            assert!(SubscriberName::try_from(name).is_err());
        }
    }

    #[test]
    fn a_valid_name_is_parsed_successfully() {
        let name = "Kurt Gödel".to_string();
        assert!(SubscriberName::try_from(name).is_ok());
    }
}
