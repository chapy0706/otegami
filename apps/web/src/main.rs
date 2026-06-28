use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Context;
use axum::middleware::from_fn_with_state;
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use time::Duration;

use application::use_cases::{CreateNote, ViewNote};
use domain::ports::{Clock, NoteRepository, PasswordHasher, SlugGenerator};
use infrastructure::{
    Argon2Hasher, CrockfordSlugGenerator, PgNoteRepository, Settings, SystemClock,
};

mod handlers;
mod rate_limit;
mod state;

use state::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 実行時設定(config.toml + 環境変数)。外部入力の境界。
    let settings = Settings::load().context("failed to load settings")?;

    let pool = PgPoolOptions::new()
        .connect(&settings.database_url)
        .await
        .context("failed to connect to Postgres")?;

    let config = Config {
        ttl: Duration::days(settings.ttl_days),
        slug_length: settings.slug_length,
        password_max_len: settings.password_max_len,
    };

    // 合成位置。具象を Arc<dyn Port> に包み、UseCase へ注入する。
    let notes: Arc<dyn NoteRepository> = Arc::new(PgNoteRepository::new(pool));
    let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2Hasher);
    let slugs: Arc<dyn SlugGenerator> = Arc::new(CrockfordSlugGenerator::new(config.slug_length));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    let state = AppState {
        create_note: Arc::new(CreateNote::new(
            notes.clone(),
            hasher.clone(),
            slugs,
            clock.clone(),
            config.ttl,
        )),
        view_note: Arc::new(ViewNote::new(notes, hasher, clock)),
        config,
    };

    // 閲覧 POST のレート制限(IP+slug 単位)。短いパスワードの成立条件(ADR-0001)。
    let limiter = Arc::new(rate_limit::build_view_limiter(
        settings.rate_limit_max,
        settings.rate_limit_period_secs,
    ));

    // 閲覧ルートにだけレート制限を掛ける。route_layer 適用後に追加する /create には掛からない。
    let view_routes = Router::new()
        .route(
            "/n/:slug",
            get(handlers::view::show_form).post(handlers::view::submit),
        )
        .route_layer(from_fn_with_state(limiter, rate_limit::rate_limit_view));

    let app = view_routes
        // 作成導線は Cloudflare Access の背後に置く(ADR-0004)。アプリ側では認証しない。
        .route(
            "/create",
            get(handlers::create::show_form).post(handlers::create::submit),
        )
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&settings.bind_addr)
        .await
        .with_context(|| format!("failed to bind {}", settings.bind_addr))?;
    println!("otegami web listening on {}", settings.bind_addr);
    // レート制限が送信元 IP を見るため、接続元情報を渡す。
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .context("server error")?;
    Ok(())
}
