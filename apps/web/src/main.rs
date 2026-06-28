use std::sync::Arc;

use anyhow::Context;
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use time::Duration;

use application::use_cases::{CreateNote, ViewNote};
use domain::ports::{Clock, NoteRepository, PasswordHasher, SlugGenerator};
use infrastructure::{Argon2Hasher, CrockfordSlugGenerator, PgNoteRepository, SystemClock};

mod handlers;
mod state;

use state::{AppState, Config};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;
    let pool = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .context("failed to connect to Postgres")?;

    // 設定。外部化(bootstrap.toml / 環境)は issue-08。ここでは既定値を置く。
    let config = Config {
        ttl: Duration::days(1),
        slug_length: 6,
        password_max_len: 4,
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

    let app = Router::new()
        // 閲覧は公開。POST のレート制限ミドルウェアは issue-08(cross-cutting)で付与する。
        .route(
            "/n/:slug",
            get(handlers::view::show_form).post(handlers::view::submit),
        )
        // 作成導線は Cloudflare Access の背後に置く(ADR-0004)。アプリ側では認証しない。
        .route(
            "/create",
            get(handlers::create::show_form).post(handlers::create::submit),
        )
        .with_state(state);

    let addr = "0.0.0.0:8080";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("failed to bind {addr}"))?;
    println!("otegami web listening on {addr}");
    axum::serve(listener, app).await.context("server error")?;
    Ok(())
}
