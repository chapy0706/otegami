#!/usr/bin/env bash
# .claude/verify.sh
#
# プロジェクト固有の検証スクリプト。
# .claude/hooks/run-verify.sh（Stop hook）から呼ばれる。
#
# 役割:
#   コードを変更したあと、意図通りか・壊れていないかを確認する。
#   format / lint / test をここに並べる。
#
# 方針:
#   - 速い検証（fmt --check, clippy）はそのまま並べる
#   - test は cargo がビルドを兼ねるため、ビルドの健全性もここで担保される
#   - 失敗したら非ゼロで終了する。run-verify.sh がそれを Claude に返す
#
# 注意:
#   このファイルは検証の強度そのもの。Claude Code から編集できないよう
#   .claude/settings.json の deny で Edit/Write を禁止すること。

set -euo pipefail

# Rust / Axum / Askama プロジェクトの検証。

echo "[verify] fmt"
cargo fmt --all -- --check

echo "[verify] clippy"
cargo clippy --all-targets --all-features -- -D warnings

echo "[verify] test"
cargo test --all

echo "[verify] passed."
