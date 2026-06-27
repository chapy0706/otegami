---
status: open
created_at: 2026-06-27
closed_at:
---

# アプリケーション層: CreateNote / ViewNote / PurgeNotes

## 概要・背景・目的

domain のポートを束ねるユースケースを実装する。ポートは fake で差し替えてテストし、具象(DB)には依存させない。otegami の業務手順をここに集約する。

## 受け入れ条件

- [ ] CreateNote: body / title / raw_password / burn を受け、RawPassword 検証 → ハッシュ化 → slug 採番(衝突時リトライ)→ `expires_at = now + TTL` を算出 → 保存。Output に slug を返す
- [ ] ViewNote: slug 照合 → パスワード検証 → 初回なら viewed_at 記録 → body 返却。失敗は NotFoundOrWrongPassword に畳む
- [ ] PurgeNotes: 失効分・閲覧済み burn 分を削除し、件数を返す
- [ ] 各ユースケースは `async fn execute(&self, Input) -> Result<Output, Error>` 形
- [ ] 依存は `Arc<dyn Port>` のコンストラクタ注入
- [ ] 時刻は Clock 経由(now を直接呼ばない)
- [ ] fake を用いた単体テスト(正常・パスワード違い・存在なし・burn 時の viewed 記録)
- [ ] make verify が通る

## 技術的な検討事項

- slug 衝突は NoteRepository.insert が一意制約違反を返したらリトライ(上限回数を設ける)
- TTL は設定値(既定1日)から算出。設定値は合成位置(presentation)から渡す
- CreateNote のハッシュ化は PasswordHasher.hash 経由にする

## 関連ADR・依存issue

- 依存: issue-03
- ADR: 0001(単一エラー), 0003(TTL / burn)
- スキル: 02-implement-usecase.md

## 想定工数・優先度

- 優先度: 高
- 工数: 中
