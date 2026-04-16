# osef.me v2

Self-hosted osu!mania difficulty and performance rating platform.

## Structure

- `api/` — Deno backend (REST API + DB queries)
- `processor/` — Rust crates for difficulty/performance calculators
- `migrations/` — Database migrations
- `build/` — Build artifacts
- `compose.yml` — Docker Compose (dev)
- `compose.prod.yml` — Docker Compose (prod)

## Requirements

- Rust (latest stable)
- Deno 2.x
- Docker + Docker Compose

## Quick start

```sh
docker compose up
```
