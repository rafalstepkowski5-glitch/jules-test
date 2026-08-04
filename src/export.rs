use crate::handlers::AuthUser;
use crate::AppState;
use axum::{extract::State, http::StatusCode};

pub async fn to_pdf(
    AuthUser(claims): AuthUser,
    State(_state): State<AppState>,
) -> Result<&'static str, StatusCode> {
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok("PDF export not implemented yet")
}

pub async fn to_md(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    let data = crate::db::get_history(&state.pool, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut md = String::from("# Metrics Export\n\n");
    for row in data {
        md.push_str(&format!(
            "* {}: CPU: {}, MEM: {}\n",
            row.ts, row.cpu, row.mem
        ));
    }
    Ok(md)
}

pub async fn to_txt(
    AuthUser(claims): AuthUser,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    if claims.role != "admin" {
        return Err(StatusCode::FORBIDDEN);
    }
    let data = crate::db::get_history(&state.pool, 100)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut txt = String::new();
    for row in data {
        txt.push_str(&format!(
            "ID: {}, TS: {}, CPU: {}, MEM: {}\n",
            row.id, row.ts, row.cpu, row.mem
        ));
    }
    Ok(txt)
}
