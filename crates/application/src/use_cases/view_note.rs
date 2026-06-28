use std::sync::Arc;

use domain::errors::ViewNoteError;
use domain::ports::{Clock, NoteRepository, PasswordHasher};
use domain::value_objects::Slug;

/// 閲覧入力。slug とパスワードはともに外部入力(境界)であり、ここで検証する。
pub struct ViewNoteInput {
    pub raw_slug: String,
    pub password: String,
}

/// 閲覧出力。本文はそのまま返し、エスケープは描画側(Askama)に委ねる。
#[derive(Debug)]
pub struct ViewNoteOutput {
    pub title: Option<String>,
    pub body: String,
}

/// ノート閲覧のユースケース。
///
/// 失敗は「見つからない」と「パスワード違い」を区別せず
/// `NotFoundOrWrongPassword` に畳む(ADR-0001)。
pub struct ViewNote {
    notes: Arc<dyn NoteRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl ViewNote {
    pub fn new(
        notes: Arc<dyn NoteRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self {
            notes,
            hasher,
            clock,
        }
    }

    pub async fn execute(&self, input: ViewNoteInput) -> Result<ViewNoteOutput, ViewNoteError> {
        // 1. slug の正規化。不正な形は「無い」と同じ扱いに畳む。
        let slug =
            Slug::parse(&input.raw_slug).map_err(|_| ViewNoteError::NotFoundOrWrongPassword)?;

        // 2. 取得(Port 経由)。存在しなくても理由は明かさない。
        let note = self
            .notes
            .find_by_slug(&slug)
            .await
            .map_err(|_| ViewNoteError::Unexpected)?
            .ok_or(ViewNoteError::NotFoundOrWrongPassword)?;

        // 3. パスワード照合。失敗も同じ単一エラーに畳む。
        if !self.hasher.verify(&input.password, note.password_hash()) {
            return Err(ViewNoteError::NotFoundOrWrongPassword);
        }

        // 4. 初回閲覧なら viewed_at を記録(Clock 経由)。
        if note.viewed_at().is_none() {
            self.notes
                .mark_viewed(&slug, self.clock.now())
                .await
                .map_err(|_| ViewNoteError::Unexpected)?;
        }

        // 5. 本文はそのまま返す。
        Ok(ViewNoteOutput {
            title: note.title().map(str::to_owned),
            body: note.body().to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    use domain::entities::{Note, NoteSnapshot};
    use domain::errors::InsertError;
    use domain::ports::RepositoryError;
    use domain::value_objects::PasswordHash;

    struct FakeNotes {
        note: Option<Note>,
        viewed: Mutex<bool>,
    }

    #[async_trait]
    impl NoteRepository for FakeNotes {
        async fn insert(&self, _note: &Note) -> Result<(), InsertError> {
            Ok(())
        }
        async fn find_by_slug(&self, _slug: &Slug) -> Result<Option<Note>, RepositoryError> {
            Ok(self.note.clone())
        }
        async fn mark_viewed(
            &self,
            _slug: &Slug,
            _at: OffsetDateTime,
        ) -> Result<(), RepositoryError> {
            *self.viewed.lock().unwrap() = true;
            Ok(())
        }
        async fn delete_purgeable(&self, _now: OffsetDateTime) -> Result<u64, RepositoryError> {
            Ok(0)
        }
    }

    struct FakeHasher {
        accepts: bool,
    }
    impl PasswordHasher for FakeHasher {
        fn hash(&self, raw: &str) -> Result<PasswordHash, domain::errors::HashError> {
            Ok(PasswordHash::from_stored(format!("hash:{raw}")))
        }
        fn verify(&self, _raw: &str, _hash: &PasswordHash) -> bool {
            self.accepts
        }
    }

    struct FixedClock(OffsetDateTime);
    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.0
        }
    }

    fn sample_note(viewed_at: Option<OffsetDateTime>) -> Note {
        Note::restore(NoteSnapshot {
            slug: Slug::parse("ABC234").unwrap(),
            title: None,
            body: "hello".to_owned(),
            password_hash: PasswordHash::from_stored("$argon2id$...".to_owned()),
            burn_after_view: false,
            viewed_at,
            expires_at: OffsetDateTime::now_utc(),
            created_at: OffsetDateTime::now_utc(),
        })
    }

    #[tokio::test]
    async fn 正しいパスワードなら本文を返し_viewed_at_を立てる() {
        let notes = Arc::new(FakeNotes {
            note: Some(sample_note(None)),
            viewed: Mutex::new(false),
        });
        let view = ViewNote::new(
            notes.clone(),
            Arc::new(FakeHasher { accepts: true }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let out = view
            .execute(ViewNoteInput {
                raw_slug: "ABC234".into(),
                password: "0a1b".into(),
            })
            .await
            .unwrap();

        assert_eq!(out.body, "hello");
        assert!(*notes.viewed.lock().unwrap());
    }

    #[tokio::test]
    async fn 既に閲覧済みなら_viewed_at_を再記録しない() {
        let notes = Arc::new(FakeNotes {
            note: Some(sample_note(Some(OffsetDateTime::now_utc()))),
            viewed: Mutex::new(false),
        });
        let view = ViewNote::new(
            notes.clone(),
            Arc::new(FakeHasher { accepts: true }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let out = view
            .execute(ViewNoteInput {
                raw_slug: "ABC234".into(),
                password: "0a1b".into(),
            })
            .await
            .unwrap();

        assert_eq!(out.body, "hello");
        assert!(!*notes.viewed.lock().unwrap());
    }

    #[tokio::test]
    async fn パスワード違いは単一エラーに畳む() {
        let view = ViewNote::new(
            Arc::new(FakeNotes {
                note: Some(sample_note(None)),
                viewed: Mutex::new(false),
            }),
            Arc::new(FakeHasher { accepts: false }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let err = view
            .execute(ViewNoteInput {
                raw_slug: "ABC234".into(),
                password: "zzzz".into(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, ViewNoteError::NotFoundOrWrongPassword));
    }

    #[tokio::test]
    async fn 存在しない_slug_もパスワード違いと同じエラー() {
        let view = ViewNote::new(
            Arc::new(FakeNotes {
                note: None,
                viewed: Mutex::new(false),
            }),
            Arc::new(FakeHasher { accepts: true }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let err = view
            .execute(ViewNoteInput {
                raw_slug: "ABC234".into(),
                password: "0a1b".into(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, ViewNoteError::NotFoundOrWrongPassword));
    }
}
