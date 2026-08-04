use sqlx::{SqlitePool, Row}; use chrono::{DateTime, Utc};

#[derive(Debug, Clone, serde::Serialize)]
pub struct MetricPoint { pub id: i64, pub ts: DateTime<Utc>, pub cpu: f64, pub mem: f64, pub users: f64, pub rps: f64 }

#[derive(Debug, Clone)]
pub struct UserRow { pub username: String, pub password_hash: String, pub role: String }

pub type DbPool = SqlitePool;

pub async fn init_db_with_url(url: &str, admin_pass: &str) -> anyhow::Result<DbPool> {
    let pool = SqlitePool::connect(url).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS metrics (id INTEGER PRIMARY KEY AUTOINCREMENT, ts TEXT NOT NULL, cpu REAL, mem REAL, users REAL, rps REAL)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT NOT NULL)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS refresh_tokens (jti TEXT PRIMARY KEY, username TEXT NOT NULL, revoked BOOLEAN NOT NULL DEFAULT 0, expires_at TEXT NOT NULL)").execute(&pool).await?;
    let cnt: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;
    if cnt==0 {
        let hash = crate::auth::hash_password(admin_pass)?;
        sqlx::query("INSERT INTO users (username,password_hash,role) VALUES (?,?,?)").bind("admin").bind(hash).bind("admin").execute(&pool).await?;
    }
    Ok(pool)
}

pub async fn init_db() -> anyhow::Result<DbPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data.db?mode=rwc".into());
    let pass = std::env::var("ADMIN_PASS").map_err(|_| anyhow::anyhow!("ADMIN_PASS must be set for initial admin creation"))?;
    init_db_with_url(&url, &pass).await
}

pub async fn save_metric(pool: &DbPool, cpu: f64, mem: f64, users: f64, rps: f64) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO metrics (ts,cpu,mem,users,rps) VALUES (?,?,?,?,?)").bind(Utc::now().to_rfc3339()).bind(cpu).bind(mem).bind(users).bind(rps).execute(pool).await?;
    sqlx::query("DELETE FROM metrics WHERE id NOT IN (SELECT id FROM metrics ORDER BY id DESC LIMIT 10000)").execute(pool).await?;
    Ok(())
}

pub async fn get_history(pool: &DbPool, lim: i64) -> anyhow::Result<Vec<MetricPoint>> {
    let rows = sqlx::query("SELECT id,ts,cpu,mem,users,rps FROM metrics ORDER BY id DESC LIMIT ?").bind(lim).fetch_all(pool).await?;
    let mut v = Vec::with_capacity(rows.len());
    for row in rows {
        let id: i64 = row.try_get("id")?;
        let ts_str: String = row.try_get("ts")?;
        let ts = DateTime::parse_from_rfc3339(&ts_str)?.with_timezone(&Utc);
        let cpu: f64 = row.try_get("cpu")?;
        let mem: f64 = row.try_get("mem")?;
        let users: f64 = row.try_get("users")?;
        let rps: f64 = row.try_get("rps")?;
        v.push(MetricPoint { id, ts, cpu, mem, users, rps });
    }
    v.reverse();
    Ok(v)
}

pub async fn get_user(pool: &DbPool, u: &str) -> anyhow::Result<Option<UserRow>> {
    if let Some(row) = sqlx::query("SELECT username,password_hash,role FROM users WHERE username=?").bind(u).fetch_optional(pool).await? {
        let username: String = row.try_get("username")?;
        let password_hash: String = row.try_get("password_hash")?;
        let role: String = row.try_get("role")?;
        Ok(Some(UserRow { username, password_hash, role }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    async fn build_test_pool() -> DbPool {
        let temp_path = std::env::temp_dir().join(format!("webx_db_test_{}.db", Uuid::new_v4()));
        let url = format!("sqlite://{}?mode=rwc", temp_path.display());
        init_db_with_url(&url, "admin-pass").await.expect("create test db")
    }

    #[tokio::test]
    async fn get_user_returns_admin_after_init() {
        let pool = build_test_pool().await;
        let user = get_user(&pool, "admin").await.expect("query admin");
        assert!(user.is_some(), "admin user should exist");
        let user = user.unwrap();
        assert_eq!(user.username, "admin");
        assert_eq!(user.role, "admin");
        assert!(user.password_hash.starts_with("$argon2"), "stored password should be hashed");
    }

    #[tokio::test]
    async fn save_metric_and_get_history_roundtrip() {
        let pool = build_test_pool().await;
        save_metric(&pool, 1.23, 4.56, 7.89, 0.12).await.expect("save metric");
        let history = get_history(&pool, 10).await.expect("fetch history");
        assert_eq!(history.len(), 1);
        let point = &history[0];
        assert_eq!(point.cpu, 1.23);
        assert_eq!(point.mem, 4.56);
        assert_eq!(point.users, 7.89);
        assert_eq!(point.rps, 0.12);
    }
}
