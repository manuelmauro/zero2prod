use serde::Deserialize;

#[derive(Deserialize)]
pub struct LoginRequestBody {
    email: String,
    password: String,
}
