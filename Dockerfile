# syntax=docker/dockerfile:1

# ---- build stage -------------------------------------------------
# A1 は ARM64。Coolify が A1 上で建てればこの stage はネイティブ arm64 で走る。
FROM rust:1.96-slim-bookworm AS builder

WORKDIR /app

# SQLx をオフラインでビルドする(.sqlx/ をコミット済み前提)。
# DB が無くてもクエリの型検査ができる。
ENV SQLX_OFFLINE=true

# TLS は rustls を推奨(OpenSSL 依存を避ける)。
# native-tls を使う場合のみ、ここで pkg-config libssl-dev を入れる。

COPY . .

# web と cleaner の二バイナリをまとめてビルド。
RUN cargo build --release --bin otegami --bin otegami-cleaner

# ---- runtime stage -----------------------------------------------
FROM debian:bookworm-slim AS runtime

# 外向き TLS と証明書検証に必要。
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Askama はテンプレートをバイナリに焼き込むため、templates/ は実行時不要。
COPY --from=builder /app/target/release/otegami         /usr/local/bin/otegami
COPY --from=builder /app/target/release/otegami-cleaner /usr/local/bin/otegami-cleaner

# 設定はすべて環境変数で渡す。コンテナでは 0.0.0.0 で待ち受けること。
EXPOSE 8099

# 既定は Web 本体。cleaner は同じイメージで command を otegami-cleaner に差し替えて使う。
CMD ["otegami"]
