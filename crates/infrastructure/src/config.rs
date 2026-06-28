//! 実行時設定の読み込み。
//!
//! design-spec 10章。`.bootstrap.toml`(project-bootstrap への指示書)とは別物で、
//! アプリ自身が起動時に読む値。`config.toml`(任意)を土台に、環境変数で上書きする。
//! 外部入力の境界なので、不正値はここで弾く。

use std::fmt::Display;
use std::str::FromStr;

use serde::Deserialize;

/// 設定読み込みの失敗。
#[derive(Debug, thiserror::Error)]
#[error("config error: {0}")]
pub struct ConfigError(pub String);

/// アプリの実行時設定。web / cleaner の合成位置で使う。
#[derive(Debug, Clone)]
pub struct Settings {
    pub database_url: String,
    /// 既定 TTL(日)。初期値1日。
    pub ttl_days: i64,
    /// slug 長。初期値6文字。
    pub slug_length: usize,
    /// パスワード最大長。初期値4文字(ドメインの不変条件は 1〜4)。
    pub password_max_len: usize,
    /// レート制限: 期間内に許す試行回数。初期値5回。
    pub rate_limit_max: u32,
    /// レート制限: 期間(秒)。初期値600秒(10分)。
    pub rate_limit_period_secs: u64,
    /// 掃除バッチの駆動間隔(秒)。初期値600秒(10分)。cleaner では情報目的。
    pub batch_interval_secs: u64,
    /// web の待ち受けアドレス。初期値 0.0.0.0:8080。
    pub bind_addr: String,
}

/// config.toml の生の形。すべて任意で、欠けたものは既定値・環境変数で埋める。
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct FileSettings {
    database_url: Option<String>,
    ttl_days: Option<i64>,
    slug_length: Option<usize>,
    password_max_len: Option<usize>,
    rate_limit_max: Option<u32>,
    rate_limit_period_secs: Option<u64>,
    batch_interval_secs: Option<u64>,
    bind_addr: Option<String>,
}

impl Settings {
    /// config.toml(任意)→ 環境変数の順に解決する。環境変数が最優先。
    ///
    /// 設定ファイルのパスは `OTEGAMI_CONFIG`(既定 `config.toml`)。
    /// ファイルが無ければ既定値と環境変数だけで構成する。
    pub fn load() -> Result<Self, ConfigError> {
        let path = std::env::var("OTEGAMI_CONFIG").unwrap_or_else(|_| "config.toml".to_owned());
        let file: FileSettings = match std::fs::read_to_string(&path) {
            Ok(contents) => {
                toml::from_str(&contents).map_err(|e| ConfigError(format!("{path}: {e}")))?
            }
            // ファイル不在は許容する(既定値 + 環境変数で動く)。
            Err(_) => FileSettings::default(),
        };

        let database_url = env_string("DATABASE_URL")
            .or(file.database_url)
            .ok_or_else(|| {
                ConfigError("DATABASE_URL must be set (env or config.toml)".to_owned())
            })?;

        Ok(Self {
            database_url,
            ttl_days: env_parse("OTEGAMI_TTL_DAYS")?
                .or(file.ttl_days)
                .unwrap_or(1),
            slug_length: env_parse("OTEGAMI_SLUG_LENGTH")?
                .or(file.slug_length)
                .unwrap_or(6),
            password_max_len: env_parse("OTEGAMI_PASSWORD_MAX_LEN")?
                .or(file.password_max_len)
                .unwrap_or(4),
            rate_limit_max: env_parse("OTEGAMI_RATE_LIMIT_MAX")?
                .or(file.rate_limit_max)
                .unwrap_or(5),
            rate_limit_period_secs: env_parse("OTEGAMI_RATE_LIMIT_PERIOD_SECS")?
                .or(file.rate_limit_period_secs)
                .unwrap_or(600),
            batch_interval_secs: env_parse("OTEGAMI_BATCH_INTERVAL_SECS")?
                .or(file.batch_interval_secs)
                .unwrap_or(600),
            bind_addr: env_string("OTEGAMI_BIND_ADDR")
                .or(file.bind_addr)
                .unwrap_or_else(|| "0.0.0.0:8080".to_owned()),
        })
    }
}

/// 空文字は「未設定」として扱う。
fn env_string(key: &str) -> Option<String> {
    std::env::var(key).ok().filter(|s| !s.is_empty())
}

/// 環境変数を型へパースする。空・未設定は `None`、不正値はエラー。
fn env_parse<T>(key: &str) -> Result<Option<T>, ConfigError>
where
    T: FromStr,
    T::Err: Display,
{
    match env_string(key) {
        Some(v) => v
            .parse::<T>()
            .map(Some)
            .map_err(|e| ConfigError(format!("{key}: {e}"))),
        None => Ok(None),
    }
}
