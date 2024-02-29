use axum::Router;

use super::AppState;

mod asset;
mod home;
mod login;

pub fn router() -> Router<AppState> {
    home::router().merge(login::router()).merge(asset::router())
}
