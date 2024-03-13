#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateUserRequestBody {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CreateUserResponseBody {
    pub token: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoginUserRequestBody {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoginUserResponseBody {
    pub token: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WhoamiResponseBody {
    pub username: String,
}
