use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequestBody {
    pub username: String,
    pub password: String,
}
