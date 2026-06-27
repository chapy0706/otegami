use time::OffsetDateTime;

/// 時刻取得のポート。ユースケースは現在時刻を必ずこのポート経由で得る。
pub trait Clock: Send + Sync {
    fn now(&self) -> OffsetDateTime;
}
