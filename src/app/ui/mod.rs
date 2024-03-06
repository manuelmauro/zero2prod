use axum::Router;

use super::AppState;

mod admin;
mod asset;
mod login;

pub fn router() -> Router<AppState> {
    admin::router()
        .merge(login::router())
        .merge(asset::router())
}
