use askama::Template;
use axum::response::IntoResponse;

use crate::app::extractor::AuthUser;

#[derive(Template)]
#[template(path = "index.html")]
struct AdminDashboardTemplate;

pub async fn admin_dashboard(_user: AuthUser) -> impl IntoResponse {
    AdminDashboardTemplate
}
