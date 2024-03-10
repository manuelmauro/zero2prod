use askama::Template;
use axum::response::IntoResponse;

use crate::app::extractor::session_cookie::SessionCookie;

#[derive(Template)]
#[template(path = "index.html")]
struct AdminDashboardTemplate;

#[tracing::instrument(name = "Admin dashboard")]
pub async fn admin_dashboard(session: SessionCookie) -> impl IntoResponse {
    AdminDashboardTemplate
}
