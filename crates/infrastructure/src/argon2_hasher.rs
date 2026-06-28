use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash as Argon2Hash, PasswordHasher as _, PasswordVerifier as _};

use domain::errors::HashError;
use domain::ports::PasswordHasher;
use domain::value_objects::PasswordHash;

/// Argon2 によるパスワードハッシュの具象。
///
/// domain の `PasswordHasher` と argon2 クレートのトレイト名が衝突するため、
/// 後者は `as _` で取り込む。verify は CPU 律速のため、ハンドラから呼ぶ際は
/// `tokio::task::spawn_blocking` への退避を検討する(issue-06)。
pub struct Argon2Hasher;

impl PasswordHasher for Argon2Hasher {
    fn hash(&self, raw: &str) -> Result<PasswordHash, HashError> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .map_err(|_| HashError)?
            .to_string();
        Ok(PasswordHash::from_stored(hash))
    }

    fn verify(&self, raw: &str, hash: &PasswordHash) -> bool {
        // 保存済みハッシュが壊れていても「照合失敗」に畳む(理由は明かさない)。
        let Ok(parsed) = Argon2Hash::new(hash.as_str()) else {
            return false;
        };
        Argon2::default()
            .verify_password(raw.as_bytes(), &parsed)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_した値は同じ平文で_verify_できる() {
        let hasher = Argon2Hasher;
        let hash = hasher.hash("0a1b").unwrap();
        assert!(hasher.verify("0a1b", &hash));
    }

    #[test]
    fn 異なる平文では_verify_に失敗する() {
        let hasher = Argon2Hasher;
        let hash = hasher.hash("0a1b").unwrap();
        assert!(!hasher.verify("zzzz", &hash));
    }

    #[test]
    fn 壊れたハッシュは_verify_に失敗する() {
        let hasher = Argon2Hasher;
        let broken = PasswordHash::from_stored("not-a-valid-hash".to_owned());
        assert!(!hasher.verify("0a1b", &broken));
    }
}
