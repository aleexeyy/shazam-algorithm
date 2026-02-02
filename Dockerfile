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
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/shazam-server /usr/local/bin/shazam-server
COPY --from=frontend-builder /frontend/dist /opt/shazam/frontend/dist

EXPOSE 8000
ENV FRONTEND_DIST=/opt/shazam/frontend/dist
ENV RUST_LOG=info
CMD ["shazam-server"]
