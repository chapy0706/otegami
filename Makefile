SHELL := /bin/bash

.PHONY: help
help:
	@echo ""
	@echo "otegami Makefile"
	@echo ""
	@echo "  setup          初期セットアップ（sqlx-cli, cargo-watch）"
	@echo "  dev            開発サーバー起動（Web 本体・ホットリロード）"
	@echo "  run/web        Web 本体を起動"
	@echo "  run/cleaner    掃除バッチを単発実行"
	@echo ""
	@echo "  test           全テスト実行（cargo test）"
	@echo ""
	@echo "  lint           静的解析（clippy・警告をエラー扱い）"
	@echo "  fmt            フォーマット適用（cargo fmt）"
	@echo "  fmt/check      フォーマット検査（差分があれば失敗）"
	@echo ""
	@echo "  db/up          マイグレーション適用（sqlx migrate run）"
	@echo "  db/new name=X  マイグレーションファイル新規作成"
	@echo "  db/prepare     SQLx オフラインデータ生成（.sqlx/）"
	@echo ""
	@echo "  verify         全チェック（fmt/check + lint + test）"
	@echo "  evidence       verify + ログ出力"
	@echo ""
	@echo "  issue/list     docs/issues 配下の未完了 Issue を表示"
	@echo "  issue/new      Issue テンプレートを作成"
	@echo ""

# ------------------------
# Setup
# ------------------------

.PHONY: setup
setup:
	cargo install sqlx-cli --no-default-features --features rustls,postgres
	cargo install cargo-watch

# ------------------------
# Run
# ------------------------

.PHONY: dev
dev:
	cargo watch -x 'run -p web'

.PHONY: run/web
run/web:
	cargo run -p web

.PHONY: run/cleaner
run/cleaner:
	cargo run -p cleaner

# ------------------------
# Test
# ------------------------

.PHONY: test
test:
	cargo test --all

# ------------------------
# Lint / Format
# ------------------------

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt/check
fmt/check:
	cargo fmt --all -- --check

# ------------------------
# DB Migration（SQLx）
# ------------------------

.PHONY: db/up
db/up:
	sqlx migrate run

.PHONY: db/new
db/new:
	@if [ -z "$(name)" ]; then echo "使い方: make db/new name=migration_name" && exit 1; fi
	sqlx migrate add $(name)

.PHONY: db/prepare
db/prepare:
	cargo sqlx prepare --workspace

# ------------------------
# Verify（重要: Claude Code はこれを必ず通すこと）
# ------------------------

.PHONY: verify
verify: fmt/check lint test
	@echo ""
	@echo "verify passed."

.PHONY: evidence
evidence:
	@mkdir -p tmp/evidence
	cargo fmt --all -- --check 2>&1 | tee tmp/evidence/fmt.log
	cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tee tmp/evidence/lint.log
	cargo test --all 2>&1 | tee tmp/evidence/test.log
	@echo ""
	@echo "evidence saved to tmp/evidence/"

# ------------------------
# Issue 管理
# ------------------------

.PHONY: issue/list
issue/list:
	@echo ""
	@echo "未完了 Issue 一覧 (docs/issues/):"
	@echo ""
	@if ls docs/issues/*.md >/dev/null 2>&1; then \
		grep -l "status: open" docs/issues/*.md 2>/dev/null \
			| xargs -I{} sh -c 'echo "  $$(basename {}): $$(grep "^# " {} | head -1 | sed "s/^# //")"' \
			|| echo "  （未完了の Issue はありません）"; \
	else \
		echo "  （docs/issues/ に Issue ファイルがありません）"; \
	fi
	@echo ""

.PHONY: issue/new
issue/new:
	@if [ -z "$(name)" ]; then echo "使い方: make issue/new name=issue-XX-issue-name" && exit 1; fi
	@if [ -f "docs/issues/$(name).md" ]; then echo "すでに存在します: docs/issues/$(name).md" && exit 1; fi
	cp docs/issues/_template.md docs/issues/$(name).md
	@echo "作成しました: docs/issues/$(name).md"
