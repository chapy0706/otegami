use std::fmt;

use thiserror::Error;

/// Crockford Base32 の文字集合。0-9 と、紛らわしい I/L/O/U を除いた英大文字。
const CROCKFORD_ALPHABET: &str = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

/// 公開ハンドル。手打ち・書き写しを前提に Crockford Base32 系の文字だけを許す。
///
/// 大文字小文字は区別しない(正規化して保持する)。長さは生成側の関心事であり、
/// ここでは文字集合の membership のみを検証する。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Slug(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SlugParseError {
    #[error("slug is empty")]
    Empty,
    #[error("slug contains a character outside the Crockford Base32 set")]
    InvalidCharacter,
}

impl Slug {
    /// 入力を大文字へ正規化し、Crockford 文字集合に属するかを検証する。
    pub fn parse(raw: &str) -> Result<Self, SlugParseError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(SlugParseError::Empty);
        }

        let normalized = trimmed.to_ascii_uppercase();
        if !normalized.chars().all(|c| CROCKFORD_ALPHABET.contains(c)) {
            return Err(SlugParseError::InvalidCharacter);
        }

        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 大文字の正常な_slug_を受理する() {
        let slug = Slug::parse("ABC234").unwrap();
        assert_eq!(slug.as_str(), "ABC234");
    }

    #[test]
    fn 小文字は大文字へ正規化する() {
        let slug = Slug::parse("abc234").unwrap();
        assert_eq!(slug.as_str(), "ABC234");
    }

    #[test]
    fn 前後の空白を無視する() {
        let slug = Slug::parse("  abc234 ").unwrap();
        assert_eq!(slug.as_str(), "ABC234");
    }

    #[test]
    fn crockford_から外れた文字は弾く() {
        // I / L / O / U は集合から除外されている。
        for raw in ["ABCI23", "ABCL23", "ABCO23", "ABCU23"] {
            assert_eq!(Slug::parse(raw), Err(SlugParseError::InvalidCharacter));
        }
    }

    #[test]
    fn 記号や非_ascii_は弾く() {
        assert_eq!(Slug::parse("AB-234"), Err(SlugParseError::InvalidCharacter));
        assert_eq!(Slug::parse("ABあ34"), Err(SlugParseError::InvalidCharacter));
    }

    #[test]
    fn 空文字は弾く() {
        assert_eq!(Slug::parse(""), Err(SlugParseError::Empty));
        assert_eq!(Slug::parse("   "), Err(SlugParseError::Empty));
    }
}
