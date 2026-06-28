//! 閲覧 POST のレート制限。
//!
//! 短いパスワードを成立させる前提条件(ADR-0001)。同一 IP・同一 slug 単位で
//! 試行回数を制限し、総当たりを実質的に止める。超過時は 429 を返す。
//!
//! Cloudflare Tunnel 経由では送信元 IP が `CF-Connecting-IP` に入るため、
//! それを優先し、無ければ `X-Forwarded-For` の先頭、最後に接続元 IP を使う。

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{ConnectInfo, Request, State};
use axum::http::{Method, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use governor::clock::DefaultClock;
use governor::state::keyed::DefaultKeyedStateStore;
use governor::{Quota, RateLimiter};

use crate::handlers::render_error;

/// IP+slug をキーにした閲覧用レートリミッタ。
pub type ViewRateLimiter = RateLimiter<String, DefaultKeyedStateStore<String>, DefaultClock>;

/// `max` 回 / `period_secs` 秒 を許すリミッタを作る。
pub fn build_view_limiter(max: u32, period_secs: u64) -> ViewRateLimiter {
    let max = NonZeroU32::new(max.max(1)).expect("max is at least 1");
    // 1セルの補充間隔 = 期間 / 最大回数。最低 1 秒。
    let period = (Duration::from_secs(period_secs.max(1)) / max.get()).max(Duration::from_secs(1));
    let quota = Quota::with_period(period)
        .expect("period is non-zero")
        .allow_burst(max);
    RateLimiter::keyed(quota)
}

/// 閲覧 POST にだけ制限を掛けるミドルウェア。GET(フォーム表示)は素通しする。
pub async fn rate_limit_view(
    State(limiter): State<Arc<ViewRateLimiter>>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    req: Request,
    next: Next,
) -> Response {
    if req.method() != Method::POST {
        return next.run(req).await;
    }

    let key = view_key(&req, peer);
    match limiter.check_key(&key) {
        Ok(_) => next.run(req).await,
        Err(_) => render_error(
            "試行が多すぎます。しばらく待ってから、もう一度お試しください",
            StatusCode::TOO_MANY_REQUESTS,
        ),
    }
}

/// IP と slug を結合したキー。slug は大文字へ正規化し、表記揺れで枠を分けない。
fn view_key(req: &Request, peer: SocketAddr) -> String {
    let ip = client_ip(req, peer);
    let slug = req
        .uri()
        .path()
        .strip_prefix("/n/")
        .unwrap_or_default()
        .to_ascii_uppercase();
    format!("{ip}|{slug}")
}

/// 送信元 IP。CF-Connecting-IP → X-Forwarded-For 先頭 → 接続元 の順。
fn client_ip(req: &Request, peer: SocketAddr) -> String {
    let header = |name: &str| {
        req.headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::trim)
            .filter(|s| !s.is_empty())
    };

    if let Some(ip) = header("cf-connecting-ip") {
        return ip.to_owned();
    }
    if let Some(xff) = header("x-forwarded-for") {
        if let Some(first) = xff.split(',').next() {
            let first = first.trim();
            if !first.is_empty() {
                return first.to_owned();
            }
        }
    }
    peer.ip().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 同一キーは上限回数で打ち止めになる() {
        let limiter = build_view_limiter(5, 600);
        let key = "1.2.3.4|ABC234".to_owned();
        // 5回までは許可、6回目で拒否(バースト上限)。
        for _ in 0..5 {
            assert!(limiter.check_key(&key).is_ok());
        }
        assert!(limiter.check_key(&key).is_err());
    }

    #[test]
    fn 別の_ip_や_slug_は独立に数える() {
        let limiter = build_view_limiter(5, 600);
        let a = "1.1.1.1|ABC234".to_owned();
        let b = "2.2.2.2|ABC234".to_owned();
        let c = "1.1.1.1|ZZZ999".to_owned();
        for _ in 0..5 {
            assert!(limiter.check_key(&a).is_ok());
        }
        assert!(limiter.check_key(&a).is_err());
        // IP が違えば独立。
        assert!(limiter.check_key(&b).is_ok());
        // slug が違えば独立。
        assert!(limiter.check_key(&c).is_ok());
    }
}
