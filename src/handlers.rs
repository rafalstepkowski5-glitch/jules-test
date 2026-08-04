use askama::Template;
use axum::{
    extract::{State, FromRequestParts},
    http::{StatusCode, header, HeaderMap, Response, request::Parts},
    response::{Html, IntoResponse},
    Json,
};
use crate::{AppState, auth, db, charts, security};

#[derive(Template)]
#[template(path = "login.html")]
struct LoginTemplate {}

#[derive(Template)]
#[template(path = "dashboard.html")]
struct DashboardTemplate {
    username: String,
    role: String,
    cpu_chart: String,
    mem_chart: String,
}

pub async fn login_page() -> impl IntoResponse {
    let template = LoginTemplate {};
    let csrf_token = security::generate_csrf_token();
    
    let html = template.render().unwrap();
    let mut response = Html(html).into_response();
    response.headers_mut().append(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&format!("csrf_token={}; Path=/; SameSite=Strict; HttpOnly", csrf_token)).unwrap(),
    );
    response
}

pub async fn dashboard(AuthUser(claims): AuthUser, State(state): State<AppState>) -> impl IntoResponse {
    let history = db::get_history(&state.pool, 50).await.unwrap_or_default();
    let cpu_vals: Vec<f64> = history.iter().map(|m| m.cpu).collect();
    let mem_vals: Vec<f64> = history.iter().map(|m| m.mem).collect();
    
    let cpu_chart = charts::svg_line_chart(&cpu_vals, "#3b82f6");
    let mem_chart = charts::svg_line_chart(&mem_vals, "#10b981");

    let template = DashboardTemplate {
        username: claims.sub,
        role: claims.role,
        cpu_chart,
        mem_chart,
    };
    Html(template.render().unwrap())
}

pub async fn login_api(
    State(state): State<AppState>,
    Json(payload): Json<auth::LoginRequest>,
) -> Result<impl IntoResponse, StatusCode> {
    let user = db::get_user(&state.pool, &payload.username).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if let Some(u) = user {
        if auth::verify_password(&u.password_hash, &payload.password) {
            let access = auth::create_access_token(&u.username, &u.role);
            let (refresh, _jti) = auth::create_refresh_token(&u.username);
            
            let _ = sqlx::query("INSERT INTO refresh_tokens (jti, username, expires_at) VALUES (?, ?, ?)")
                .bind(&_jti)
                .bind(&u.username)
                .bind(chrono::Utc::now().to_rfc3339())
                .execute(&state.pool)
                .await;

            let mut response = Response::builder().status(200).body(axum::body::Body::empty()).unwrap();
            let headers = response.headers_mut();
            headers.append(
                header::SET_COOKIE,
                header::HeaderValue::from_str(&format!("access_token={}; Path=/; SameSite=Strict; HttpOnly", access)).unwrap(),
            );
            headers.append(
                header::SET_COOKIE,
                header::HeaderValue::from_str(&format!("refresh_token={}; Path=/api/auth/refresh; SameSite=Strict; HttpOnly", refresh)).unwrap(),
            );
            return Ok(response);
        }
    }
    Err(StatusCode::UNAUTHORIZED)
}

pub async fn refresh_api(headers: HeaderMap, State(state): State<AppState>) -> Result<impl IntoResponse, StatusCode> {
    let token = extract_cookie(&headers, "refresh_token").ok_or(StatusCode::UNAUTHORIZED)?;
    let claims = auth::verify_jwt(&token).map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    let user = db::get_user(&state.pool, &claims.sub).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::UNAUTHORIZED)?;
    
    let access = auth::create_access_token(&user.username, &user.role);
    let mut response = Response::builder().status(200).body(axum::body::Body::empty()).unwrap();
    response.headers_mut().append(
        header::SET_COOKIE,
        header::HeaderValue::from_str(&format!("access_token={}; Path=/; SameSite=Strict; HttpOnly", access)).unwrap(),
    );
    Ok(response)
}

pub async fn logout_api() -> impl IntoResponse {
    let mut response = Response::builder().status(200).body(axum::body::Body::empty()).unwrap();
    let headers = response.headers_mut();
    headers.append(
        header::SET_COOKIE,
        header::HeaderValue::from_static("access_token=; Path=/; Expires=Thu, 01 Jan 1970 00:00:00 GMT"),
    );
    headers.append(
        header::SET_COOKIE,
        header::HeaderValue::from_static("refresh_token=; Path=/api/auth/refresh; Expires=Thu, 01 Jan 1970 00:00:00 GMT"),
    );
    response
}

pub async fn all_metrics_json(AuthUser(_claims): AuthUser, State(state): State<AppState>) -> Result<Json<Vec<db::MetricPoint>>, StatusCode> {
    db::get_history(&state.pool, 100).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn history_json(AuthUser(_claims): AuthUser, State(state): State<AppState>) -> Result<Json<Vec<db::MetricPoint>>, StatusCode> {
    db::get_history(&state.pool, 100).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub fn extract_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers.get(header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|c| {
            c.split(';')
                .find_map(|p| {
                    let p = p.trim();
                    if p.starts_with(&(name.to_owned() + "=")) {
                        Some(p[name.len()+1..].to_string())
                    } else {
                        None
                    }
                })
        })
}

pub struct AuthUser(pub auth::Claims);

#[axum::async_trait]
impl<S: Send + Sync> FromRequestParts<S> for AuthUser {
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let token = extract_cookie(&parts.headers, "access_token").ok_or(StatusCode::UNAUTHORIZED)?;
        match auth::verify_jwt(&token) {
            Ok(claims) => Ok(AuthUser(claims)),
            Err(_) => Err(StatusCode::UNAUTHORIZED),
        }
    }
}

pub async fn protected_metrics(headers: HeaderMap) -> Result<String, StatusCode> {
    let expected = std::env::var("PROM_TOKEN").map_err(|_| {
        tracing::error!("Krytyczny błąd: Zmienna środowiskowa PROM_TOKEN nie została ustawiona!");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let auth_header = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()).unwrap_or("");
    if auth_header != format!("Bearer {}", expected) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    use prometheus::Encoder;
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    Ok(String::from_utf8(buffer).unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header, HeaderMap};
    use tower::util::ServiceExt;
    use std::env;
    use uuid::Uuid;

    async fn build_test_state() -> AppState {
        env::set_var("JWT_SECRET", "abcdefghijklmnopqrstuvwxyz012345");
        let temp_path = std::env::temp_dir().join(format!("webx_test_{}.db", Uuid::new_v4()));
        let url = format!("sqlite://{}?mode=rwc", temp_path.display());
        let pool = crate::db::init_db_with_url(&url, "admin-pass").await.expect("create test db");
        AppState { pool }
    }

    fn find_set_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
        headers.get_all(header::SET_COOKIE).iter().find_map(|value| {
            let value = value.to_str().ok()?;
            value.split(';').find_map(|segment| {
                let segment = segment.trim();
                if segment.starts_with(&(name.to_owned() + "=")) {
                    Some(segment[name.len() + 1..].to_string())
                } else {
                    None
                }
            })
        })
    }

    #[tokio::test]
    async fn login_api_sets_access_and_refresh_tokens() {
        let state = build_test_state().await;
        let app = axum::Router::new()
            .route("/api/auth/login", axum::routing::post(login_api))
            .with_state(state);

        let body = serde_json::json!({"username":"admin","password":"admin-pass"}).to_string();
        let request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let headers = response.headers();
        assert!(find_set_cookie(headers, "access_token").is_some(), "access_token cookie missing");
        assert!(find_set_cookie(headers, "refresh_token").is_some(), "refresh_token cookie missing");
    }

    #[tokio::test]
    async fn refresh_api_returns_new_access_token() {
        let state = build_test_state().await;
        let app = axum::Router::new()
            .route("/api/auth/login", axum::routing::post(login_api))
            .route("/api/auth/refresh", axum::routing::post(refresh_api))
            .with_state(state.clone());

        let login_body = serde_json::json!({"username":"admin","password":"admin-pass"}).to_string();
        let login_request = Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(login_body))
            .unwrap();

        let login_response = app.clone().oneshot(login_request).await.expect("login response");
        assert_eq!(login_response.status(), StatusCode::OK);
        let refresh_token = find_set_cookie(login_response.headers(), "refresh_token").expect("refresh token cookie");

        let refresh_request = Request::builder()
            .method("POST")
            .uri("/api/auth/refresh")
            .header(header::COOKIE, format!("refresh_token={}", refresh_token))
            .body(Body::empty())
            .unwrap();

        let refresh_response = app.oneshot(refresh_request).await.expect("refresh response");
        assert_eq!(refresh_response.status(), StatusCode::OK);
        let new_access = find_set_cookie(refresh_response.headers(), "access_token").expect("new access token cookie");
        assert!(crate::auth::verify_jwt(&new_access).is_ok(), "new access token should be valid");
    }
}
