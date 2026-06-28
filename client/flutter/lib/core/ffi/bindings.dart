// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:ffi';
import 'dart:io';

import 'package:ffi/ffi.dart';

/// Native callback type for receiving events from the Rust engine.
///
/// - `eventId`: one of the EVENT_* constants
/// - `payload`: JSON-encoded event data (UTF-8, null-terminated)
/// - `payloadLen`: byte length of payload (excluding null terminator)
typedef EventCallbackNative = Void Function(
  Uint32 eventId,
  Pointer<Utf8> payload,
  IntPtr payloadLen,
);

/// Dart FFI bindings to the Rust core engine (`librdcs_core`).
///
/// The native library exposes 13 C-ABI functions for screen capture,
/// remote connection, input forwarding, file transfer, messaging,
/// quality control, and event callbacks. All string parameters are
/// UTF-8 encoded JSON.
///
/// Library naming convention:
///   - macOS:   librdcs_core.dylib
///   - Linux:   librdcs_core.so
///   - Windows: rdcs_core.dll
class RdcsBindings {
  RdcsBindings() : _lib = _loadLibrary() {
    _bindFunctions();
  }

  final DynamicLibrary _lib;

  // ── Library loading ──────────────────────────────────────────

  static DynamicLibrary _loadLibrary() {
    if (Platform.isMacOS) {
      return DynamicLibrary.open('librdcs_core.dylib');
    } else if (Platform.isLinux) {
      return DynamicLibrary.open('librdcs_core.so');
    } else if (Platform.isWindows) {
      return DynamicLibrary.open('rdcs_core.dll');
    }
    throw UnsupportedError(
      'Unsupported platform: ${Platform.operatingSystem}',
    );
  }

  // ── FFI function signatures ──────────────────────────────────

  // 1. rdcs_engine_create — Create a new engine instance.
  //    Returns an opaque handle pointer on success, nullptr on failure.
  //    void* rdcs_engine_create(const char* config_json);
  late final Pointer<Void> Function(Pointer<Utf8> configJson) engineCreate;

  // 2. rdcs_engine_destroy — Destroy an engine instance and free resources.
  //    void rdcs_engine_destroy(void* handle);
  late final void Function(Pointer<Void> handle) engineDestroy;

  // 3. rdcs_start_capture — Begin capturing the local screen.
  //    int32_t rdcs_start_capture(void* handle, const char* config_json);
  late final int Function(Pointer<Void> handle, Pointer<Utf8> configJson)
      startCapture;

  // 4. rdcs_stop_capture — Stop the current screen capture.
  //    int32_t rdcs_stop_capture(void* handle);
  late final int Function(Pointer<Void> handle) stopCapture;

  // 5. rdcs_connect — Initiate a connection to a remote device by code.
  //    Returns a positive session ID on success, or negative error code.
  //    int32_t rdcs_connect(void* handle, const char* target_code);
  late final int Function(Pointer<Void> handle, Pointer<Utf8> targetCode)
      connect;

  // 6. rdcs_disconnect — Terminate a specific remote session.
  //    int32_t rdcs_disconnect(void* handle, uint64_t session_id);
  late final int Function(Pointer<Void> handle, int sessionId) disconnect;

  // 7. rdcs_send_input — Send an input event (mouse or keyboard).
  //    int32_t rdcs_send_input(void* handle, uint64_t session_id,
  //                            const char* event_json);
  late final int Function(
      Pointer<Void> handle, int sessionId, Pointer<Utf8> eventJson) sendInput;

  // 8. rdcs_send_file — Initiate a file transfer.
  //    int32_t rdcs_send_file(void* handle, uint64_t session_id,
  //                           const char* path, const char* dest);
  late final int Function(
      Pointer<Void> handle,
      int sessionId,
      Pointer<Utf8> path,
      Pointer<Utf8> dest) sendFile;

  // 9. rdcs_send_message — Send a chat message to the remote session.
  //    int32_t rdcs_send_message(void* handle, uint64_t session_id,
  //                              const char* text);
  late final int Function(
      Pointer<Void> handle, int sessionId, Pointer<Utf8> text) sendMessage;

  // 10. rdcs_set_quality — Dynamically adjust streaming quality.
  //     mode: 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  //     int32_t rdcs_set_quality(void* handle, uint64_t session_id,
  //                              int32_t mode);
  late final int Function(Pointer<Void> handle, int sessionId, int mode)
      setQuality;

  // 11. rdcs_generate_invite — Generate a new 4-digit invite code.
  //     Returns a heap-allocated C string that must be freed with
  //     rdcs_free_string. Returns nullptr on error.
  //     char* rdcs_generate_invite(void* handle);
  late final Pointer<Utf8> Function(Pointer<Void> handle) generateInvite;

  // 12. rdcs_register_callback — Register a callback for engine events.
  //     event_id: one of the EVENT_* constants, or 0 to receive all events.
  //     int32_t rdcs_register_callback(void* handle, uint32_t event_id,
  //                                    void (*callback)(uint32_t, const char*,
  //                                                     size_t));
  late final int Function(Pointer<Void> handle, int eventId,
      Pointer<NativeFunction<EventCallbackNative>> callback) registerCallback;

  // 13. rdcs_free_string — Free a Rust-allocated C string.
  //     void rdcs_free_string(char* ptr);
  late final void Function(Pointer<Utf8> ptr) rdcsFreeString;

  // ── Bind all symbols ─────────────────────────────────────────

  void _bindFunctions() {
    engineCreate = _lib
        .lookup<
          NativeFunction<Pointer<Void> Function(Pointer<Utf8>)>
        >('rdcs_engine_create')
        .asFunction();

    engineDestroy = _lib
        .lookup<NativeFunction<Void Function(Pointer<Void>)>>(
          'rdcs_engine_destroy',
        )
        .asFunction();

    startCapture = _lib
        .lookup<
          NativeFunction<Int32 Function(Pointer<Void>, Pointer<Utf8>)>
        >('rdcs_start_capture')
        .asFunction();

    stopCapture = _lib
        .lookup<NativeFunction<Int32 Function(Pointer<Void>)>>(
          'rdcs_stop_capture',
        )
        .asFunction();

    connect = _lib
        .lookup<
          NativeFunction<Int32 Function(Pointer<Void>, Pointer<Utf8>)>
        >('rdcs_connect')
        .asFunction();

    disconnect = _lib
        .lookup<
          NativeFunction<Int32 Function(Pointer<Void>, Uint64)>
        >('rdcs_disconnect')
        .asFunction();

    sendInput = _lib
        .lookup<
          NativeFunction<
            Int32 Function(Pointer<Void>, Uint64, Pointer<Utf8>)
          >
        >('rdcs_send_input')
        .asFunction();

    sendFile = _lib
        .lookup<
          NativeFunction<
            Int32 Function(
                Pointer<Void>, Uint64, Pointer<Utf8>, Pointer<Utf8>)
          >
        >('rdcs_send_file')
        .asFunction();

    sendMessage = _lib
        .lookup<
          NativeFunction<
            Int32 Function(Pointer<Void>, Uint64, Pointer<Utf8>)
          >
        >('rdcs_send_message')
        .asFunction();

    setQuality = _lib
        .lookup<
          NativeFunction<Int32 Function(Pointer<Void>, Uint64, Int32)>
        >('rdcs_set_quality')
        .asFunction();

    generateInvite = _lib
        .lookup<
          NativeFunction<Pointer<Utf8> Function(Pointer<Void>)>
        >('rdcs_generate_invite')
        .asFunction();

    registerCallback = _lib
        .lookup<
          NativeFunction<
            Int32 Function(
                Pointer<Void>,
                Uint32,
                Pointer<NativeFunction<EventCallbackNative>>)
          >
        >('rdcs_register_callback')
        .asFunction();

    rdcsFreeString = _lib
        .lookup<NativeFunction<Void Function(Pointer<Utf8>)>>(
          'rdcs_free_string',
        )
        .asFunction();
  }
}
