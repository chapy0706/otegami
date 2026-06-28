use time::OffsetDateTime;

use domain::ports::Clock;

/// 実時刻を返す Clock の具象。テストでは固定 Clock に差し替える。
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}
