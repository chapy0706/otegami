# otegami-cleaner の定期駆動

otegami-cleaner は失効分(TTL満了)と閲覧済み burn 分を物理削除する掃除バッチである(ADR-0002, ADR-0003)。常駐せず、1回実行して終了する。定期駆動は外部スケジューラに委ねる。既定間隔は10分。

実行には `DATABASE_URL` が必要。成功時は削除件数を標準出力に記録して exit 0、失敗時は非ゼロで終了する。

手元での単発実行:

```
DATABASE_URL=postgres://USER@HOST:5432/otegami make run/cleaner
```

## cron

10分間隔。ビルド済みバイナリのパスと接続情報は環境に合わせて置き換える。

```
*/10 * * * * DATABASE_URL=postgres://USER@HOST:5432/otegami /opt/otegami/bin/cleaner >> /var/log/otegami/cleaner.log 2>&1
```

## systemd timer

サービス本体 `otegami-cleaner.service`:

```
[Unit]
Description=otegami cleaner (purge expired and burned notes)

[Service]
Type=oneshot
Environment=DATABASE_URL=postgres://USER@HOST:5432/otegami
ExecStart=/opt/otegami/bin/cleaner
```

10分間隔のタイマー `otegami-cleaner.timer`:

```
[Unit]
Description=Run otegami cleaner every 10 minutes

[Timer]
OnBootSec=10min
OnUnitActiveSec=10min
Persistent=true

[Install]
WantedBy=timers.target
```

有効化:

```
systemctl enable --now otegami-cleaner.timer
```

## Coolify スケジュール

Coolify のスケジュールタスク(Scheduled Task)として、対象リソースに対し次を設定する。

- Command: `cleaner`
- Frequency: `*/10 * * * *`

`DATABASE_URL` はリソースの環境変数として与える。
