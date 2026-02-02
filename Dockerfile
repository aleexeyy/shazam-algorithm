# syntax=docker/dockerfile:1

FROM node:20-bookworm-slim AS frontend-builder
WORKDIR /frontend
COPY front-end/package.json front-end/package-lock.json ./
RUN npm ci
COPY front-end ./
RUN npm run build

FROM rust:1-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY benches ./benches

RUN cargo build --release --bin shazam-server

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
 && apt-get install -y --no-install-recommends \
    ca-certificates \
    ffmpeg \
    curl \
 && ARCH="$(uname -m)" \
 && if [ "$ARCH" = "x86_64" ]; then BIN="yt-dlp_linux"; \
    elif [ "$ARCH" = "aarch64" ]; then BIN="yt-dlp_linux_aarch64"; \
    else echo "Unsupported arch: $ARCH" && exit 1; fi \
 && curl -L "https://github.com/yt-dlp/yt-dlp/releases/latest/download/$BIN" \
    -o /usr/local/bin/yt-dlp \
 && chmod +x /usr/local/bin/yt-dlp \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shazam-server /usr/local/bin/shazam-server
COPY --from=frontend-builder /frontend/dist /opt/shazam/frontend/dist

RUN mkdir -p /app/audio /app/log \
    && chown -R nobody:nogroup /app

USER nobody

EXPOSE 8000
ENV FRONTEND_DIST=/opt/shazam/frontend/dist
ENV RUST_LOG=debug
CMD ["shazam-server"]
