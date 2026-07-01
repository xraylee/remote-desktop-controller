# Test Coverage Analysis - RDCS Flutter Client

**Date:** 2026-06-30
**Test Framework:** Flutter Test
**Current Test Lines:** 1,912
**Production Code Lines:** 7,040
**Test Coverage Ratio:** ~27% (by line count)

---

## Current Test Coverage

### ✅ Well-Covered Areas

1. **UI Widget Tests** (~80% coverage)
   - `HomePage` - device code display, buttons, copy functionality
   - `ConnectPage` - form validation, input handling, navigation
   - `SessionScreen` - all states (connecting, connected, disconnected, error)
   - `ConnectionConfirmDialog` - accept/reject flows, countdown timer
   - `SettingsScreen` - form interactions, preferences

2. **UI Integration Tests** (~70% coverage)
   - End-to-end UI flows
   - Dialog interactions
   - Navigation between screens
   - Performance benchmarks (dialog opening time)
   - Accessibility checks (semantic labels, tap targets)

### ⚠️ Partially Covered Areas

1. **Session Management** (~30% coverage)
   - Basic provider tests exist in helpers
   - Missing: edge cases, error recovery, state transitions

2. **Configuration** (~20% coverage)
   - Basic config loading tested in helpers
   - Missing: persistence, validation, migration

### ❌ No Test Coverage

1. **Core Services** (0% coverage)
   - `SignalingService` - WebSocket communication
   - `WebSocketClient` - connection management, heartbeat
   - `SignalingProvider` - state management integration

2. **FFI Layer** (0% coverage)
   - `EngineIsolate` - Rust FFI communication
   - `NativeBindings` - platform bindings
   - `EngineEvents` - event stream handling

3. **Video Rendering** (0% coverage)
   - `VideoRenderer` - frame decoding, display
   - Frame buffer management
   - FPS and latency tracking

4. **Input Control** (0% coverage)
   - Mouse event handling
   - Keyboard event handling
   - Scroll event handling

5. **Tray Service** (0% coverage)
   - System tray integration
   - Auto-start functionality
   - Menu handling

6. **Config Repository** (0% coverage)
   - SharedPreferences persistence
   - Config validation
   - Migration logic

---

## Critical Testing Gaps (Priority Order)

### P0 - Critical (Must Have)

1. **SignalingService Unit Tests**
   - Connection lifecycle (connect, disconnect, reconnect)
   - Message handling (all message types)
   - Error handling (network errors, server errors)
   - Stream management (nearby devices, invitations, errors)

2. **VideoRenderer Widget Tests**
   - Frame decoding and display
   - FPS tracking accuracy
   - Latency calculation
   - Memory management (image disposal)

3. **EngineIsolate Integration Tests**
   - FFI method calls
   - Event emission
   - Error propagation
   - Isolate communication

### P1 - High (Should Have)

4. **WebSocketClient Unit Tests**
   - Connection state management
   - Message serialization/deserialization
   - Heartbeat mechanism
   - Reconnection logic

5. **Session Flow Integration Tests**
   - Complete connection flow (request → accept → relay → video)
   - Input event handling (mouse, keyboard)
   - Session termination scenarios
   - Error recovery paths

6. **Config Repository Unit Tests**
   - Save/load operations
   - Default values
   - Validation rules
   - Migration between versions

### P2 - Medium (Nice to Have)

7. **Performance Tests**
   - Video frame rendering latency (<16ms for 60fps)
   - Input event latency (<10ms)
   - Memory usage during long sessions
   - CPU usage profiling

8. **Tray Service Integration Tests**
   - Tray icon display
   - Menu interactions
   - Window show/hide
   - Auto-start behavior

9. **Error Recovery Tests**
   - Network interruption handling
   - Signaling server failure
   - Relay server failure
   - Decoder errors

---

## Test Strategy by Component

### 1. SignalingService (TDD Required)

**Test Files to Create:**
- `test/core/signaling/signaling_service_test.dart`
- `test/core/signaling/websocket_client_test.dart`
- `test/core/signaling/signaling_message_test.dart`

**Key Test Scenarios:**
```dart
// Connection lifecycle
test('connects and registers device')
test('disconnects cleanly')
test('reconnects after connection loss')
test('handles connection timeout')

// Message handling
test('routes connect_request to invitations stream')
test('routes nearby_update to nearby devices')
test('routes error messages to errors stream')
test('handles malformed messages gracefully')

// State management
test('connection state reflects WebSocket state')
test('nearby devices list updates correctly')
test('multiple message handlers dont conflict')
```

### 2. VideoRenderer (TDD Required)

**Test Files to Create:**
- `test/features/session/video_renderer_test.dart`
- `test/features/session/video_frame_test.dart`

**Key Test Scenarios:**
```dart
// Frame rendering
test('decodes and displays BGRA frame')
test('handles frame size changes')
test('disposes old frames when new arrives')
test('shows placeholder when no frames')

// Performance tracking
test('calculates FPS correctly')
test('calculates latency from timestamp')
test('updates stats overlay')

// Error handling
test('handles invalid base64 data')
test('handles wrong frame size')
test('handles decoding errors')
```

### 3. EngineIsolate (TDD Required)

**Test Files to Create:**
- `test/core/ffi/engine_isolate_test.dart`
- `test/core/ffi/engine_events_test.dart`

**Key Test Scenarios:**
```dart
// FFI calls
test('connect() calls native and returns result')
test('disconnect() cleans up resources')
test('sendInput() transmits events')

// Event handling
test('emits frameReady events')
test('emits sessionState events')
test('emits error events')

// Error handling
test('handles FFI call failures')
test('handles invalid event data')
test('isolate restart on crash')
```

### 4. Session Flow Integration (TDD Required)

**Test Files to Create:**
- `test/integration/session_flow_test.dart`
- `test/integration/input_control_test.dart`

**Key Test Scenarios:**
```dart
// Complete flow
test('controller connects to controlled device')
test('controlled device accepts connection')
test('relay assigned and data channel opens')
test('video frames transmitted')
test('input events transmitted')
test('session terminates cleanly')

// Error scenarios
test('handles connection rejection')
test('handles relay assignment failure')
test('handles network interruption')
test('handles decoder failure')
```

---

## Test Execution Plan

### Phase 1: Core Services (Days 1-2)
- SignalingService unit tests
- WebSocketClient unit tests
- Config repository unit tests

### Phase 2: FFI Layer (Days 3-4)
- EngineIsolate integration tests
- EngineEvents tests
- Native bindings mock setup

### Phase 3: Video & Input (Days 5-6)
- VideoRenderer widget tests
- Input control tests
- Performance benchmarks

### Phase 4: Integration (Days 7-8)
- Session flow integration tests
- Error recovery tests
- End-to-end scenarios

### Phase 5: Performance (Days 9-10)
- Rendering latency tests
- Input latency tests
- Memory profiling
- CPU profiling

---

## Test Infrastructure Needs

### Mocks & Fakes Required

1. **FakeEngineIsolate** ✅ (Already exists in helpers.dart)
2. **MockWebSocketClient** ❌ (Need to create)
3. **MockSignalingService** ❌ (Need to create)
4. **FakeVideoFrameSource** ❌ (Need to create)
5. **MockNativeBindings** ❌ (Need to create)

### Test Utilities Required

1. **Frame generator** - Create test BGRA frames
2. **Event simulator** - Simulate engine events
3. **Network simulator** - Simulate connection issues
4. **Performance profiler** - Measure latency/FPS

### Dependencies to Add

```yaml
dev_dependencies:
  mockito: ^5.4.0           # For mocking
  build_runner: ^2.4.0      # Already present
  integration_test: ^1.0.0  # For integration tests
  flutter_driver: ^1.0.0    # For performance tests
```

---

## Success Criteria

- [ ] Core services: >80% line coverage
- [ ] FFI layer: >70% line coverage
- [ ] Video rendering: >85% line coverage
- [ ] UI components: >90% line coverage (already achieved)
- [ ] Integration tests: All critical paths covered
- [ ] Performance tests: All latency requirements verified
- [ ] All tests pass in CI/CD pipeline
- [ ] Test execution time <2 minutes for unit tests
- [ ] Test execution time <5 minutes for full suite

---

## Next Steps

1. ✅ Complete coverage analysis (Task #1)
2. Create unit tests for SignalingService (Task #2)
3. Create integration tests for session flow (Task #3)
4. Create widget tests for VideoRenderer (Task #4)
5. Create performance benchmarks (Task #5)
6. Run full test suite and generate coverage report (Task #6)
