# Skill: UseCase の実装パターン

このスキルは、otegami における UseCase(Application 層)の実装手順とパターンを定義します。

---

## UseCase の責務

- ドメインルールを適用して処理を進める
- Port(trait)を呼び出す
- HTTP・DB・フレームワーク・時刻取得の実装に依存しない
- SQL を直接書かない
- 失敗は例外ではなく `Result<_, XxxError>` で返す
- 副作用(DB 書き込み・外部呼び出し)を UseCase 内で完結させる

---

## ファイル配置

Port interface(trait)は利用側、すなわち domain に置きます。

```
crates/
├── domain/src/
│   ├── entities/note.rs
│   ├── value_objects/slug.rs
│   ├── value_objects/password.rs
│   ├── errors.rs                 // ドメイン/ユースケースのエラー定義
│   └── ports/
│       ├── note_repository.rs    // Port(trait)
│       ├── password_hasher.rs
│       └── clock.rs
├── application/src/
│   └── use_cases/
│       └── view_note.rs          // UseCase 本体 + 単体テスト
└── infrastructure/src/
    ├── pg/note_repository.rs     // Port の実装(Adapter)
    ├── argon2_hasher.rs
    └── system_clock.rs
```

---

## 実装テンプレート

### Port trait(domain/ports/note_repository.rs)

`dyn` で注入するため `Send + Sync` を要求し、非同期メソッドには `#[async_trait]` を使います。

```rust
use async_trait::async_trait;
use time::OffsetDateTime;

use crate::entities::Note;
use crate::value_objects::Slug;

#[derive(Debug)]
pub struct RepositoryError(pub String);

#[async_trait]
pub trait NoteRepository: Send + Sync {
    async fn find_by_slug(&self, slug: &Slug) -> Result<Option<Note>, RepositoryError>;
    async fn mark_viewed(&self, slug: &Slug, at: OffsetDateTime) -> Result<(), RepositoryError>;
}
```

### Clock trait(domain/ports/clock.rs)

時刻は必ず Clock ポート経由で取得します。`OffsetDateTime::now_utc()` を UseCase から直接呼ばないこと。

```rust
use time::OffsetDateTime;

pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}
```

### PasswordHasher trait(domain/ports/password_hasher.rs)

```rust
use crate::value_objects::PasswordHash;

pub trait PasswordHasher: Send + Sync {
    fn verify(&self, raw: &str, hash: &PasswordHash) -> bool;
}
```

### ドメインエラー(domain/errors.rs)

閲覧失敗は「見つからない」と「パスワード違い」を区別しない単一の値に畳みます(ADR-0001)。理由を持たせないこと自体が設計です。

```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ViewNoteError {
    #[error("not found or wrong password")]
    NotFoundOrWrongPassword,
    #[error("unexpected error")]
    Unexpected,
}
```

### UseCase 本体(application/use_cases/view_note.rs)

```rust
use std::sync::Arc;

use domain::errors::ViewNoteError;
use domain::ports::{Clock, NoteRepository, PasswordHasher};
use domain::value_objects::Slug;

pub struct ViewNoteInput {
    pub raw_slug: String,
    pub password: String,
}

pub struct ViewNoteOutput {
    pub title: Option<String>,
    pub body: String,
}

pub struct ViewNote {
    notes: Arc<dyn NoteRepository>,
    hasher: Arc<dyn PasswordHasher>,
    clock: Arc<dyn Clock>,
}

impl ViewNote {
    pub fn new(
        notes: Arc<dyn NoteRepository>,
        hasher: Arc<dyn PasswordHasher>,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { notes, hasher, clock }
    }

    pub async fn execute(
        &self,
        input: ViewNoteInput,
    ) -> Result<ViewNoteOutput, ViewNoteError> {
        // 1. slug の正規化。不正な形は「無い」と同じ扱いに畳む
        let slug = Slug::parse(&input.raw_slug)
            .map_err(|_| ViewNoteError::NotFoundOrWrongPassword)?;

        // 2. 取得(Port 経由)。存在しなくても理由は明かさない
        let note = self
            .notes
            .find_by_slug(&slug)
            .await
            .map_err(|_| ViewNoteError::Unexpected)?
            .ok_or(ViewNoteError::NotFoundOrWrongPassword)?;

        // 3. パスワード照合。失敗も同じ単一エラーに畳む
        if !self.hasher.verify(&input.password, note.password_hash()) {
            return Err(ViewNoteError::NotFoundOrWrongPassword);
        }

        // 4. 初回閲覧なら viewed_at を記録(Clock 経由)
        if note.viewed_at().is_none() {
            self.notes
                .mark_viewed(&slug, self.clock.now())
                .await
                .map_err(|_| ViewNoteError::Unexpected)?;
        }

        // 5. 本文はそのまま返す。エスケープは描画側(Askama)に委ねる
        Ok(ViewNoteOutput {
            title: note.title().map(str::to_owned),
            body: note.body().to_owned(),
        })
    }
}
```

---

## 実装ルール

- UseCase の公開メソッドは `async fn execute(&self, input: XxxInput) -> Result<XxxOutput, XxxError>` に統一する
- 依存(Port)はコンストラクタ(`new`)で `Arc<dyn Port>` として受け取る。UseCase 内で具象を組み立てない
- ドメイン/ユースケースのエラーは `domain/errors.rs` に定義し、`Result` で返す。`panic!`・`unwrap`・`expect` を業務経路で使わない
- 時刻処理は Clock ポート経由にする
- 複数テーブルを更新する場合は、トランザクション用の Port(Transactor)を経由する
- 失敗理由を不必要に細分化しない(単一エラーへ畳む設計を崩さない)

---

## テストテンプレート(fake で Port を差し替える)

具象(Postgres・Argon2)に依存させず、trait の fake 実装でテストします。

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Mutex;
    use time::OffsetDateTime;

    use domain::entities::{Note, NoteSnapshot};
    use domain::ports::{Clock, NoteRepository, PasswordHasher, RepositoryError};
    use domain::value_objects::{PasswordHash, Slug};

    struct FakeNotes {
        note: Option<Note>,
        viewed: Mutex<bool>,
    }

    #[async_trait]
    impl NoteRepository for FakeNotes {
        async fn find_by_slug(&self, _slug: &Slug) -> Result<Option<Note>, RepositoryError> {
            Ok(self.note.clone())
        }
        async fn mark_viewed(&self, _slug: &Slug, _at: OffsetDateTime) -> Result<(), RepositoryError> {
            *self.viewed.lock().unwrap() = true;
            Ok(())
        }
    }

    struct FakeHasher {
        accepts: bool,
    }
    impl PasswordHasher for FakeHasher {
        fn verify(&self, _raw: &str, _hash: &PasswordHash) -> bool {
            self.accepts
        }
    }

    struct FixedClock(OffsetDateTime);
    impl Clock for FixedClock {
        fn now(&self) -> OffsetDateTime {
            self.0
        }
    }

    fn sample_note() -> Note {
        Note::restore(NoteSnapshot {
            slug: Slug::parse("ABC234").unwrap(),
            title: None,
            body: "hello".to_owned(),
            password_hash: PasswordHash::from_stored("$argon2id$...".to_owned()),
            burn_after_view: false,
            viewed_at: None,
            expires_at: OffsetDateTime::now_utc(),
            created_at: OffsetDateTime::now_utc(),
        })
    }

    #[tokio::test]
    async fn 正しいパスワードなら本文を返し_viewed_at_を立てる() {
        let notes = Arc::new(FakeNotes { note: Some(sample_note()), viewed: Mutex::new(false) });
        let view = ViewNote::new(
            notes.clone(),
            Arc::new(FakeHasher { accepts: true }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let out = view
            .execute(ViewNoteInput { raw_slug: "ABC234".into(), password: "0a1b".into() })
            .await
            .unwrap();

        assert_eq!(out.body, "hello");
        assert!(*notes.viewed.lock().unwrap());
    }

    #[tokio::test]
    async fn パスワード違いは単一エラーに畳む() {
        let view = ViewNote::new(
            Arc::new(FakeNotes { note: Some(sample_note()), viewed: Mutex::new(false) }),
            Arc::new(FakeHasher { accepts: false }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let err = view
            .execute(ViewNoteInput { raw_slug: "ABC234".into(), password: "zzzz".into() })
            .await
            .unwrap_err();

        assert!(matches!(err, ViewNoteError::NotFoundOrWrongPassword));
    }

    #[tokio::test]
    async fn 存在しない_slug_もパスワード違いと同じエラー() {
        let view = ViewNote::new(
            Arc::new(FakeNotes { note: None, viewed: Mutex::new(false) }),
            Arc::new(FakeHasher { accepts: true }),
            Arc::new(FixedClock(OffsetDateTime::now_utc())),
        );

        let err = view
            .execute(ViewNoteInput { raw_slug: "ABC234".into(), password: "0a1b".into() })
            .await
            .unwrap_err();

        assert!(matches!(err, ViewNoteError::NotFoundOrWrongPassword));
    }
}
```

`Note` は fake と Adapter の双方から復元できるよう、`Note::restore(NoteSnapshot)` のような復元コンストラクタを domain に用意します。`#[derive(Clone)]` も付けておくと fake が書きやすくなります。

---

## 確認チェックリスト

- [ ] UseCase が HTTP / DB / フレームワーク / 時刻取得に直接依存していない
- [ ] 依存(Port)をコンストラクタで `Arc<dyn Port>` として受け取っている
- [ ] 時刻処理が Clock ポート経由になっている
- [ ] エラーが `domain/errors.rs` に定義され、`Result` で返っている(業務経路で `unwrap`/`panic!` を使っていない)
- [ ] 閲覧失敗が単一エラーに畳まれている(存在有無を漏らしていない)
- [ ] 単体テストが追加されている(Port は fake で差し替え)
- [ ] `make verify` が通る
