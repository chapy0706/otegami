# Skill: Issue の読み方・着手フロー

このスキルは、`docs/issues/` 配下の Issue ファイルを起点に作業を開始するための手順を定義します。otegami(Rust + Axum + Askama)を例に書いています。

---

## 手順

### 1. Issue ファイルを読む

指定された Issue ファイルを読み、以下を把握してください。

- frontmatter の `status` : `open` のもののみ着手対象
- `## 概要・背景・目的` : 何をなぜするのか
- `## 受け入れ条件` : 何をもって完了とするか(チェックリスト)
- `## 技術的な検討事項` : 実装上の注意点
- `## 関連ADR・依存issue` : 参照すべき ADR と、先に完了が必要な Issue
- `## 想定工数・優先度`

### 2. 関連 ADR と依存 Issue を確認する

`## 関連ADR・依存issue` に記載された ADR(`docs/adr/`)を必ず読んでください。読まずに実装を始めてはいけません。

otegami では特に次の ADR が設計の背骨です。着手前に該当するものを読むこと。

- ADR-0001 セキュリティモデル(短いパスワード・slug の推測困難性・レート制限・単一エラー)
- ADR-0002 掃除バイナリの分離
- ADR-0003 TTL と burn の削除モデル
- ADR-0004 作成導線の Cloudflare Access 委譲

依存 Issue が `status: open` のままなら、着手前に人間へ確認してください。

### 3. 既存コードを確認する

変更対象の層に既存実装がある場合は、先に読んでから作業を始めてください。以下で絞り込んでから読むこと。

```sh
rg "対象の型名・関数名" crates/ apps/
tree crates apps
```

層の地図は次の通りです。

```
crates/domain          エンティティ・値オブジェクト・ポート(trait)・ドメインエラー
crates/application     ユースケース
crates/infrastructure  ポートの具象実装(Postgres・Argon2・slug・時計)
apps/web               Axum ハンドラ・Askama テンプレート・合成位置(main)
apps/cleaner           掃除バッチ
```

### 4. 影響範囲と変更計画を説明する

着手前に、以下を人間に説明して承認を得てください。

- 変更するファイル一覧
- 変更する理由
- テスト追加の方針
- `make verify` への影響

承認なしに実装を開始しないこと。

### 5. 該当スキルを読む

実装に入る前に `.claude/skills/` から該当スキルを読んでください。

| 実装内容 | 読むスキル |
|---|---|
| UseCase を新規作成する | 02-implement-usecase.md |
| Adapter(Repository / Hasher / SlugGenerator など)を作成する | 03-implement-adapter.md |
| Axum ハンドラ・Askama テンプレートを追加する | 04-implement-handler.md |
| Issue を閉じる | 05-close-issue.md |

---

## 注意

- `status: open` の Issue のみ着手対象です
- `status: closed` の Issue は変更しません
- Issue に記載のない設計判断が必要になった場合は実装を止め、人間に確認してください
- otegami の不変条件を壊す変更をしないこと。とりわけ次を守る。
  - 本文やパスワードをログに出力しない
  - 閲覧失敗は「見つからない」と「パスワード違い」を区別しない単一エラーに畳む
  - テンプレートで `|safe` を使わない(本文は必ず自動エスケープを通す)
  - domain にフレームワーク・DB・HTTP・時刻取得を持ち込まない
