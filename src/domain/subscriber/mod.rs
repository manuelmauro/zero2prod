pub mod email;
pub mod name;

pub use name::Name;

#[derive(serde::Deserialize)]
pub struct NewSubscriber {
    pub name: Name,
    pub email: String,
}
