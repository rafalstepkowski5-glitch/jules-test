use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use std::net::SocketAddr;
use tower_http::{
    compression::CompressionLayer, cors::CorsLayer, limit::RequestBodyLimitLayer,
    services::ServeDir, trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod charts;
mod db;
mod export;
mod handlers;
mod metrics;
mod security;

#[derive(Clone)]
pub struct AppState {
    pub pool: db::DbPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer().json())
        .init();

    // Early fail-closed security checks for critical environment variables
    let jwt_secret = std::env::var("JWT_SECRET").map_err(|_| {
        anyhow::anyhow!("Krytyczny błąd: Zmienna środowiskowa JWT_SECRET nie została ustawiona!")
    })?;
    if jwt_secret.trim().len() < 32 {
        anyhow::bail!("Zagrożenie bezpieczeństwa: JWT_SECRET musi mieć co najmniej 32 znaki");
    }
    // Require an explicit admin password in environment for initial user creation
    let admin_pass = std::env::var("ADMIN_PASS")
        .map_err(|_| anyhow::anyhow!("Krytyczny błąd: Zmienna środowiskowa ADMIN_PASS nie została ustawiona! Ustaw silne hasło administracyjne."))?;
    if admin_pass.trim().len() < 8 {
        anyhow::bail!("Zagrożenie bezpieczeństwa: ADMIN_PASS musi mieć co najmniej 8 znaków");
    }
    let _prom_token = std::env::var("PROM_TOKEN").map_err(|_| {
        anyhow::anyhow!("Krytyczny błąd: Zmienna środowiskowa PROM_TOKEN nie została ustawiona!")
    })?;

    let pool = db::init_db().await?;
    let (prom_layer, _metrics_handle) = PrometheusMetricLayer::pair();
    metrics::init_metrics(pool.clone());

    let allowed = std::env::var("ALLOWED_ORIGINS").unwrap_or_default();
    let cors = if allowed.is_empty() {
        CorsLayer::new().allow_methods([])
    } else {
        let origin = allowed.parse::<axum::http::HeaderValue>()?;
        CorsLayer::new().allow_origin(origin)
    };

    let state = AppState { pool };

    let app = Router::new()
        .route("/", get(handlers::dashboard))
        .route("/login", get(handlers::login_page))
        .route("/api/auth/login", post(handlers::login_api))
        .route("/api/auth/refresh", post(handlers::refresh_api))
        .route("/api/auth/logout", post(handlers::logout_api))
        .route("/api/metrics/all", get(handlers::all_metrics_json))
        .route("/api/metrics/history", get(handlers::history_json))
        .route("/api/export/pdf", post(export::to_pdf))
        .route("/api/export/md", post(export::to_md))
        .route("/api/export/txt", post(export::to_txt))
        .route("/metrics", get(handlers::protected_metrics))
        .nest_service("/static", ServeDir::new("static")) // JS tutaj
        .layer(RequestBodyLimitLayer::new(1 * 1024 * 1024)) // 1MB limit
        .layer(middleware::from_fn(security::security_headers_middleware))
        .layer(middleware::from_fn(security::csrf_middleware))
        .layer(middleware::from_fn(security::global_rate_limit_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(CompressionLayer::new())
        .layer(cors)
        .layer(prom_layer)
        .with_state(state);

    println!("WebX start: http://localhost:3000 -> /login admin/admin123");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
