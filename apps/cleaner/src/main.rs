//! otegami-cleaner。
//!
//! 失効分(TTL満了)と閲覧済み burn 分を物理削除する掃除バッチ(ADR-0002, 0003)。
//! PurgeNotes を1回だけ実行して終了する。定期駆動は外部スケジューラに委ねる
//! (cron / systemd-timer / Coolify スケジュール。既定間隔10分)。
//! 削除条件は PurgeNotes に集約し、ここは合成と駆動だけを担う。

use std::sync::Arc;

use anyhow::Context;
use sqlx::postgres::PgPoolOptions;

use application::use_cases::PurgeNotes;
use domain::ports::{Clock, NoteRepository};
use infrastructure::{PgNoteRepository, Settings, SystemClock};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // web と同じ実行時設定を読む(config.toml + 環境変数)。
    let settings = Settings::load().context("failed to load settings")?;
    let pool = PgPoolOptions::new()
        .connect(&settings.database_url)
        .await
        .context("failed to connect to Postgres")?;

    // 合成位置。具象を Arc<dyn Port> に包んで PurgeNotes へ注入する。
    let notes: Arc<dyn NoteRepository> = Arc::new(PgNoteRepository::new(pool));
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let purge = PurgeNotes::new(notes, clock);

    // 失敗時は `?` で非ゼロ終了する(anyhow がエラーを表示)。
    let out = purge.execute().await.context("purge failed")?;
    println!("otegami-cleaner: deleted {} notes", out.deleted);
    Ok(())
}
