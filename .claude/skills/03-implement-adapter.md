# Skill: Adapter(Port の具象実装)の実装パターン

このスキルは、otegami における Infrastructure 層の Adapter 実装手順とパターンを定義します。Adapter は domain で定義された Port(trait)の具象実装です。

---

## Adapter の責務

- domain の Port(trait)を実装する
- SQLx(Postgres)・Argon2・乱数などの外部手段にアクセスする
- 外部の型を domain の型に変換する(境界での変換)
- 業務判断を持たない
- 失敗には文脈を添えて返す(`Adapter名.メソッド名: ...`)

---

## ファイル配置

```
crates/infrastructure/src/
├── pg/
│   ├── note_repository.rs
│   └── note_repository_it.rs       // 統合テスト
├── argon2_hasher.rs
├── crockford_slug.rs
└── system_clock.rs

migrations/
└── 20260624120000_create_notes.sql
```

---

## 実装テンプレート

### Repository 実装(pg/note_repository.rs)

DB 行を domain のエンティティへ変換するのが境界の仕事です。SQLx の型付きクエリ(`query!`)で列の型を保証し、値オブジェクトは `parse` を通してから組み立てます。

```rust
use async_trait::async_trait;
use sqlx::PgPool;
use time::OffsetDateTime;

use domain::entities::{Note, NoteSnapshot};
use domain::ports::{NoteRepository, RepositoryError};
use domain::value_objects::{PasswordHash, Slug};

pub struct PgNoteRepository {
    pool: PgPool,
}

impl PgNoteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

fn context(adapter_method: &str, e: impl std::fmt::Display) -> RepositoryError {
    RepositoryError(format!("{adapter_method}: {e}"))
}

#[async_trait]
impl NoteRepository for PgNoteRepository {
    async fn find_by_slug(&self, slug: &Slug) -> Result<Option<Note>, RepositoryError> {
        // expires_at で論理失効を表現する。物理削除(cleaner)前でも、
        // 期限切れは「無い」として扱う。
        let row = sqlx::query!(
            r#"
            SELECT slug, title, body, password_hash,
                   burn_after_view, viewed_at, expires_at, created_at
            FROM notes
            WHERE slug = $1 AND expires_at > now()
            "#,
            slug.as_str(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| context("PgNoteRepository.find_by_slug", e))?;

        let Some(row) = row else {
            return Ok(None);
        };

        // 境界での変換: DB 行 -> ドメインエンティティ
        let note = Note::restore(NoteSnapshot {
            slug: Slug::parse(&row.slug)
                .map_err(|e| context("PgNoteRepository.find_by_slug(slug)", e))?,
            title: row.title,
            body: row.body,
            password_hash: PasswordHash::from_stored(row.password_hash),
            burn_after_view: row.burn_after_view,
            viewed_at: row.viewed_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        });
        Ok(Some(note))
    }

    async fn mark_viewed(
        &self,
        slug: &Slug,
        at: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        // 初回のみ確定する。二度目以降は viewed_at を上書きしない。
        sqlx::query!(
            r#"UPDATE notes SET viewed_at = $2 WHERE slug = $1 AND viewed_at IS NULL"#,
            slug.as_str(),
            at,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| context("PgNoteRepository.mark_viewed", e))?;
        Ok(())
    }
}
```

### PasswordHasher 実装(argon2_hasher.rs)

domain の `PasswordHasher` と argon2 クレートのトレイト名が衝突するため、後者は `as _` で取り込みます。検証は CPU を使うため、ハンドラから呼ぶときは `tokio::task::spawn_blocking` に載せることを検討します。

```rust
use argon2::password_hash::{rand_core::OsRng, SaltString};
use argon2::{Argon2, PasswordHash as Argon2Hash, PasswordHasher as _, PasswordVerifier as _};

use domain::ports::PasswordHasher;
use domain::value_objects::PasswordHash;

pub struct Argon2Hasher;

impl PasswordHasher for Argon2Hasher {
    fn verify(&self, raw: &str, hash: &PasswordHash) -> bool {
        let Ok(parsed) = Argon2Hash::new(hash.as_str()) else {
            return false;
        };
        Argon2::default()
            .verify_password(raw.as_bytes(), &parsed)
            .is_ok()
    }
}

impl Argon2Hasher {
    // 作成時に使うハッシュ化。CreateNote 用の別 Port に置いてもよい。
    pub fn hash(&self, raw: &str) -> Result<PasswordHash, String> {
        let salt = SaltString::generate(&mut OsRng);
        let hash = Argon2::default()
            .hash_password(raw.as_bytes(), &salt)
            .map_err(|e| format!("Argon2Hasher.hash: {e}"))?
            .to_string();
        Ok(PasswordHash::from_stored(hash))
    }
}
```

### SlugGenerator 実装(crockford_slug.rs)

Crockford Base32(大文字小文字を区別せず I/L/O/U を除く 32 文字)から生成します。

```rust
use rand::Rng;

use domain::ports::SlugGenerator;
use domain::value_objects::Slug;

const ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";

pub struct CrockfordSlugGenerator {
    length: usize,
}

impl CrockfordSlugGenerator {
    pub fn new(length: usize) -> Self {
        Self { length }
    }
}

impl SlugGenerator for CrockfordSlugGenerator {
    fn generate(&self) -> Slug {
        let mut rng = rand::thread_rng();
        let s: String = (0..self.length)
            .map(|_| ALPHABET[rng.gen_range(0..ALPHABET.len())] as char)
            .collect();
        // 生成器は常に正しい文字集合で作るため、ここでの失敗は不変条件違反
        Slug::parse(&s).expect("generated slug must be valid")
    }
}
```

### マイグレーション(migrations/20260624120000_create_notes.sql)

外部に出るのは `slug` だけで、連番 `id` は秘匿します。RLS は使わず、アクセス制御は slug + パスワード + レート制限 + Cloudflare Access(作成導線)で担います。

```sql
CREATE TABLE notes (
    id              BIGSERIAL    PRIMARY KEY,
    slug            TEXT         NOT NULL UNIQUE,
    title           TEXT,
    body            TEXT         NOT NULL,
    password_hash   TEXT         NOT NULL,
    burn_after_view BOOLEAN      NOT NULL DEFAULT FALSE,
    viewed_at       TIMESTAMPTZ,
    expires_at      TIMESTAMPTZ  NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now()
);

CREATE INDEX idx_notes_slug ON notes (slug);
CREATE INDEX idx_notes_purge ON notes (expires_at);
```

---

## トランザクション境界の扱い方

複数テーブルを更新する場合は、Transactor Port を経由します(otegami は単一テーブルのため通常は不要)。

```rust
// domain/ports/transactor.rs
use async_trait::async_trait;

#[async_trait]
pub trait Transactor: Send + Sync {
    async fn with_tx<T, F>(&self, f: F) -> Result<T, RepositoryError>
    where
        F: /* TxContext を受け取るクロージャ */ Send;
}
```

---

## 統合テストテンプレート(`#[sqlx::test]`)

`#[sqlx::test]` は隔離されたテスト用 DB を用意し、`migrations/` を自動適用します。具象 Adapter の振る舞いを実 DB で確認します。

```rust
use sqlx::PgPool;

use domain::ports::NoteRepository;
use domain::value_objects::Slug;

use crate::pg::note_repository::PgNoteRepository;

#[sqlx::test]
async fn 期限切れのノートは見つからない(pool: PgPool) -> sqlx::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO notes (slug, body, password_hash, expires_at)
        VALUES ($1, 'x', '$argon2id$...', now() - interval '1 hour')
        "#,
        "ABC234",
    )
    .execute(&pool)
    .await?;

    let repo = PgNoteRepository::new(pool);
    let found = repo
        .find_by_slug(&Slug::parse("ABC234").unwrap())
        .await
        .unwrap();

    assert!(found.is_none());
    Ok(())
}
```

---

## 確認チェックリスト

- [ ] domain の Port(trait)を正しく実装している
- [ ] DB 行を型付きクエリで受け、値オブジェクトは `parse` を通して domain の型へ変換している
- [ ] エラーが `Adapter名.メソッド名: ...` の形式で文脈付きになっている
- [ ] 業務判断を Adapter 内に書いていない
- [ ] `find_by_slug` が `expires_at > now()` で論理失効を反映している
- [ ] `mark_viewed` が初回のみ(`viewed_at IS NULL`)で確定している
- [ ] マイグレーションが追加され、`slug` に UNIQUE と索引、`expires_at` に索引がある
- [ ] 統合テストが追加されている(または TODO として Issue に残している)
- [ ] `make verify` が通る
