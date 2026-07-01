# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0

# RDCS Flutter Client - Build Automation

PROJECT_ROOT := $(shell cd ../.. && pwd)
RUST_LIB := $(PROJECT_ROOT)/target/debug/librdcs_core.dylib
FRAMEWORKS_DIR := build/macos/Build/Products/Debug/rdcs_client.app/Contents/Frameworks

.PHONY: help
help:
	@echo "RDCS Flutter Client - Build Commands"
	@echo ""
	@echo "  make build-rust    - Build Rust FFI library"
	@echo "  make run           - Build and run (with hot reload)"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make rebuild       - Clean + rebuild everything"
	@echo ""

.PHONY: build-rust
build-rust:
	@echo "━━━ Building Rust FFI library ━━━"
	cd $(PROJECT_ROOT) && cargo build -p rdcs-ffi
	@echo "✅ Rust library built: $(RUST_LIB)"

.PHONY: copy-ffi-lib
copy-ffi-lib: build-rust
	@echo "━━━ Copying FFI library to Frameworks ━━━"
	@mkdir -p $(FRAMEWORKS_DIR)
	@if [ -f "$(RUST_LIB)" ]; then \
		cp -v "$(RUST_LIB)" "$(FRAMEWORKS_DIR)/"; \
		echo "✅ Library copied to $(FRAMEWORKS_DIR)"; \
	else \
		echo "❌ Rust library not found at $(RUST_LIB)"; \
		exit 1; \
	fi

.PHONY: run
run: build-rust
	@echo "━━━ Running Flutter app ━━━"
	@flutter run -d macos &
	@sleep 8
	@echo ""
	@echo "━━━ Copying FFI library to running app ━━━"
	@$(MAKE) copy-ffi-lib
	@echo ""
	@echo "✅ App is running. Use 'r' to hot reload, 'R' to hot restart, 'q' to quit."
	@echo "   Logs: tail -f /tmp/flutter_run.log"
	@wait

.PHONY: build
build: build-rust
	@echo "━━━ Building Flutter app ━━━"
	flutter build macos
	@$(MAKE) copy-ffi-lib

.PHONY: clean
clean:
	@echo "━━━ Cleaning Flutter build ━━━"
	flutter clean
	@echo "✅ Clean complete"

.PHONY: rebuild
rebuild: clean build
	@echo "✅ Rebuild complete"

.PHONY: check-deps
check-deps:
	@echo "━━━ Checking dependencies ━━━"
	@which flutter > /dev/null || (echo "❌ Flutter not installed" && exit 1)
	@which cargo > /dev/null || (echo "❌ Rust/Cargo not installed" && exit 1)
	@[ -f "$(PROJECT_ROOT)/Cargo.toml" ] || (echo "❌ Project root not found" && exit 1)
	@echo "✅ All dependencies present"
