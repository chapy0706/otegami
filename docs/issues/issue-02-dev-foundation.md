---
status: open
created_at: 2026-06-27
closed_at:
---

# Claude Code 開発基盤設定

## 概要・背景・目的

project-bootstrap で開発基盤を配置し、Cargo workspace の骨組みを立てる。以降の実装 issue が「層が既にあり、make verify が回る」状態から始められるようにする。

## 受け入れ条件

- [ ] project-bootstrap に rust-axum-askama stack が追加されている(`templates/makefiles/Makefile.rust-axum-askama`, `templates/verify/verify.rust-axum-askama.sh`, `templates/skills/rust/`)
- [ ] `.bootstrap.toml`(stack.id=rust-axum-askama, skills_base=rust)を配置し、`bootstrap otegami` が成功する
- [ ] `.claude/`(hooks, rules, settings.json, verify.sh, skills)と `CLAUDE.md`, `Makefile`, `docs/` が生成されている
- [ ] 生成された CLAUDE.md の kyusoku 由来「ログ append-only」記述を otegami 用に修正している
- [ ] Cargo workspace の空スケルトン(crates/domain, crates/application, crates/infrastructure, apps/web, apps/cleaner)が立っている
- [ ] `.claude/settings.json` の deny で Makefile と .claude/ の編集を禁止している
- [ ] 空スケルトンで `make verify` が通る

## 技術的な検討事項

- ソース骨組みは bootstrap の管轄外。`cargo new` でワークスペースを手で作る
- ルートの Cargo.toml に members を列挙し、各 crate は最小の lib/bin で起動できる状態にする
- managed-settings.json はシステム領域へ別途手動配置(bootstrap 管轄外)

## 関連ADR・依存issue

- 依存: issue-01
- ADR: 0002(二バイナリ構成)

## 想定工数・優先度

- 優先度: 最高
- 工数: 小〜中
