.PHONY: dev dev-build dev-down dev-logs prod prod-build prod-down build-all test-rust test-go test-web test-all lint-rust lint-go lint-web lint-all db-shell db-migrate db-seed redis-cli minio-console clean health reset

# ===========================
# Development environment
# ===========================

dev:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml up -d

dev-build:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml build

dev-down:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml down

dev-logs:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml logs -f $(service)

# ===========================
# Production environment
# ===========================

prod:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.prod.yml up -d

prod-build:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.prod.yml build

prod-down:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.prod.yml down

# ===========================
# Build
# ===========================

build-all:
	docker compose -f deploy/docker/docker-compose.yml build

# ===========================
# Tests
# ===========================

test-rust:
	cargo test --workspace

test-go:
	cd services/api && go test ./...

test-web:
	cd web/admin && npm test

test-all: test-rust test-go test-web

# ===========================
# Linting
# ===========================

lint-rust:
	cargo clippy --workspace -- -D warnings

lint-go:
	cd services/api && golangci-lint run

lint-web:
	cd web/admin && npm run lint

lint-all: lint-rust lint-go lint-web

# ===========================
# Database & utilities
# ===========================

db-shell:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml exec postgres psql -U rdcs -d rdcs

db-migrate:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml restart postgres

db-seed:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml restart postgres

redis-cli:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml exec redis redis-cli

minio-console:
	@echo "MinIO Console: http://localhost:9001"
	@echo "  User: minioadmin"
	@echo "  Password: minioadmin_dev"

# ===========================
# Cleanup
# ===========================

clean:
	docker compose -f deploy/docker/docker-compose.yml -f deploy/docker/docker-compose.dev.yml down -v --remove-orphans

# ===========================
# Health check & reset
# ===========================

health:
	@bash scripts/health-check.sh

reset:
	@bash scripts/reset-dev.sh
