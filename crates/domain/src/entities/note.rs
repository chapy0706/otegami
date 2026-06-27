use time::OffsetDateTime;

use crate::value_objects::{PasswordHash, Slug};

/// 永続化層などから Note を復元するための素材。
///
/// `Note::restore` 専用の入口。検証はここでは行わず、すでに整合した状態
/// (作成時に検証済み・DB から読み出した値)を組み立て直す用途に使う。
#[derive(Debug, Clone)]
pub struct NoteSnapshot {
    pub slug: Slug,
    pub title: Option<String>,
    pub body: String,
    pub password_hash: PasswordHash,
    pub burn_after_view: bool,
    pub viewed_at: Option<OffsetDateTime>,
    pub expires_at: OffsetDateTime,
    pub created_at: OffsetDateTime,
}

/// ノート本体。otegami の中心となるエンティティ。
///
/// 不変条件を保つため内部状態は非公開とし、参照はアクセサ経由で渡す。
#[derive(Debug, Clone)]
pub struct Note {
    slug: Slug,
    title: Option<String>,
    body: String,
    password_hash: PasswordHash,
    burn_after_view: bool,
    viewed_at: Option<OffsetDateTime>,
    expires_at: OffsetDateTime,
    created_at: OffsetDateTime,
}

impl Note {
    /// スナップショットから Note を復元する。
    pub fn restore(snapshot: NoteSnapshot) -> Self {
        Self {
            slug: snapshot.slug,
            title: snapshot.title,
            body: snapshot.body,
            password_hash: snapshot.password_hash,
            burn_after_view: snapshot.burn_after_view,
            viewed_at: snapshot.viewed_at,
            expires_at: snapshot.expires_at,
            created_at: snapshot.created_at,
        }
    }

    pub fn slug(&self) -> &Slug {
        &self.slug
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> &str {
        &self.body
    }

    pub fn password_hash(&self) -> &PasswordHash {
        &self.password_hash
    }

    pub fn burn_after_view(&self) -> bool {
        self.burn_after_view
    }

    pub fn viewed_at(&self) -> Option<OffsetDateTime> {
        self.viewed_at
    }

    pub fn expires_at(&self) -> OffsetDateTime {
        self.expires_at
    }

    pub fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
}
