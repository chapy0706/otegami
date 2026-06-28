//! Postgres(sqlx)による Adapter。

mod note_repository;

pub use note_repository::PgNoteRepository;

// DB を要する統合テストは `integration` feature でのみ取り込む。
#[cfg(all(test, feature = "integration"))]
mod note_repository_it;
