# デプロイと配信

otegami を公開するための構成。配信は Cloudflare Tunnel、ホスティングは Coolify、作成導線の保護は Cloudflare Access に委ねる。アプリ内に認証コードは持たない(ADR-0004)。

関連: [実行時設定](configuration.md) / [cleaner の定期駆動](cleaner-scheduling.md)

## 全体像

```
ブラウザ
  └─ HTTPS ─> Cloudflare(エッジ)
                ├─ /create*  : Cloudflare Access で運用者に限定
                └─ それ以外   : 公開(/n/<slug> など)
                      └─ Cloudflare Tunnel ─> Coolify 上の otegami(web) ─> Postgres
                                              otegami(cleaner) は別途スケジュール駆動
```

経路の暗号化は Cloudflare Tunnel に委ねる。otegami 自体は TLS を終端しない。

## Coolify へのデプロイ

前提: Coolify 上に Postgres を用意し、接続情報を控える。

1. web 本体をアプリケーションとして登録する。
   - ビルド: ワークスペースから `cargo build --release -p web`、成果物は `target/release/web`。
   - 起動コマンド: `web`
   - 公開ポート: `8080`(`OTEGAMI_BIND_ADDR` で変更可)
2. 環境変数を設定する(最低限 `DATABASE_URL`。他は[設定表](configuration.md)参照)。
3. マイグレーションを適用する: `sqlx migrate run`(`make db/up`)を初回およびスキーマ変更時に実行。
4. cleaner は常駐させず、スケジュールタスクとして登録する([cleaner の定期駆動](cleaner-scheduling.md))。

## Cloudflare Tunnel

1. 対象ホストに `cloudflared` を導入し、Tunnel を作成する。
2. Public Hostname を otegami の web(例: `http://localhost:8080`、Coolify の内部アドレス)に向ける。
3. DNS を Tunnel に紐づけ、`https://<host>/` で到達できることを確認する。

Coolify が Cloudflare 連携を提供する場合はそれに従ってもよい。いずれの場合も、アプリは Tunnel の背後に置き、直接公開しない。

## Cloudflare Access(作成導線の保護)

作成系エンドポイント(`/create`、GET/POST)のみを Cloudflare Access で保護し、運用者本人に限定する。閲覧経路 `/n/<slug>` は公開のままにする。

1. Zero Trust ダッシュボード > Access > Applications で Self-hosted アプリを追加する。
2. Application domain を otegami のホストに、path を `/create` に設定する(作成導線だけを対象にする)。
3. Policy を作成し、運用者のメールアドレス(または IdP グループ)に Allow を与える。
4. それ以外(`/n/*` など)は Access の対象外とし、公開のままにする。

設定ミスは「作成不能」または「意図しない開放」に直結する。Access の設定は構成として管理し、変更は記録する。アプリ側にはこの認証に関するコードを置かない。

### 動作確認

- 未認証で `/create` を開くと Access のログイン画面に飛ぶ。
- 認証後は作成フォームが表示され、ノートを発行できる。
- `/n/<slug>` は未認証でも開け、パスワードで保護される。
