//! Infrastructure 層。
//!
//! Domain が定義する Port を実装する(DB・外部 API など)。
//! sqlx / argon2 / rand はこの crate にのみ入り、domain には漏らさない。

pub mod argon2_hasher;
pub mod config;
pub mod crockford_slug;
pub mod pg;
pub mod system_clock;

pub use argon2_hasher::Argon2Hasher;
pub use config::{ConfigError, Settings};
pub use crockford_slug::CrockfordSlugGenerator;
pub use pg::PgNoteRepository;
pub use system_clock::SystemClock;
