---
status: open
created_at: 2026-06-27
closed_at:
---

# 横断的関心: レート制限・Cloudflare Access・設定・デプロイ

## 概要・背景・目的

短いパスワードを成立させるレート制限、作成導線の保護、実行時設定の読み込み、Coolify へのデプロイを整える。セキュリティモデルの前提条件を満たし、公開可能な状態にする。

## 受け入れ条件

- [ ] 閲覧 POST(`/n/:slug`)に tower-governor 等で IP・slug 単位のレート制限(既定: 5回/10分、超過でロック)
- [ ] 設定(DATABASE_URL, 既定TTL, slug 長, パスワード最大長, レート制限閾値, バッチ間隔)を config.toml か環境変数から読み込む
- [ ] `/create` を Cloudflare Access の背後に置く(手順を docs に記載)。アプリ側に認証コードを持ち込まない
- [ ] Cloudflare Tunnel 経由で配信し、Coolify にデプロイする
- [ ] cleaner の定期駆動をスケジューラに登録する
- [ ] make verify が通る

## 技術的な検討事項

- レート制限はセキュリティモデルの成立条件(ADR-0001)。欠かさない
- Access はアプリ外の設定。bootstrap・コードの管轄外で、手順を文書化する
- 実行時設定は `.bootstrap.toml` とは別物(design-spec 10章)

## 関連ADR・依存issue

- 依存: issue-06, issue-07
- ADR: 0001, 0004
- スキル: 04-implement-handler.md

## 想定工数・優先度

- 優先度: 高(公開の前提)
- 工数: 中
