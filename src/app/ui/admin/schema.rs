use secrecy::Secret;

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ChangePasswordRequestBody {
    pub current_password: Secret<String>,
    pub new_password: Secret<String>,
    pub new_password_check: Secret<String>,
}
