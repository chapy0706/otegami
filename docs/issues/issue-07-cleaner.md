---
status: open
created_at: 2026-06-27
closed_at:
---

# cleaner バイナリ: 失効分・閲覧済み burn 分の物理削除

## 概要・背景・目的

掃除を Web 本体から分離した otegami-cleaner として実装し、PurgeNotes を単発実行する。定期駆動は外部スケジューラに委ねる(ADR-0002)。

## 受け入れ条件

- [ ] apps/cleaner が PurgeNotes を1回実行して終了する(実行 → 件数ログ → exit 0)
- [ ] 具象(PgNoteRepository, SystemClock)を注入して PurgeNotes を駆動する
- [ ] `cargo run -p cleaner` と `make run/cleaner` で動く
- [ ] Web 本体(apps/web)と独立してビルド・実行できる
- [ ] 10分間隔の駆動例(cron / systemd-timer / Coolify スケジュール)を docs に記載
- [ ] 失敗時は非ゼロで終了する
- [ ] make verify が通る

## 技術的な検討事項

- アプリ内常駐タスクではなく独立バイナリにする(ADR-0002)
- 削除条件は PurgeNotes に集約し、cleaner は駆動だけを担う

## 関連ADR・依存issue

- 依存: issue-04(PurgeNotes), issue-05(具象)
- ADR: 0002, 0003
- スキル: 03-implement-adapter.md, 05-close-issue.md

## 想定工数・優先度

- 優先度: 中
- 工数: 小
