pub mod email;
pub mod name;

use self::email::Email;
use self::name::Name;

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub name: Name,
    pub email: Email,
}
