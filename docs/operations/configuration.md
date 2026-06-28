# 実行時設定

otegami(web / cleaner)は起動時に設定を読む。これは `.bootstrap.toml`(project-bootstrap への指示書)とは別物で、アプリ自身の動作値である(design-spec 10章)。

## 読み込み順

1. 既定値(コード内)
2. `config.toml`(任意。パスは環境変数 `OTEGAMI_CONFIG` で変更可、既定 `config.toml`)
3. 環境変数(最優先)

ファイルが無くても、環境変数と既定値だけで起動する。雛形は `config.example.toml`。

## 項目

| config.toml キー | 環境変数 | 既定値 | 用途 |
| --- | --- | --- | --- |
| `database_url` | `DATABASE_URL` | (必須) | Postgres 接続情報 |
| `ttl_days` | `OTEGAMI_TTL_DAYS` | `1` | 既定 TTL(日) |
| `slug_length` | `OTEGAMI_SLUG_LENGTH` | `6` | slug 長(文字) |
| `password_max_len` | `OTEGAMI_PASSWORD_MAX_LEN` | `4` | パスワード最大長(文字) |
| `rate_limit_max` | `OTEGAMI_RATE_LIMIT_MAX` | `5` | レート制限: 期間内の試行回数 |
| `rate_limit_period_secs` | `OTEGAMI_RATE_LIMIT_PERIOD_SECS` | `600` | レート制限: 期間(秒) |
| `batch_interval_secs` | `OTEGAMI_BATCH_INTERVAL_SECS` | `600` | 掃除バッチ間隔(秒、目安) |
| `bind_addr` | `OTEGAMI_BIND_ADDR` | `0.0.0.0:8080` | web 待ち受けアドレス |

`DATABASE_URL` は秘密を含むため、本番では config.toml に直書きせず環境変数で与えることを推奨する(秘密はコミットしない)。

## レート制限について

閲覧 POST(`/n/:slug`)に IP+slug 単位で課す。短いパスワードを許容するための前提条件であり、欠かしてはならない(ADR-0001)。送信元 IP は Cloudflare Tunnel 経由を想定し、`CF-Connecting-IP` → `X-Forwarded-For` 先頭 → 接続元 の順で判定する。超過時は 429 を返す。
