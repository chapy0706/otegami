use thiserror::Error;

/// ノート本文。素のテキストとして運ぶため、内容の解釈・整形はしない。
///
/// 空の本文は意味を持たないため弾くが、空白や改行はそのまま保持する
/// (貼られた文字をそのまま正確に運ぶ、という原則を崩さない)。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteBody(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoteBodyError {
    #[error("body must not be empty")]
    Empty,
}

impl NoteBody {
    pub fn parse(raw: impl Into<String>) -> Result<Self, NoteBodyError> {
        let body = raw.into();
        if body.is_empty() {
            return Err(NoteBodyError::Empty);
        }
        Ok(Self(body))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 通常の本文を受理する() {
        let body = NoteBody::parse("hello\nworld").unwrap();
        assert_eq!(body.as_str(), "hello\nworld");
    }

    #[test]
    fn 空白だけの本文は保持する() {
        let body = NoteBody::parse("   ").unwrap();
        assert_eq!(body.as_str(), "   ");
    }

    #[test]
    fn 空文字は弾く() {
        assert_eq!(NoteBody::parse(""), Err(NoteBodyError::Empty));
    }
}
