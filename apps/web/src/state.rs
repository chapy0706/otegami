use std::sync::Arc;

use time::Duration;

use application::use_cases::{CreateNote, ViewNote};

/// 実行時設定。合成位置(main)で組み立て、AppState 経由でハンドラに渡す。
/// 本来は bootstrap.toml / 環境から読む(外部化は issue-08)。
#[derive(Clone)]
pub struct Config {
    /// ノートの既定 TTL(ADR-0003)。CreateNote の失効時刻算出に使う。
    pub ttl: Duration,
    /// 発行する slug の長さ。
    pub slug_length: usize,
    /// 閲覧パスワードの最大長(ドメインの不変条件は 1〜4。表示・入力制限に使う)。
    pub password_max_len: usize,
}

/// アプリ全体で共有する状態。UseCase は具象を `Arc<dyn Port>` で内包済み。
#[derive(Clone)]
pub struct AppState {
    pub create_note: Arc<CreateNote>,
    pub view_note: Arc<ViewNote>,
    pub config: Config,
}
