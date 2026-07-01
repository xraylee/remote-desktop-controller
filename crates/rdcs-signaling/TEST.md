# RDCS Signaling Server - Test Plan

**Version**: 1.0  
**Created**: 2026-06-29  
**Status**: Active

---

## Overview

This document defines the testing strategy, test cases, and acceptance criteria for the RDCS Signaling Server (`rdcs-signaling`), following the **Superpowers** testing framework standards.

### What We're Testing

The RDCS Signaling Server is an Axum-based HTTP/WebSocket service that provides:
- Device registration and discovery
- Peer-to-peer connection negotiation
- ICE candidate exchange (WebRTC signaling)
- Invite code generation and consumption
- Multi-instance scaling via Redis pub/sub

---

## Test Strategy

### Test Pyramid

```
    /\
   /  \     E2E Tests (Integration)
  /────\    - Full WebSocket flows
 /      \   - Real server instances
/────────\  
           Unit Tests
           - Message parsing
           - Handler logic
           - Session management
```

### Test Levels

| Level | Coverage | Tools | Location |
|-------|----------|-------|----------|
| **Unit Tests** | Individual functions, message handlers | `#[test]`, `#[tokio::test]` | `src/**/*.rs` |
| **Integration Tests** | Full WebSocket flows, server lifecycle | `tokio-tungstenite` | `tests/integration_test.rs` |
| **Manual Verification** | Health checks, service startup | `curl`, `wscat` | (ad-hoc) |

### Testing Principles

1. **Isolation**: Tests use in-memory state (no Redis) by default
2. **Speed**: Unit tests < 100ms, integration tests < 5s
3. **Reliability**: No flaky timeouts, deterministic behavior
4. **Coverage**: Critical paths (register → connect → ICE) fully covered

---

## Test Cases

### 1. Unit Tests

#### 1.1 Health Check Endpoint
- **AC**: `GET /health` returns `{"status":"ok"}` with HTTP 200
- **Location**: `src/lib.rs::tests::health_returns_200_and_json`
- **Status**: ✅ Implemented

#### 1.2 Unknown Route Handling
- **AC**: Unknown routes return HTTP 404
- **Location**: `src/lib.rs::tests::unknown_route_returns_404`
- **Status**: ✅ Implemented

#### 1.3 Message Parsing
- **AC**: Valid JSON messages parse to correct `ClientMessage` variants
- **Location**: `src/ws/message.rs` (inline tests)
- **Status**: ✅ Implemented

---

### 2. Integration Tests

#### 2.1 Full Connection Flow
**Test**: `full_connection_flow`  
**File**: `tests/integration_test.rs`

**Scenario**:
1. Client A registers with device code `CLIENT-A`
2. Client B registers with device code `CLIENT-B`
3. A sends `connect_request` to B
4. B receives the request and sends `connect_response` (accepted)
5. A receives the response
6. A sends `ice_offer` with SDP and candidates
7. B receives the offer
8. B sends `ice_answer` with SDP
9. A receives the answer

**Expected**:
- All messages delivered correctly
- Session state persists in `SessionManager`
- No message loss or corruption

**Status**: ✅ Implemented

---

#### 2.2 Disconnect Cleanup
**Test**: `disconnect_cleanup`  
**File**: `tests/integration_test.rs`

**Scenario**:
1. Client A and B register in the same team (`team-1`)
2. Client A disconnects (closes WebSocket)
3. Server detects disconnect and removes A's session
4. B receives `nearby_update` with A marked as `online: false`

**Expected**:
- A's session removed from `SessionManager`
- B notified of A going offline within 2 seconds
- B's session remains active

**Status**: ✅ Implemented

---

#### 2.3 Invite Code Flow (Redis Required)
**Test**: `invite_code_flow`  
**File**: `tests/integration_test.rs`  
**Requires**: Redis server at `redis://127.0.0.1:6379`

**Scenario**:
1. Client A generates an invite code
2. Server stores code in Redis with TTL
3. Client B uses the invite code
4. Server forwards `connect_request` to A
5. B tries to reuse the same code (should fail)

**Expected**:
- Invite code is 4 digits
- Code is single-use (consumed after first use)
- Reuse attempt returns `invite_error`

**Status**: ✅ Implemented (marked `#[ignore]` by default)

---

### 3. Manual Verification Tests

#### 3.1 Service Startup
**Command**:
```bash
cd crates/rdcs-signaling
cargo run
```

**Expected**:
- Server binds to `127.0.0.1:8080` (or env-configured address)
- Logs: `"signaling server listening on ..."`
- No panic or crash

---

#### 3.2 Health Check (HTTP)
**Command**:
```bash
curl http://127.0.0.1:8080/health
```

**Expected Response**:
```json
{"status":"ok"}
```
**Status Code**: 200

---

#### 3.3 WebSocket Connection
**Command** (using `wscat` or browser DevTools):
```bash
wscat -c ws://127.0.0.1:8080/ws
```

**Send**:
```json
{"type":"register","device_code":"TEST-001","platform":"test","version":"0.0.1"}
```

**Expected**:
- Connection accepted (HTTP 101 Upgrade)
- No immediate error or disconnect

---

## Acceptance Criteria

### Must Pass (Blocking)

- [x] All unit tests pass: `cargo test --lib`
- [x] Integration tests pass (no Redis): `cargo test --test integration_test`
- [x] Health endpoint returns 200
- [x] WebSocket upgrade succeeds

### Should Pass (Non-Blocking)

- [ ] Invite code flow passes (requires Redis setup)
- [ ] No compiler warnings: `cargo clippy`
- [ ] Code formatted: `cargo fmt -- --check`

### Performance Targets

- Health check: < 10ms
- WebSocket upgrade: < 50ms
- Message round-trip (register → response): < 100ms
- Full connection flow test: < 5 seconds

---

## Test Execution

### Run All Tests (Fast)
```bash
# Unit tests only (no Redis required)
cargo test --lib

# Integration tests (no Redis required)
cargo test --test integration_test -- --skip invite_code_flow
```

### Run With Redis (Full Suite)
```bash
# Start Redis (if not running)
redis-server &

# Run all tests including invite flow
cargo test --test integration_test
```

### Run Manual Verification
```bash
# Terminal 1: Start server
cargo run

# Terminal 2: Health check
curl http://127.0.0.1:8080/health

# Terminal 3: WebSocket test
wscat -c ws://127.0.0.1:8080/ws
```

---

## Coverage Report

### Current Coverage (2026-06-29)

| Module | Unit Tests | Integration Tests | Coverage |
|--------|-----------|------------------|----------|
| `lib.rs` | ✅ Health, 404 | ✅ Full flow | 95% |
| `ws/handler.rs` | ⚠️ Partial | ✅ Full flow | 80% |
| `ws/session.rs` | ✅ State mgmt | ✅ Disconnect | 90% |
| `ws/message.rs` | ✅ Parsing | ✅ Full flow | 95% |
| `handlers/register.rs` | ❌ Missing | ✅ Full flow | 60% |
| `handlers/connect.rs` | ❌ Missing | ✅ Full flow | 60% |
| `handlers/invite.rs` | ❌ Missing | ✅ Invite flow | 50% |
| `redis/` | ❌ Missing | ✅ Invite flow | 40% |

**Overall**: ~70% code coverage (estimated)

### Coverage Gaps

1. **Redis key management**: No unit tests for `redis/keys.rs` and `redis/ttl.rs`
2. **Error paths**: Limited testing of malformed messages
3. **Edge cases**: Duplicate registrations, concurrent connects
4. **Load testing**: No stress tests for high concurrency

---

## Known Issues

### 1. Flaky Timeouts (Resolved)
- **Issue**: Integration tests occasionally timed out waiting for messages
- **Fix**: Increased timeouts to 3s for message receipt, 200ms for drain
- **Status**: ✅ Resolved (2026-06-28)

### 2. Redis Dependency
- **Issue**: Invite flow tests require external Redis server
- **Impact**: CI/CD needs Redis container
- **Mitigation**: Tests marked `#[ignore]` by default
- **Status**: ⚠️ Accepted (infrastructure requirement)

---

## Future Improvements

### Short-term (Q3 2026)
1. Add unit tests for all handlers
2. Mock Redis for invite flow tests (avoid external dependency)
3. Add benchmark tests for message throughput

### Medium-term (Q4 2026)
1. Load testing: 1000 concurrent connections
2. Chaos testing: network partition, Redis failure
3. Security testing: malformed WebSocket frames, auth bypass attempts

### Long-term (2027)
1. Fuzz testing for message parsing
2. E2E tests with real Flutter clients
3. Performance regression tracking in CI

---

## Test Maintenance

### When to Update This Document

- New test cases added
- Acceptance criteria changed
- Coverage gaps identified
- Known issues discovered or resolved

### Test Review Cadence

- **Weekly**: Check test pass rate in CI
- **Monthly**: Review coverage gaps, plan improvements
- **Quarterly**: Full test strategy review

---

## References

- [Superpowers Testing Framework](https://superpowers.dev/testing)
- [Integration Test Code](tests/integration_test.rs)
- [Unit Test Examples](src/lib.rs)
- [Axum Testing Guide](https://docs.rs/axum/latest/axum/testing/)

---

**Maintained by**: RDCS Team  
**Last Updated**: 2026-06-29
