# RDCS Developer Guide

## Prerequisites

Before you begin, ensure you have the following installed:

| Tool | Minimum Version | Purpose |
|------|----------------|---------|
| Docker | 24.0+ | Container runtime |
| Docker Compose | 2.20+ | Service orchestration |
| Rust | 1.80+ | Signaling & relay server development |
| Go | 1.22+ | Management API development |
| Node.js | 20+ | Web admin console development |
| Make | 3.81+ | Build automation |

> Only Docker and Docker Compose are required to run the full stack. The language toolchains are needed only for local development outside of Docker.

## Quick Start

Get the entire development environment running with a single command:

```bash
# Start all 7 services in development mode
make dev

# Verify all services are healthy
make health
```

The first run takes several minutes to build images and download dependencies. Subsequent starts are much faster thanks to Docker layer caching.

To view logs from all services:

```bash
make dev-logs
```

To view logs from a specific service:

```bash
make dev-logs service=api
```

To stop the development environment:

```bash
make dev-down
```

## Service Endpoints

Once all services are running, the following endpoints are available on `localhost`:

| Service | URL | Description |
|---------|-----|-------------|
| Signaling Server | `https://localhost:8443` | WebSocket signaling (TLS disabled in dev) |
| Relay Server | `udp://localhost:3478` | STUN/TURN UDP endpoint |
| Management API | `http://localhost:8080` | REST API (`/healthz` for health check) |
| Web Admin Console | `http://localhost:3000` | React SPA with Vite HMR |
| PostgreSQL | `localhost:5432` | Database (user: `rdcs`, password: `rdcs_dev`, db: `rdcs`) |
| Redis | `localhost:6379` | Session cache |
| MinIO API | `http://localhost:9000` | S3-compatible object storage |
| MinIO Console | `http://localhost:9001` | MinIO web UI (user: `minioadmin`, password: `minioadmin_dev`) |

## Hot-Reload Guide

Each application service supports hot-reload in development mode:

### Rust Services (signaling, relay)
- **Tool**: `cargo-watch`
- **Watched paths**: `crates/*/src/`
- **Behavior**: Edits to `.rs` files trigger automatic recompilation and restart
- **Delay**: ~2-10 seconds depending on change scope
- **Example**: Edit `crates/rdcs-signaling/src/main.rs` and watch the container logs

### Go API
- **Tool**: `air`
- **Watched paths**: `services/api/**/*.go`
- **Behavior**: Edits to `.go` files trigger automatic recompilation and restart
- **Delay**: ~1-3 seconds
- **Config**: `services/api/.air.toml`

### Web Admin Console
- **Tool**: Vite dev server (HMR)
- **Watched paths**: `web/admin/src/`, `web/admin/public/`
- **Behavior**: Edits to `.tsx`, `.ts`, `.css` files trigger instant browser updates
- **Delay**: <1 second (hot module replacement, no full reload)

## Makefile Targets

### Development Environment

| Target | Description |
|--------|-------------|
| `make dev` | Start all services in development mode |
| `make dev-build` | Build all dev images without starting |
| `make dev-down` | Stop all development services |
| `make dev-logs [service=X]` | Tail logs (optionally for a single service) |

### Production Environment

| Target | Description |
|--------|-------------|
| `make prod` | Start all services in production mode |
| `make prod-build` | Build all production images |
| `make prod-down` | Stop all production services |

### Testing & Linting

| Target | Description |
|--------|-------------|
| `make test-rust` | Run Rust workspace tests |
| `make test-go` | Run Go API tests |
| `make test-web` | Run Web console tests |
| `make test-all` | Run all test suites |
| `make lint-rust` | Run clippy with `-D warnings` |
| `make lint-go` | Run golangci-lint |
| `make lint-web` | Run ESLint |
| `make lint-all` | Run all linters |

### Database & Utilities

| Target | Description |
|--------|-------------|
| `make db-shell` | Open PostgreSQL interactive shell |
| `make db-migrate` | Re-run database migrations (restarts postgres) |
| `make db-seed` | Re-run seed data (restarts postgres) |
| `make redis-cli` | Open Redis CLI |
| `make minio-console` | Open MinIO Console URL |

### Maintenance

| Target | Description |
|--------|-------------|
| `make health` | Check health status of all 7 services |
| `make clean` | Remove all containers, volumes, and orphans |
| `make reset` | Full reset: clean + rebuild dev environment |
| `make build-all` | Build all service images without starting |

## Common Troubleshooting

### Port conflicts

If a port is already in use on your host, set an alternative in your environment:

```bash
# Example: change API port from 8080 to 8081
API_PORT=8081 make dev
```

### Container fails to start

Check the service logs:

```bash
make dev-logs service=signaling
```

### Database migrations not applied

Migrations run automatically when PostgreSQL starts for the first time (empty volume). To re-run:

```bash
make clean
make dev
```

Or to reset just the database:

```bash
docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml restart postgres
```

### Hot-reload not working

1. **Rust services**: Ensure your editor saves files (not just buffer changes). cargo-watch detects file system events.
2. **Go API**: Ensure `.air.toml` exists in `services/api/`. Check `make dev-logs service=api` for air output.
3. **Web console**: Ensure the Vite dev server started correctly. Check `make dev-logs service=web` for the Vite ready message.

### MinIO buckets missing

Run the MinIO init container:

```bash
docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml --profile init run --rm minio-init
```

### Full environment reset

If things are in a bad state:

```bash
make reset
```

This removes all containers, volumes, and networks, then rebuilds and restarts everything.

## Project Structure

```
remote-desktop-controller/
  crates/                  # Rust workspace crates
    rdcs-signaling/        #   WebSocket signaling server
    rdcs-relay/            #   UDP media relay
    rdcs-core/             #   Shared core library
    rdcs-crypto/           #   Encryption (NaCl/libsodium)
    rdcs-transport/        #   Network transport layer
    rdcs-connection/       #   Connection management
    rdcs-codec/            #   Video codec interfaces
    rdcs-platform/         #   Platform abstractions
    rdcs-transfer/         #   File transfer logic
    rdcs-ffi/              #   FFI bindings for client
    rdcs-macos/            #   macOS-specific code
  services/
    api/                   # Go management API
  web/
    admin/                 # React admin console
  client/
    flutter/               # Flutter desktop client (future)
  deploy/
    docker/                # Dockerfiles and compose files
  migrations/
    postgres/              # SQL migration scripts
  scripts/                 # Helper scripts
  Makefile                 # Build automation
```

## Development Workflow

1. **Start the stack**: `make dev`
2. **Make changes**: Edit source files; services hot-reload automatically
3. **Run tests**: `make test-all` before committing
4. **Check lint**: `make lint-all` before committing
5. **Verify health**: `make health` to confirm all services are running
6. **Commit**: Follow [Conventional Commits](../CONTRIBUTING.md) format

## Environment Files

| File | Purpose |
|------|---------|
| `deploy/docker/.env.dev` | Default development environment variables |
| `deploy/docker/.env.prod.example` | Production environment template |

For development, the defaults in `.env.dev` are sufficient. No configuration needed.

For production, copy `.env.prod.example` to `.env` and fill in your domain, passwords, and email.
