# Skill: Issue 完了の確認フロー

このスキルは、Issue の受け入れ条件を確認して閉じるための手順を定義します。

---

## 完了確認の手順

### 1. make verify を実行する

```sh
make verify
```

`fmt/check`(整形)→ `lint`(clippy・警告エラー扱い)→ `test` の順で走ります。失敗した場合は自分で修正して再実行してください。3 回試みても通らない場合は、失敗ログを添えて人間に報告してください。

### 2. 受け入れ条件を一つずつ確認する

Issue ファイルの `## 受け入れ条件` に記載されたチェックリストを順番に確認してください。

```
- [ ] ViewNote UseCase が実装されている
- [ ] UseCase が HTTP / DB に依存していない
- [ ] 閲覧失敗が単一エラーに畳まれている
- [ ] 単体テストが追加されている
- [ ] make verify がエラーを発生させない
```

すべてチェックが入ったことを確認してから次のステップに進んでください。

### 3. Issue ファイルの frontmatter を更新する

Issue ファイル先頭の frontmatter の `status: open` を `status: closed` に変更します。また `closed_at` に完了日時を記入してください。

```md
---
status: closed
created_at: YYYY-MM-DD
closed_at: YYYY-MM-DD
---
```

### 4. 完了報告を人間に行う

以下の形式で完了報告を行ってください。

```
## 完了報告

Issue: docs/issues/issue-XX-name.md

### 実装した内容
- ...

### 変更したファイル
- crates/application/src/use_cases/view_note.rs(新規)
- crates/domain/src/errors.rs(更新)

### make verify の結果
verify passed.

### 未解決の事項(次 Issue に引き継ぐもの)
- (あれば記載)
```

---

## 注意

- `make verify` が通らない状態で Issue を閉じてはいけません
- 受け入れ条件を一部スキップする場合は、理由を添えて人間に確認を求めてください
- 未解決の事項は必ず次の Issue として切り出すか、Issue 内に TODO として残してください
- 閉じる前に otegami の不変条件を最後に確認すること。
  - テンプレートで `|safe` を使っていないか
  - 本文・パスワードをログに出していないか
  - 閲覧失敗を単一エラー/単一応答に畳めているか
  - domain にフレームワーク・DB・HTTP・時刻取得が漏れ込んでいないか
