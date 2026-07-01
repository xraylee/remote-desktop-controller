#!/bin/bash
# Copyright 2026 RDCS Contributors
# SPDX-License-Identifier: Apache-2.0
#
# Comprehensive test script for rdcs-signaling server
# Following Superpowers testing framework standards

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
SIGNALING_DIR="$PROJECT_ROOT/crates/rdcs-signaling"
REPORT_FILE="$PROJECT_ROOT/test-signaling-report.txt"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

log() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

success() {
    echo -e "${GREEN}[PASS]${NC} $1"
    ((PASSED_TESTS++))
}

fail() {
    echo -e "${RED}[FAIL]${NC} $1"
    ((FAILED_TESTS++))
}

warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

skip() {
    echo -e "${YELLOW}[SKIP]${NC} $1"
    ((SKIPPED_TESTS++))
}

section() {
    echo ""
    echo "=================================================="
    echo "$1"
    echo "=================================================="
    echo ""
}

# Initialize report
init_report() {
    cat > "$REPORT_FILE" <<EOF
RDCS Signaling Server - Test Execution Report
==============================================
Date: $(date '+%Y-%m-%d %H:%M:%S')
Host: $(hostname)
Platform: $(uname -s) $(uname -m)

EOF
}

# Append to report
append_report() {
    echo "$1" >> "$REPORT_FILE"
}

# Check prerequisites
check_prerequisites() {
    section "1. Prerequisites Check"

    log "Checking Rust toolchain..."
    if command -v cargo &> /dev/null; then
        RUST_VERSION=$(cargo --version)
        success "Rust installed: $RUST_VERSION"
        append_report "✓ Rust: $RUST_VERSION"
    else
        fail "Rust/Cargo not found"
        append_report "✗ Rust: Not found"
        exit 1
    fi

    log "Checking Redis availability..."
    if command -v redis-cli &> /dev/null; then
        if redis-cli ping &> /dev/null; then
            success "Redis server is running"
            append_report "✓ Redis: Running"
        else
            warn "Redis CLI found but server not responding"
            append_report "⚠ Redis: Not running (some tests will be skipped)"
        fi
    else
        warn "Redis CLI not found (invite code tests will be skipped)"
        append_report "⚠ Redis: Not found"
    fi

    log "Checking project structure..."
    if [ -d "$SIGNALING_DIR" ]; then
        success "Signaling crate found at: $SIGNALING_DIR"
        append_report "✓ Project: $SIGNALING_DIR"
    else
        fail "Signaling crate not found"
        append_report "✗ Project: Not found"
        exit 1
    fi
}

# Run unit tests
run_unit_tests() {
    section "2. Unit Tests (Library)"
    ((TOTAL_TESTS++))

    log "Running unit tests in rdcs-signaling..."
    append_report ""
    append_report "Unit Tests:"

    cd "$SIGNALING_DIR"
    if cargo test --lib --quiet 2>&1 | tee /tmp/unit-test-output.txt; then
        success "Unit tests passed"
        append_report "✓ Unit tests: PASSED"

        # Extract test count
        TEST_COUNT=$(grep -E "test result: ok\." /tmp/unit-test-output.txt | sed 's/.*\. \([0-9]*\) passed.*/\1/' || echo "unknown")
        log "  → $TEST_COUNT tests passed"
        append_report "  Tests executed: $TEST_COUNT"
    else
        fail "Unit tests failed"
        append_report "✗ Unit tests: FAILED"
        append_report "  See output above for details"
    fi
}

# Run integration tests (without Redis)
run_integration_tests_no_redis() {
    section "3. Integration Tests (No Redis)"
    ((TOTAL_TESTS++))

    log "Running integration tests (skipping invite_code_flow)..."
    append_report ""
    append_report "Integration Tests (No Redis):"

    cd "$SIGNALING_DIR"
    if cargo test --test integration_test -- --skip invite_code_flow --quiet 2>&1 | tee /tmp/integration-test-output.txt; then
        success "Integration tests passed (no Redis)"
        append_report "✓ Integration tests: PASSED"

        # Extract test count
        TEST_COUNT=$(grep -E "test result: ok\." /tmp/integration-test-output.txt | sed 's/.*\. \([0-9]*\) passed.*/\1/' || echo "unknown")
        log "  → $TEST_COUNT tests passed"
        append_report "  Tests executed: $TEST_COUNT"
    else
        fail "Integration tests failed"
        append_report "✗ Integration tests: FAILED"
    fi
}

# Run integration tests with Redis
run_integration_tests_with_redis() {
    section "4. Integration Tests (With Redis)"
    ((TOTAL_TESTS++))

    # Check if Redis is available
    if ! redis-cli ping &> /dev/null; then
        skip "Redis not available, skipping invite_code_flow test"
        append_report ""
        append_report "Integration Tests (With Redis):"
        append_report "⊘ Skipped: Redis not running"
        return
    fi

    log "Running full integration test suite (including invite_code_flow)..."
    append_report ""
    append_report "Integration Tests (With Redis):"

    cd "$SIGNALING_DIR"
    if cargo test --test integration_test --quiet 2>&1 | tee /tmp/integration-redis-test-output.txt; then
        success "Full integration test suite passed"
        append_report "✓ Full integration tests: PASSED"

        TEST_COUNT=$(grep -E "test result: ok\." /tmp/integration-redis-test-output.txt | sed 's/.*\. \([0-9]*\) passed.*/\1/' || echo "unknown")
        log "  → $TEST_COUNT tests passed"
        append_report "  Tests executed: $TEST_COUNT"
    else
        fail "Integration tests with Redis failed"
        append_report "✗ Full integration tests: FAILED"
    fi
}

# Run clippy linting
run_clippy() {
    section "5. Code Quality (Clippy)"
    ((TOTAL_TESTS++))

    log "Running Clippy linter..."
    append_report ""
    append_report "Code Quality:"

    cd "$SIGNALING_DIR"
    if cargo clippy --quiet -- -D warnings 2>&1 | tee /tmp/clippy-output.txt; then
        success "Clippy checks passed (no warnings)"
        append_report "✓ Clippy: PASSED (no warnings)"
    else
        warn "Clippy found issues"
        append_report "⚠ Clippy: WARNINGS FOUND"
        append_report "$(cat /tmp/clippy-output.txt | head -20)"
    fi
}

# Run format check
run_format_check() {
    section "6. Code Formatting"
    ((TOTAL_TESTS++))

    log "Checking code formatting..."
    append_report ""
    append_report "Code Formatting:"

    cd "$SIGNALING_DIR"
    if cargo fmt -- --check 2>&1; then
        success "Code is properly formatted"
        append_report "✓ Format: PASSED"
    else
        warn "Code formatting issues found"
        append_report "⚠ Format: NEEDS FORMATTING"
    fi
}

# Manual verification instructions
show_manual_tests() {
    section "7. Manual Verification (Instructions)"

    cat <<'EOF'
To perform manual verification:

1. Start the signaling server:
   cd crates/rdcs-signaling
   cargo run

2. In another terminal, check health endpoint:
   curl http://127.0.0.1:8080/health
   Expected: {"status":"ok"}

3. Test WebSocket connection (requires wscat):
   npm install -g wscat
   wscat -c ws://127.0.0.1:8080/ws

   Then send:
   {"type":"register","device_code":"TEST-001","platform":"test","version":"0.0.1"}

   Expected: Connection stays open, no errors

EOF

    append_report ""
    append_report "Manual Verification:"
    append_report "⊘ Not automated (see instructions above)"
}

# Generate summary
generate_summary() {
    section "8. Test Summary"

    echo ""
    echo "Test Execution Summary:"
    echo "  Total test suites: $TOTAL_TESTS"
    echo "  Passed: $PASSED_TESTS"
    echo "  Failed: $FAILED_TESTS"
    echo "  Skipped: $SKIPPED_TESTS"
    echo ""

    append_report ""
    append_report "=========================================="
    append_report "SUMMARY"
    append_report "=========================================="
    append_report "Total test suites: $TOTAL_TESTS"
    append_report "Passed: $PASSED_TESTS"
    append_report "Failed: $FAILED_TESTS"
    append_report "Skipped: $SKIPPED_TESTS"
    append_report ""

    if [ $FAILED_TESTS -eq 0 ]; then
        success "All tests passed! ✓"
        append_report "Result: ✓ ALL TESTS PASSED"
        echo ""
        log "Report saved to: $REPORT_FILE"
        return 0
    else
        fail "Some tests failed"
        append_report "Result: ✗ SOME TESTS FAILED"
        echo ""
        log "Report saved to: $REPORT_FILE"
        return 1
    fi
}

# Main execution
main() {
    log "Starting comprehensive test execution for rdcs-signaling..."
    echo ""

    init_report

    check_prerequisites
    run_unit_tests
    run_integration_tests_no_redis
    run_integration_tests_with_redis
    run_clippy
    run_format_check
    show_manual_tests

    generate_summary
}

# Run main
main
exit $?
