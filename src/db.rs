use sqlx::SqlitePool; use chrono::{DateTime, Utc};
#[derive(Debug, Clone, sqlx::FromRow, serde::Serialize)] pub struct MetricPoint { pub id: i64, pub ts: DateTime<Utc>, pub cpu: f64, pub mem: f64, pub users: f64, pub rps: f64 }
#[derive(Debug, Clone, sqlx::FromRow)] pub struct UserRow { pub username: String, pub password_hash: String, pub role: String }
pub type DbPool = SqlitePool;

pub async fn init_db() -> anyhow::Result<DbPool> {
    let url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data.db?mode=rwc".into());
    let pool = SqlitePool::connect(&url).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS metrics (id INTEGER PRIMARY KEY AUTOINCREMENT, ts TEXT NOT NULL, cpu REAL, mem REAL, users REAL, rps REAL)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY AUTOINCREMENT, username TEXT UNIQUE NOT NULL, password_hash TEXT NOT NULL, role TEXT NOT NULL)").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS refresh_tokens (jti TEXT PRIMARY KEY, username TEXT NOT NULL, revoked BOOLEAN NOT NULL DEFAULT 0, expires_at TEXT NOT NULL)").execute(&pool).await?;
    let (cnt,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users").fetch_one(&pool).await?;
    if cnt==0 { 
        let pass = std::env::var("ADMIN_PASS").unwrap_or_else(|_| "admin123".into()); 
        let hash = crate::auth::hash_password(&pass)?; 
        sqlx::query("INSERT INTO users (username,password_hash,role) VALUES (?,?,?)").bind("admin").bind(hash).bind("admin").execute(&pool).await?;
        
        let pass_viewer = "viewer123";
        let hash_viewer = crate::auth::hash_password(&pass_viewer)?;
        sqlx::query("INSERT INTO users (username,password_hash,role) VALUES (?,?,?)").bind("viewer").bind(hash_viewer).bind("viewer").execute(&pool).await?;
    }
    Ok(pool)
}

pub async fn save_metric(pool: &DbPool, cpu: f64, mem: f64, users: f64, rps: f64) -> anyhow::Result<()> {
    sqlx::query("INSERT INTO metrics (ts,cpu,mem,users,rps) VALUES (?,?,?,?,?)").bind(Utc::now().to_rfc3339()).bind(cpu).bind(mem).bind(users).bind(rps).execute(pool).await?;
    sqlx::query("DELETE FROM metrics WHERE id NOT IN (SELECT id FROM metrics ORDER BY id DESC LIMIT 10000)").execute(pool).await?; Ok(())
}
pub async fn get_history(pool: &DbPool, lim: i64) -> anyhow::Result<Vec<MetricPoint>> {
    let mut r = sqlx::query_as::<_, MetricPoint>("SELECT id,ts,cpu,mem,users,rps FROM metrics ORDER BY id DESC LIMIT ?").bind(lim).fetch_all(pool).await?; r.reverse(); Ok(r)
}
pub async fn get_user(pool: &DbPool, u: &str) -> anyhow::Result<Option<UserRow>> {
    Ok(sqlx::query_as::<_, UserRow>("SELECT username,password_hash,role FROM users WHERE username=?").bind(u).fetch_optional(pool).await?)
}
