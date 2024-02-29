use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequestBody {
    username: String,
    password: String,
}
