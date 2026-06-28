use std::sync::Arc;

use domain::errors::PurgeNotesError;
use domain::ports::{Clock, NoteRepository};

/// 掃除出力。削除した件数を返す。
#[derive(Debug)]
pub struct PurgeNotesOutput {
    pub deleted: u64,
}

/// 掃除のユースケース。失効分(TTL満了)と閲覧済み burn 分を物理削除する(ADR-0003)。
///
/// 削除条件は永続化層(Adapter)の単一クエリに委ね、ここでは現在時刻を渡して件数を受け取る。
/// 掃除バッチ(otegami-cleaner)から駆動される。
pub struct PurgeNotes {
    notes: Arc<dyn NoteRepository>,
    clock: Arc<dyn Clock>,
}

impl PurgeNotes {
    pub fn new(notes: Arc<dyn NoteRepository>, clock: Arc<dyn Clock>) -> Self {
        Self { notes, clock }
    }

    pub async fn execute(&self) -> Result<PurgeNotesOutput, PurgeNotesError> {
        let now = self.clock.now();
        let deleted = self
            .notes
            .delete_purgeable(now)
            .await
            .map_err(|_| PurgeNotesError::Unexpected)?;
        Ok(PurgeNotesOutput { deleted })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    use domain::entities::Note;
    use domain::errors::{InsertError, RepositoryError};
    use domain::value_objects::Slug;

    struct FakeNotes {
        deleted: u64,
        called_with: Mutex<Option<OffsetDateTime>>,
    }

    #[async_trait]
    impl NoteRepository for FakeNotes {
        async fn insert(&self, _note: &Note) -> Result<(), InsertError> {
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
        async fn delete_purgeable(&self, now: OffsetDateTime) -> Result<u64, RepositoryError> {
            *self.called_with.lock().unwrap() = Some(now);
            Ok(self.deleted)
        }
    }

    struct FixedClock(OffsetDateTime);
    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.0
        }
    }

    #[tokio::test]
    async fn 削除件数を返し_現在時刻を_clock_経由で渡す() {
        let now = OffsetDateTime::UNIX_EPOCH;
        let notes = Arc::new(FakeNotes {
            deleted: 3,
            called_with: Mutex::new(None),
        });
        let purge = PurgeNotes::new(notes.clone(), Arc::new(FixedClock(now)));

        let out = purge.execute().await.unwrap();

        assert_eq!(out.deleted, 3);
        assert_eq!(*notes.called_with.lock().unwrap(), Some(now));
    }
}
