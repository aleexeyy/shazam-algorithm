#!/usr/bin/env bash
set -euo pipefail

docker compose build
docker compose up -d

echo "Waiting for app health..."
for i in {1..60}; do
  if curl -fsS "http://localhost:${SERVER_PORT:-8000}/healthz" >/dev/null; then
    echo "OK"
    exit 0
  fi
  sleep 1
done

echo "App did not become healthy in time" >&2
docker compose logs --no-color app >&2 || true
exit 1

