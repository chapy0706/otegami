use crate::errors::HashError;
use crate::value_objects::PasswordHash;

/// パスワードのハッシュ化と照合(Argon2 実装を想定)。
pub trait PasswordHasher: Send + Sync {
    /// 平文をハッシュ化する。
    fn hash(&self, raw: &str) -> Result<PasswordHash, HashError>;

    /// 平文と保存済みハッシュを照合する。
    fn verify(&self, raw: &str, hash: &PasswordHash) -> bool;
}
