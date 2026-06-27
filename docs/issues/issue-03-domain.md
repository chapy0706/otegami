---
status: open
created_at: 2026-06-27
closed_at:
---

# ドメイン層: エンティティ・値オブジェクト・ポート・エラー

## 概要・背景・目的

otegami の中心となる domain crate を実装する。フレームワーク・DB・HTTP を一切知らない純粋な層として、Note エンティティ、値オブジェクト、ポート(trait)、エラーを定義し、以降の層が依存する内側を固める。

## 受け入れ条件

- [ ] Note エンティティ(slug, title, body, password_hash, burn_after_view, viewed_at, expires_at, created_at)とアクセサ、`Note::restore(NoteSnapshot)`、`#[derive(Clone)]`
- [ ] 値オブジェクト: Slug(Crockford Base32・大小無視・parse で検証), PasswordHash(from_stored / as_str), RawPassword(半角英数1〜4文字の検証), NoteBody, NoteTitle
- [ ] ポート(trait): NoteRepository, PasswordHasher(hash / verify), SlugGenerator, Clock
- [ ] エラー: ViewNoteError(NotFoundOrWrongPassword / Unexpected), CreateNoteError, RepositoryError を thiserror で定義
- [ ] domain crate が axum / sqlx / askama に依存していない(Cargo.toml で確認)
- [ ] 値オブジェクトの単体テスト(Slug の正常/異常、RawPassword の長さ・文字種)
- [ ] make verify が通る

## 技術的な検討事項

- async を持つ trait は async-trait を使う
- Slug::parse は大文字へ正規化し、Crockford 文字集合の membership を検証する
- domain の依存は最小(time, thiserror, async-trait 程度)に留める

## 関連ADR・依存issue

- 依存: issue-02
- ADR: 0001(単一エラー・パスワード制約), 0003(TTL / burn のフィールド)
- スキル: 02-implement-usecase.md(ポートの置き方)

## 想定工数・優先度

- 優先度: 高
- 工数: 中
