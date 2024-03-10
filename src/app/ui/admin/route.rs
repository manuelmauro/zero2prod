use askama::Template;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
};

use crate::app::extractor::session_cookie::SessionCookie;

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
struct AdminDashboardTemplate;

#[tracing::instrument(name = "Admin dashboard", skip(cookie))]
pub async fn admin_dashboard(cookie: Option<SessionCookie>) -> impl IntoResponse {
    if let Some(_session) = cookie {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(AdminDashboardTemplate.render().unwrap()))
            .unwrap()
    } else {
        Redirect::temporary("/login").into_response()
    }
}
