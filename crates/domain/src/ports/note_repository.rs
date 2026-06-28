use async_trait::async_trait;
use time::OffsetDateTime;

use crate::entities::Note;
use crate::errors::{InsertError, RepositoryError};
use crate::value_objects::Slug;

/// ノートの永続化ポート。`dyn` で注入するため `Send + Sync` を要求する。
#[async_trait]
pub trait NoteRepository: Send + Sync {
    /// 新規ノートを挿入する。slug の一意制約違反は `InsertError::Conflict` で返し、
    /// ユースケース側の slug 採番リトライに委ねる。
    async fn insert(&self, note: &Note) -> Result<(), InsertError>;

    /// slug でノートを引く。存在しなければ `None`。
    async fn find_by_slug(&self, slug: &Slug) -> Result<Option<Note>, RepositoryError>;

    /// 初回閲覧成功時刻を記録する(追記のみ・終了時刻に相当する確定更新)。
    async fn mark_viewed(&self, slug: &Slug, at: OffsetDateTime) -> Result<(), RepositoryError>;

    /// 失効分(TTL満了)と閲覧済み burn 分を物理削除し、削除件数を返す(ADR-0003)。
    async fn delete_purgeable(&self, now: OffsetDateTime) -> Result<u64, RepositoryError>;
}
