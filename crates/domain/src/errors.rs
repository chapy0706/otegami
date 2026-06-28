//! ドメイン/ユースケースのエラー。

use thiserror::Error;

/// 閲覧失敗。「見つからない」と「パスワード違い」を区別しない単一の値に畳む(ADR-0001)。
/// 理由を持たせないこと自体が設計であり、ノートの存在を推し量らせない。
#[derive(Debug, Error)]
pub enum ViewNoteError {
    #[error("not found or wrong password")]
    NotFoundOrWrongPassword,
    #[error("unexpected error")]
    Unexpected,
}

/// 作成失敗。作成は運用者のみが行うため、原因の区別を持たせてよい。
#[derive(Debug, Error)]
pub enum CreateNoteError {
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("could not allocate a unique slug")]
    SlugUnavailable,
    #[error("unexpected error")]
    Unexpected,
}

/// 掃除失敗。掃除は運用経路(バッチ)のため、原因を持たせず単一の値に畳む。
#[derive(Debug, Error)]
pub enum PurgeNotesError {
    #[error("unexpected error")]
    Unexpected,
}

/// 永続化層から返る低レベルなエラー。ユースケース側で適切な単一エラーへ畳む。
#[derive(Debug, Error)]
#[error("repository error: {0}")]
pub struct RepositoryError(pub String);

/// ノート挿入の失敗。slug の一意制約違反(衝突)を区別し、ユースケース側の
/// slug 採番リトライ判断に使う。それ以外の障害は `Backend` に畳む。
#[derive(Debug, Error)]
pub enum InsertError {
    #[error("slug already exists")]
    Conflict,
    #[error(transparent)]
    Backend(#[from] RepositoryError),
}

/// パスワードハッシュ化の失敗。
#[derive(Debug, Error)]
#[error("failed to hash password")]
pub struct HashError;
