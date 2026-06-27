use thiserror::Error;

/// 閲覧パスワードの平文。半角英数1〜4文字に制限する(ADR-0001)。
///
/// 短さは手打ちの利便のための割り切りであり、安全は推測困難な slug と
/// レート制限で担保する。平文は保持・保存せず、ハッシュ化の入口でのみ扱う。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawPassword(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum RawPasswordError {
    #[error("password must be between 1 and 4 characters")]
    InvalidLength,
    #[error("password must contain only half-width alphanumeric characters")]
    InvalidCharacter,
}

impl RawPassword {
    pub fn parse(raw: &str) -> Result<Self, RawPasswordError> {
        let len = raw.chars().count();
        if !(1..=4).contains(&len) {
            return Err(RawPasswordError::InvalidLength);
        }
        if !raw.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(RawPasswordError::InvalidCharacter);
        }
        Ok(Self(raw.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// 保存済みのパスワードハッシュ(Argon2)。検証は照合側に委ね、ここでは保持に徹する。
#[derive(Debug, Clone)]
pub struct PasswordHash(String);

impl PasswordHash {
    /// 保存済み文字列(DB から復元した値や Hasher の出力)から組み立てる。
    pub fn from_stored(stored: String) -> Self {
        Self(stored)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 半角英数1から4文字を受理する() {
        for raw in ["a", "1", "0a1b", "Z9zZ"] {
            assert_eq!(RawPassword::parse(raw).unwrap().as_str(), raw);
        }
    }

    #[test]
    fn 空文字は長さで弾く() {
        assert_eq!(RawPassword::parse(""), Err(RawPasswordError::InvalidLength));
    }

    #[test]
    fn 五文字以上は長さで弾く() {
        assert_eq!(
            RawPassword::parse("abcde"),
            Err(RawPasswordError::InvalidLength)
        );
    }

    #[test]
    fn 記号や全角は文字種で弾く() {
        assert_eq!(
            RawPassword::parse("a-b"),
            Err(RawPasswordError::InvalidCharacter)
        );
        assert_eq!(
            RawPassword::parse("ぱ"),
            Err(RawPasswordError::InvalidCharacter)
        );
    }

    #[test]
    fn from_stored_は値をそのまま保持する() {
        let hash = PasswordHash::from_stored("$argon2id$abc".to_owned());
        assert_eq!(hash.as_str(), "$argon2id$abc");
    }
}
