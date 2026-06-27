---
status: open
created_at: 2026-06-27
closed_at:
---

# インフラ層: Postgres リポジトリ・Argon2・Crockford slug・時計・マイグレーション

## 概要・背景・目的

domain のポートを具象実装する。Postgres へのアクセス、パスワードハッシュ、slug 生成、時刻を提供し、ユースケースを実際に動かせる状態にする。

## 受け入れ条件

- [ ] migrations/ に notes テーブル作成(design-spec の DDL、slug UNIQUE、expires_at 索引)
- [ ] PgNoteRepository: insert / find_by_slug(`expires_at > now()` で論理失効)/ mark_viewed(初回のみ)/ purge
- [ ] DB 行 → ドメインへの境界変換(Slug::parse 等)
- [ ] エラーは「Adapter名.メソッド名: ...」の文脈付き
- [ ] Argon2Hasher(hash / verify、トレイト名衝突を `as _` で解決)
- [ ] CrockfordSlugGenerator(32 文字・設定長)
- [ ] SystemClock
- [ ] `#[sqlx::test]` による統合テスト(失効ノートは見つからない、insert → find)
- [ ] infrastructure crate にのみ sqlx / argon2 / rand が入り、domain には漏れない
- [ ] make verify が通る

## 技術的な検討事項

- argon2 verify は CPU 律速。ハンドラから呼ぶ場合は spawn_blocking を検討(issue-06 で対応可)
- sqlx は offline モード(.sqlx/)を用意し、型検査を CI でも効かせる(make db/prepare)
- purge は一文の DELETE(`expires_at < now() OR (burn_after_view AND viewed_at IS NOT NULL)`)

## 関連ADR・依存issue

- 依存: issue-03(ポート)。issue-04 と並行可
- ADR: 0001, 0003
- スキル: 03-implement-adapter.md

## 想定工数・優先度

- 優先度: 高
- 工数: 中〜大
