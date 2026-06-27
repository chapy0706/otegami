//! Domain 層。
//!
//! フレームワーク・DB・認証・環境変数に依存しない純粋な Rust。
//! 他のどの層にも依存しない。

pub mod entities;
pub mod errors;
pub mod ports;
pub mod value_objects;
