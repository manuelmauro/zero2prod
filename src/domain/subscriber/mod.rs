pub mod email;
pub mod name;

pub struct NewSubscriber {
    pub name: name::Name,
    pub email: email::Email,
}
