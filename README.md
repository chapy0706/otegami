# otegami

お手紙のように、テキストを一度だけ託して、別の場所で受け取る。最小構成のテキスト共有サービス。

## これは何

自分のアカウントにログインできない貸与PCなどから、作業途中のコードやAI出力といったテキストを別の環境へ持ち出したいことがある。チャットに貼るには長く、ツールも入れられない。

otegami は「テキストを貼る → 短いURLとパスワードを受け取る → 別の場所でそれを開く」という、託して受け取るだけの導線を提供する。運び手に徹し、本文を改変も解釈もせず、そのままの文字として届ける。

## 特徴

- 短いURL。6文字の slug を発行する。手打ちと記憶を前提に、書き写しに強い文字集合(Crockford Base32 系、大文字小文字の区別なし)を使う。短く出せることを必須機能とする
- 軽いパスワード。半角英数1〜4文字。短さは手打ちのための割り切りで、安全は推測困難な slug とレート制限で担保する
- 作成から1日で自動失効。この1日が共通の安全網であり、その間は何度でも読み返せる
- 「閲覧後に削除」は任意。読み返す必要のない手紙にだけ指定すると、閲覧後の掃除で消える
- 本文は素のテキストとして安全に保管・表示する。スクリプトを混ぜられても実行されず、ただの文字として描かれる
- 作成できるのは運用者だけ(Cloudflare Access)。閲覧はURLとパスワードを知る人なら誰でも

## 使い方

作成(運用者のみ):

1. 本文を貼り、パスワードを決める(任意でタイトルと「閲覧後に削除」)
2. 発行された `https://<host>/n/<slug>` を控える

閲覧(URLとパスワードを知る人):

1. `/n/<slug>` を開き、パスワードを入力する
2. 一致すれば本文が表示される。失敗時は「見つからないか、パスワードが違う」とだけ返る

## 設計の要点

二つの原則を背骨に置く。

- 主体性をユーザーに返す。残す・消す・守るの判断は作成者が握る
- 認識は丁寧に正確に渡す。貼られた文字をそのまま正確に運ぶ

クリーンアーキテクチャに沿い、内側(ドメイン)が外側(Web/DB)を知らない依存方向を保つ。Web 本体と掃除バッチは別バイナリに分け、責務と駆動を疎結合にする。

- `otegami` — Web 本体(作成・閲覧)
- `otegami-cleaner` — 失効分と閲覧済み burn 分を掃除するバッチ

詳しい設計判断とセキュリティの考え方は `docs/design-spec.md` に置く。

## 技術スタック

- 言語: Rust
- Web: Axum(tower / tower-http のミドルウェア合成)
- テンプレート: Askama(コンパイル時生成・自動エスケープ)
- DB: PostgreSQL + SQLx
- パスワードハッシュ: argon2
- slug 生成: Crockford Base32 系(6文字)
- レート制限: tower-governor 等
- ログ: tracing
- 作成導線の認証: Cloudflare Access
- 配信・暗号化: Cloudflare Tunnel
- デプロイ: Coolify(A1)

## ディレクトリ構成(予定)

```
otegami/
  Cargo.toml              # workspace
  bootstrap.toml          # 設定の単一の入口
  crates/
    domain/               # エンティティ・値オブジェクト・ドメインエラー
    application/          # ユースケースとポート(trait)
    infrastructure/       # Postgres・Argon2・slug・時計の実装
  apps/
    web/                  # otegami 本体(Axum ハンドラ + main)
    cleaner/              # otegami-cleaner バッチ
  templates/              # Askama テンプレート
  migrations/             # SQLx マイグレーション
  docs/
    README.md
    design-spec.md
    directory-structure.md
    glossary.md
    adr/
```

## 開発(これから整備)

前提は Rust と PostgreSQL。設定は `bootstrap.toml` に集約し、起動・マイグレーション・バッチ実行の手順は以降の issue で整える。想定する流れは次の通り。

```
# マイグレーション適用
sqlx migrate run

# Web 本体を起動
cargo run -p web

# 掃除バッチを単発実行(cron / systemd-timer から定期駆動)
cargo run -p cleaner
```

## ドキュメント構成

`docs/` は三層で構成する。

- README.md … 入口。何であるか、どう使うか
- design-spec.md … 設計判断とセキュリティの根拠
- directory-structure.md … 構成の意図
- glossary.md … 用語
- adr/ … 設計思想の決定を一枚ずつ独立記録

## ステータス

設計フェーズ。design-spec の確定を経て本書(README)を整備した段階。以降は ADR、bootstrap.toml、docs/issues の順で進める。
