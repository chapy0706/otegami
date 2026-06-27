---
status: open
created_at: 2026-06-27
closed_at:
---

# Web プレゼンテーション: Axum ハンドラ・Askama テンプレート・合成

## 概要・背景・目的

作成と閲覧の導線を Axum + Askama で実装し、ユースケースと具象を AppState で結線して、ブラウザから一連の流れが動く状態にする。

## 受け入れ条件

- [ ] AppState に CreateNote / ViewNote を載せ、main.rs で具象を `Arc<dyn Port>` に包んで注入
- [ ] ルート: GET/POST `/create`(作成), GET `/n/:slug`(パスワードフォーム), POST `/n/:slug`(閲覧)
- [ ] テンプレート(.html): create / password_form / note / error。`{{ }}` の自動エスケープに委ね、`|safe` を使わない
- [ ] 作成成功で短い URL(`/n/<slug>`)を提示する
- [ ] 閲覧失敗は単一応答(見つからない / パスワード違いを区別しない)
- [ ] 描画は `render() -> Html(String)`(統合クレートに依存しない)
- [ ] 本文・パスワードをログに出さない
- [ ] make verify が通る

## 技術的な検討事項

- 作成導線の認証はアプリに持ち込まない(Cloudflare Access、issue-08)。ハンドラは保護済みを前提とする
- argon2 verify を spawn_blocking に載せるか検討する
- 設定(TTL・slug 長・パスワード長)は AppState 経由で渡す

## 関連ADR・依存issue

- 依存: issue-04, issue-05
- ADR: 0001, 0004
- スキル: 04-implement-handler.md

## 想定工数・優先度

- 優先度: 高
- 工数: 中〜大
