//! ポート(trait)。利用側である domain に置き、外側(infrastructure)が実装する。

mod clock;
mod note_repository;
mod password_hasher;
mod slug_generator;

pub use clock::Clock;
pub use note_repository::NoteRepository;
pub use password_hasher::PasswordHasher;
pub use slug_generator::SlugGenerator;

// ポートのシグネチャで使うエラーは errors に定義し、ここから再公開する。
pub use crate::errors::{HashError, InsertError, RepositoryError};
