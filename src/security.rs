use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use dashmap::DashMap;
use once_cell::sync::Lazy;
use rand::RngCore;
use std::time::{Duration, Instant};

pub static RATE_LIMIT: Lazy<DashMap<String, (u32, Instant)>> = Lazy::new(DashMap::new);

pub fn check_rate_limit(key: String, max: u32, window: Duration) -> bool {
    let now = Instant::now();
    let mut e = RATE_LIMIT.entry(key).or_insert((0, now));
    if now.duration_since(e.1) > window {
        *e = (1, now);
        return true;
    }
    if e.0 >= max {
        return false;
    }
    e.0 += 1;
    true
}

pub fn generate_csrf_token() -> String {
    let mut b = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut b);
    URL_SAFE_NO_PAD.encode(b)
}

pub async fn security_headers_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut res = next.run(req).await;
    let h = res.headers_mut();
    h.insert(
        "Strict-Transport-Security",
        "max-age=63072000; includeSubDomains; preload"
            .parse()
            .unwrap(),
    );
    h.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    h.insert("X-Frame-Options", "DENY".parse().unwrap());
    h.insert("Content-Security-Policy", "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; object-src 'none'; frame-ancestors 'none'".parse().unwrap());
    h.insert(
        "Permissions-Policy",
        "geolocation=(), microphone=(), camera=()".parse().unwrap(),
    );
    Ok(res)
}

pub async fn csrf_middleware(req: Request<Body>, next: Next) -> Result<Response, StatusCode> {
    if matches!(req.method().as_str(), "POST" | "PUT" | "DELETE" | "PATCH") {
        let p = req.uri().path();
        if p == "/api/auth/login" || p == "/api/auth/refresh" {
            return Ok(next.run(req).await);
        }
        let headers = req.headers();
        let cookie_h = headers
            .get(axum::http::header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        let c1 = cookie_h
            .split(';')
            .find_map(|p| p.trim().strip_prefix("csrf_token=").map(|s| s.to_string()));
        let c2 = headers.get("x-csrf-token").and_then(|v| v.to_str().ok());
        if c1.is_none() || c2.is_none() || c1.as_deref() != c2 {
            return Err(StatusCode::FORBIDDEN);
        }
    }
    Ok(next.run(req).await)
}

pub fn client_ip_from_headers(headers: &axum::http::HeaderMap) -> Option<std::net::IpAddr> {
    if let Some(xff) = headers.get("x-forwarded-for") {
        if let Ok(s) = xff.to_str() {
            // Read from the right side of the list to prevent client-spoofing.
            // Under a trusted reverse proxy configuration, the proxy appends the real client IP
            // to the right side of the list.
            if let Some(last_ip_str) = s.split(',').next_back() {
                if let Ok(ip) = last_ip_str.trim().parse::<std::net::IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }
    if let Some(xri) = headers.get("x-real-ip") {
        if let Ok(s) = xri.to_str() {
            if let Ok(ip) = s.trim().parse::<std::net::IpAddr>() {
                return Some(ip);
            }
        }
    }
    None
}

pub async fn global_rate_limit_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let ip = client_ip_from_headers(req.headers())
        .map(|ip| ip.to_string())
        .unwrap_or_else(|| {
            req.extensions()
                .get::<axum::extract::ConnectInfo<std::net::SocketAddr>>()
                .map(|c| c.0.ip().to_string())
                .unwrap_or_else(|| "unknown".into())
        });
    if !check_rate_limit(format!("global:{ip}"), 100, Duration::from_secs(60)) {
        return Err(StatusCode::TOO_MANY_REQUESTS);
    }
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderMap;
    use std::net::IpAddr;

    #[test]
    fn test_client_ip_from_headers() {
        // Test X-Forwarded-For parsing (last IP to avoid client spoofing)
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-forwarded-for",
            "203.0.113.195, 70.41.3.18, 150.172.238.178"
                .parse()
                .unwrap(),
        );
        let ip = client_ip_from_headers(&headers).expect("should extract IP");
        assert_eq!(ip, "150.172.238.178".parse::<IpAddr>().unwrap());

        // Test X-Real-IP fallback
        let mut headers = HeaderMap::new();
        headers.insert("x-real-ip", "198.51.100.1".parse().unwrap());
        let ip = client_ip_from_headers(&headers).expect("should extract IP");
        assert_eq!(ip, "198.51.100.1".parse::<IpAddr>().unwrap());

        // Test no IP headers
        let headers = HeaderMap::new();
        assert!(client_ip_from_headers(&headers).is_none());
    }
}
