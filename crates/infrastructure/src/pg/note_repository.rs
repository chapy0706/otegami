use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;

use domain::entities::{Note, NoteSnapshot};
use domain::errors::{InsertError, RepositoryError};
use domain::ports::NoteRepository;
use domain::value_objects::{PasswordHash, Slug};

/// `NoteRepository` の Postgres 具象。業務判断は持たず、境界での変換に徹する。
pub struct PgNoteRepository {
    pool: PgPool,
}

impl PgNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// 失敗に「Adapter名.メソッド名: ...」の文脈を添える。
fn context(adapter_method: &str, e: impl std::fmt::Display) -> RepositoryError {
    RepositoryError(format!("{adapter_method}: {e}"))
}

#[async_trait]
impl NoteRepository for PgNoteRepository {
    async fn insert(&self, note: &Note) -> Result<(), InsertError> {
        // viewed_at は新規作成時は常に未閲覧(NULL)。created_at / expires_at は
        // ユースケースが Clock 経由で確定済みの値をそのまま書く。
        sqlx::query!(
            r#"
            INSERT INTO notes
                (slug, title, body, password_hash, burn_after_view, expires_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
            note.slug().as_str(),
            note.title(),
            note.body(),
            note.password_hash().as_str(),
            note.burn_after_view(),
            note.expires_at(),
            note.created_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(|e| match &e {
            // slug の一意制約違反は衝突として返し、採番リトライに委ねる。
            sqlx::Error::Database(db) if db.is_unique_violation() => InsertError::Conflict,
            _ => InsertError::Backend(context("PgNoteRepository.insert", e)),
        })?;
        Ok(())
    }

    async fn find_by_slug(&self, slug: &Slug) -> Result<Option<Note>, RepositoryError> {
        // expires_at で論理失効を表現する。物理削除(cleaner)前でも、
        // 期限切れは「無い」として扱う。
        let row = sqlx::query!(
            r#"
            SELECT slug, title, body, password_hash,
                   burn_after_view, viewed_at, expires_at, created_at
            FROM notes
            WHERE slug = $1 AND expires_at > now()
            "#,
            slug.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| context("PgNoteRepository.find_by_slug", e))?;

        let Some(row) = row else {
            return Ok(None);
        };

        // 境界での変換: DB 行 -> ドメインエンティティ。値オブジェクトは parse を通す。
        let note = Note::restore(NoteSnapshot {
            slug: Slug::parse(&row.slug)
                .map_err(|e| context("PgNoteRepository.find_by_slug(slug)", e))?,
            title: row.title,
            body: row.body,
            password_hash: PasswordHash::from_stored(row.password_hash),
            burn_after_view: row.burn_after_view,
            viewed_at: row.viewed_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        });
        Ok(Some(note))
    }

    async fn mark_viewed(&self, slug: &Slug, at: OffsetDateTime) -> Result<(), RepositoryError> {
        // 初回のみ確定する。二度目以降は viewed_at を上書きしない。
        sqlx::query!(
            r#"UPDATE notes SET viewed_at = $2 WHERE slug = $1 AND viewed_at IS NULL"#,
            slug.as_str(),
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| context("PgNoteRepository.mark_viewed", e))?;
        Ok(())
    }

    async fn delete_purgeable(&self, now: OffsetDateTime) -> Result<u64, RepositoryError> {
        // 失効分(TTL満了)と閲覧済み burn 分を一文の DELETE で消す(ADR-0003)。
        let result = sqlx::query!(
            r#"
            DELETE FROM notes
            WHERE expires_at < $1
               OR (burn_after_view AND viewed_at IS NOT NULL)
            "#,
            now,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| context("PgNoteRepository.delete_purgeable", e))?;
        Ok(result.rows_affected())
    }
}
