use axum::Router;

use super::AppState;

mod admin;
mod asset;
mod home;
mod login;
pub mod not_found;

pub fn router() -> Router<AppState> {
    home::router()
        .merge(admin::router())
        .merge(login::router())
        .merge(asset::router())
}
