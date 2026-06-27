use thiserror::Error;

/// ノートのタイトル。Note 上では任意だが、値として存在するなら空ではない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteTitle(String);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum NoteTitleError {
    #[error("title must not be empty")]
    Empty,
}

impl NoteTitle {
    pub fn parse(raw: impl Into<String>) -> Result<Self, NoteTitleError> {
        let title = raw.into();
        if title.is_empty() {
            return Err(NoteTitleError::Empty);
        }
        Ok(Self(title))
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
    fn 通常のタイトルを受理する() {
        let title = NoteTitle::parse("memo").unwrap();
        assert_eq!(title.as_str(), "memo");
    }

    #[test]
    fn 空文字は弾く() {
        assert_eq!(NoteTitle::parse(""), Err(NoteTitleError::Empty));
    }
}
