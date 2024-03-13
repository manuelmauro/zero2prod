use askama::Template;
use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::{IntoResponse, Redirect},
};

use crate::app::extractor::session_user::SessionUser;

#[derive(Template)]
#[template(path = "admin_dashboard.html")]
struct AdminDashboardTemplate {
    user: String,
}

#[tracing::instrument(name = "Admin dashboard", skip(session))]
pub async fn admin_dashboard(session: Option<SessionUser>) -> impl IntoResponse {
    if let Some(user) = session {
        Response::builder()
            .status(StatusCode::OK)
            .body(Body::from(
                AdminDashboardTemplate { user: user.id }.render().unwrap(),
            ))
            .unwrap()
    } else {
        Redirect::temporary("/login").into_response()
    }
}
