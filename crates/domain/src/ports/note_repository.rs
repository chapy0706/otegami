use async_trait::async_trait;
use time::OffsetDateTime;

use crate::entities::Note;
use crate::errors::RepositoryError;
use crate::value_objects::Slug;

/// ノートの永続化ポート。`dyn` で注入するため `Send + Sync` を要求する。
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// slug でノートを引く。存在しなければ `None`。
    async fn find_by_slug(&self, slug: &Slug) -> Result<Option<Note>, RepositoryError>;

    /// 初回閲覧成功時刻を記録する(追記のみ・終了時刻に相当する確定更新)。
    async fn mark_viewed(&self, slug: &Slug, at: OffsetDateTime) -> Result<(), RepositoryError>;
}
