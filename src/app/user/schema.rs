#[derive(serde::Deserialize, serde::Serialize)]
pub struct NewUser {
    pub username: String,
    pub password: String,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct User {
    pub token: String,
    pub username: String,
}
