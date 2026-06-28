use std::sync::Arc;

use time::Duration;

use domain::entities::{Note, NoteSnapshot};
use domain::errors::{CreateNoteError, InsertError};
use domain::ports::{Clock, NoteRepository, PasswordHasher, SlugGenerator};
use domain::value_objects::{NoteBody, NoteTitle, RawPassword};

/// slug 衝突時のリトライ上限。一意な slug を採番できなければ作成を諦める。
const MAX_SLUG_ATTEMPTS: usize = 5;

/// 作成入力。本文・パスワードは外部入力(境界)であり、ここで値オブジェクトに通して検証する。
pub struct CreateNoteInput {
    pub body: String,
    pub title: Option<String>,
    pub raw_password: String,
    pub burn_after_view: bool,
}

/// 作成出力。発行した slug を返す。
#[derive(Debug)]
pub struct CreateNoteOutput {
    pub slug: String,
}

/// ノート作成のユースケース。
///
/// パスワード検証 → ハッシュ化 → slug 採番(衝突時リトライ)→ `expires_at` 算出 → 保存、
/// という otegami の作成手順を集約する。作成は運用者のみが行うため、失敗原因は区別してよい。
pub struct CreateNote {
    notes: Arc<dyn NoteRepository>,
    hasher: Arc<dyn PasswordHasher>,
    slugs: Arc<dyn SlugGenerator>,
    clock: Arc<dyn Clock>,
    /// TTL は設定値(既定1日)。合成位置(presentation)から渡す。
    ttl: Duration,
}

impl CreateNote {
    pub fn new(
        notes: Arc<dyn NoteRepository>,
        hasher: Arc<dyn PasswordHasher>,
        slugs: Arc<dyn SlugGenerator>,
        clock: Arc<dyn Clock>,
        ttl: Duration,
    ) -> Self {
        Self {
            notes,
            hasher,
            slugs,
            clock,
            ttl,
        }
    }

    pub async fn execute(
        &self,
        input: CreateNoteInput,
    ) -> Result<CreateNoteOutput, CreateNoteError> {
        // 1. 外部入力を値オブジェクトで検証する。
        let password = RawPassword::parse(&input.raw_password)
            .map_err(|e| CreateNoteError::InvalidInput(e.to_string()))?;
        let body = NoteBody::parse(input.body)
            .map_err(|e| CreateNoteError::InvalidInput(e.to_string()))?;
        let title = match input.title {
            Some(raw) => Some(
                NoteTitle::parse(raw).map_err(|e| CreateNoteError::InvalidInput(e.to_string()))?,
            ),
            None => None,
        };

        // 2. ハッシュ化は PasswordHasher 経由。平文はここから先へ持ち出さない。
        let password_hash = self
            .hasher
            .hash(password.as_str())
            .map_err(|_| CreateNoteError::Unexpected)?;

        // 3. 時刻は Clock 経由。TTL から失効時刻を算出する(ADR-0003)。
        let created_at = self.clock.now();
        let expires_at = created_at + self.ttl;

        // 4. slug を採番して保存。一意制約違反なら別の slug でリトライする。
        for _ in 0..MAX_SLUG_ATTEMPTS {
            let slug = self.slugs.generate();
            let note = Note::restore(NoteSnapshot {
                slug: slug.clone(),
                title: title.as_ref().map(|t| t.as_str().to_owned()),
                body: body.as_str().to_owned(),
                password_hash: password_hash.clone(),
                burn_after_view: input.burn_after_view,
                viewed_at: None,
                expires_at,
                created_at,
            });

            match self.notes.insert(&note).await {
                Ok(()) => {
                    return Ok(CreateNoteOutput {
                        slug: slug.as_str().to_owned(),
                    })
                }
                Err(InsertError::Conflict) => continue,
                Err(InsertError::Backend(_)) => return Err(CreateNoteError::Unexpected),
            }
        }

        Err(CreateNoteError::SlugUnavailable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    use domain::errors::{HashError, RepositoryError};
    use domain::value_objects::{PasswordHash, Slug};

    /// insert の挙動を制御するフェイク。最初の `conflicts` 回は衝突を返す。
    struct FakeNotes {
        conflicts: Mutex<usize>,
        inserted: Mutex<Vec<Note>>,
    }

    impl FakeNotes {
        fn new(conflicts: usize) -> Self {
            Self {
                conflicts: Mutex::new(conflicts),
                inserted: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl NoteRepository for FakeNotes {
        async fn insert(&self, note: &Note) -> Result<(), InsertError> {
            let mut remaining = self.conflicts.lock().unwrap();
            if *remaining > 0 {
                *remaining -= 1;
                return Err(InsertError::Conflict);
            }
            self.inserted.lock().unwrap().push(note.clone());
            Ok(())
        }
        async fn find_by_slug(&self, _slug: &Slug) -> Result<Option<Note>, RepositoryError> {
            Ok(None)
        }
        async fn mark_viewed(
            &self,
            _slug: &Slug,
            _at: OffsetDateTime,
        ) -> Result<(), RepositoryError> {
            Ok(())
        }
        async fn delete_purgeable(&self, _now: OffsetDateTime) -> Result<u64, RepositoryError> {
            Ok(0)
        }
    }

    struct FakeHasher;
    impl PasswordHasher for FakeHasher {
        fn hash(&self, raw: &str) -> Result<PasswordHash, HashError> {
            Ok(PasswordHash::from_stored(format!("hash:{raw}")))
        }
        fn verify(&self, _raw: &str, _hash: &PasswordHash) -> bool {
            true
        }
    }

    /// 呼ばれるたびに連番の slug を返すジェネレータ。
    struct SeqSlugs {
        next: Mutex<u32>,
    }
    impl SeqSlugs {
        fn new() -> Self {
            Self {
                next: Mutex::new(0),
            }
        }
    }
    impl SlugGenerator for SeqSlugs {
        fn generate(&self) -> Slug {
            let mut n = self.next.lock().unwrap();
            // Crockford 文字集合に収まる固定長 slug を機械的に作る。
            let slug = Slug::parse(&format!("AAAA{:02}", *n)).unwrap();
            *n += 1;
            slug
        }
    }

    struct FixedClock(OffsetDateTime);
    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.0
        }
    }

    fn build(notes: Arc<FakeNotes>) -> CreateNote {
        CreateNote::new(
            notes,
            Arc::new(FakeHasher),
            Arc::new(SeqSlugs::new()),
            Arc::new(FixedClock(OffsetDateTime::UNIX_EPOCH)),
            Duration::days(1),
        )
    }

    fn input() -> CreateNoteInput {
        CreateNoteInput {
            body: "hello".into(),
            title: Some("memo".into()),
            raw_password: "0a1b".into(),
            burn_after_view: true,
        }
    }

    #[tokio::test]
    async fn 正常系は_slug_を返し失効時刻を_ttl_で算出する() {
        let notes = Arc::new(FakeNotes::new(0));
        let create = build(notes.clone());

        let out = create.execute(input()).await.unwrap();
        assert_eq!(out.slug, "AAAA00");

        let stored = notes.inserted.lock().unwrap();
        assert_eq!(stored.len(), 1);
        let note = &stored[0];
        assert_eq!(note.body(), "hello");
        assert_eq!(note.title(), Some("memo"));
        assert!(note.burn_after_view());
        assert!(note.viewed_at().is_none());
        assert_eq!(note.created_at(), OffsetDateTime::UNIX_EPOCH);
        assert_eq!(
            note.expires_at(),
            OffsetDateTime::UNIX_EPOCH + Duration::days(1)
        );
    }

    #[tokio::test]
    async fn slug_衝突時は別の_slug_でリトライする() {
        let notes = Arc::new(FakeNotes::new(2));
        let create = build(notes.clone());

        let out = create.execute(input()).await.unwrap();
        // 0,1 が衝突し 2 個目(3 回目の採番)で成功する。
        assert_eq!(out.slug, "AAAA02");
    }

    #[tokio::test]
    async fn リトライ上限を超える衝突は_slug_unavailable() {
        let notes = Arc::new(FakeNotes::new(MAX_SLUG_ATTEMPTS));
        let create = build(notes.clone());

        let err = create.execute(input()).await.unwrap_err();
        assert!(matches!(err, CreateNoteError::SlugUnavailable));
    }

    #[tokio::test]
    async fn 不正なパスワードは_invalid_input() {
        let notes = Arc::new(FakeNotes::new(0));
        let create = build(notes.clone());

        let err = create
            .execute(CreateNoteInput {
                raw_password: "あ".into(),
                ..input()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, CreateNoteError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn 空の本文は_invalid_input() {
        let notes = Arc::new(FakeNotes::new(0));
        let create = build(notes.clone());

        let err = create
            .execute(CreateNoteInput {
                body: "".into(),
                ..input()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, CreateNoteError::InvalidInput(_)));
    }
}
