use crate::domain::subscriber::{email::Email, name::Name, NewSubscriber};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SubscribeBody {
    pub email: String,
    pub name: String,
}

impl TryFrom<SubscribeBody> for NewSubscriber {
    type Error = String;
    fn try_from(value: SubscribeBody) -> Result<Self, Self::Error> {
        let name = Name::try_from(value.name)?;
        let email = Email::try_from(value.email)?;
        Ok(Self { name, email })
    }
}

#[derive(Debug, Deserialize)]
pub struct ConfirmParams {
    subscription_token: String,
}
