// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:convert';
import 'dart:ffi';
import 'dart:isolate';

import 'package:ffi/ffi.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'bindings.dart';
import 'engine_events.dart';

/// Commands sent from the main isolate to the engine isolate.
///
/// Each command carries a [replyTo] port so the engine isolate can
/// send a dedicated response, avoiding race conditions with
/// concurrent commands.
class _EngineCommand {
  _EngineCommand(this.type, this.replyTo, [this.payload]);

  final _CommandType type;
  final SendPort replyTo;
  final String? payload;
}

enum _CommandType {
  create,
  destroy,
  startCapture,
  stopCapture,
  connect,
  disconnect,
  sendInput,
  sendFile,
  sendMessage,
  setQuality,
  generateInvite,
}

/// Static native callback invoked by the Rust engine when an event fires.
///
/// Registered via `rdcs_register_callback` with `event_id = 0` (all events).
/// Converts the C string payload to a Dart string and forwards it through
/// the isolate's event [SendPort].
///
/// This function MUST be static and top-level so it can be converted to a
/// C function pointer via [Pointer.fromFunction].
void _nativeEventCallback(int eventId, Pointer<Utf8> payload, int payloadLen) {
  final port = EngineIsolate.eventSendPort;
  if (port != null) {
    // Copy the payload before the native side frees it.
    final json = payload == nullptr ? '{}' : payload.toDartString();
    port.send([eventId, json]);
  }
}

/// Manages communication with the Rust core engine via FFI in a
/// dedicated isolate.
///
/// All FFI calls happen on the background isolate to avoid blocking
/// the UI thread. Events from the engine are delivered via native
/// callbacks registered through `rdcs_register_callback`, which
/// forward to the main isolate via a [ReceivePort].
class EngineIsolate {
  EngineIsolate._();

  /// Factory constructor to create a new instance.
  factory EngineIsolate() => EngineIsolate._();

  Isolate? _isolate;
  SendPort? _commandPort;
  final _eventController = StreamController<EngineEvent>.broadcast();
  bool _disposed = false;

  /// Holds the most recent session ID returned by [connect].
  ///
  /// Used as a default for operations that require a session ID
  /// when the caller does not specify one.
  int currentSessionId = 0;

  /// Static reference to the event [SendPort] so the native callback
  /// (which must be a static function) can access it.
  static SendPort? eventSendPort;

  /// Stream of events emitted by the Rust engine.
  Stream<EngineEvent> get events => _eventController.stream;

  /// Initialise the background isolate and FFI bindings.
  Future<void> init() async {
    // Load library in main isolate first to cache the path
    final tempBindings = RdcsBindings();
    final libraryPath = RdcsBindings.cachedLibraryPath;
    print('📦 Cached library path for isolate: $libraryPath');

    final readyPort = ReceivePort();
    final eventPort = ReceivePort();

    // Expose the event SendPort for the static native callback.
    eventSendPort = eventPort.sendPort;

    _isolate = await Isolate.spawn(
      _isolateEntry,
      _IsolateInit(readyPort.sendPort, eventPort.sendPort, libraryPath),
    );

    // Listen for events pushed from the engine isolate.
    eventPort.listen((message) {
      if (_disposed) return;
      if (message is List && message.length >= 2) {
        try {
          _eventController.add(EngineEvent.fromCallback(message));
        } catch (_) {
          // Ignore malformed events.
        }
      }
    });

    // The isolate sends back its command port once ready.
    _commandPort = await readyPort.first as SendPort;
  }

  /// Creates a new engine instance with the given config JSON.
  /// Returns a result code (0 = success).
  Future<int> create(String configJson) async {
    return _sendCommand(_CommandType.create, configJson);
  }

  /// Destroys the current engine instance.
  Future<void> destroy() async {
    try {
      await _sendCommand(_CommandType.destroy);
    } catch (_) {
      // Destroy may fail if isolate is already shutting down.
    }
  }

  /// Starts screen capture with the given config JSON.
  Future<int> startCapture(String configJson) async {
    return _sendCommand(_CommandType.startCapture, configJson);
  }

  /// Stops the current screen capture.
  Future<int> stopCapture() async {
    return _sendCommand(_CommandType.stopCapture);
  }

  /// Connects to a remote device by its device code.
  /// Returns a positive session ID on success, or a negative error code.
  Future<int> connect(String targetCode) async {
    final sessionId = await _sendCommand(_CommandType.connect, targetCode);
    if (sessionId > 0) {
      currentSessionId = sessionId;
    }
    return sessionId;
  }

  /// Disconnects a specific remote session.
  Future<int> disconnect(int sessionId) async {
    return _sendCommand(_CommandType.disconnect, sessionId.toString());
  }

  /// Sends an input event (keyboard or mouse) to a remote session.
  ///
  /// [eventJson] is a JSON string describing the input event.
  Future<int> sendInput(int sessionId, String eventJson) async {
    final payload = jsonEncode({
      'session_id': sessionId,
      'event': eventJson,
    });
    return _sendCommand(_CommandType.sendInput, payload);
  }

  /// Initiates a file transfer to the remote session.
  ///
  /// [path] is the local file path; [dest] is the remote destination
  /// directory.
  Future<int> sendFile(int sessionId, String path, String dest) async {
    final payload = jsonEncode({
      'session_id': sessionId,
      'path': path,
      'dest': dest,
    });
    return _sendCommand(_CommandType.sendFile, payload);
  }

  /// Sends a chat message to the remote session.
  Future<int> sendMessage(int sessionId, String text) async {
    final payload = jsonEncode({
      'session_id': sessionId,
      'text': text,
    });
    return _sendCommand(_CommandType.sendMessage, payload);
  }

  /// Dynamically adjusts streaming quality for a session.
  ///
  /// [mode]: 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  Future<int> setQuality(int sessionId, int mode) async {
    final payload = jsonEncode({
      'session_id': sessionId,
      'mode': mode,
    });
    return _sendCommand(_CommandType.setQuality, payload);
  }

  /// Generates a new invite code for this device.
  /// Returns the invite code string, or throws on error.
  Future<String> generateInvite() async {
    return _sendCommandForString(_CommandType.generateInvite);
  }

  /// Disposes the isolate and cleans up resources.
  Future<void> dispose() async {
    _disposed = true;
    await destroy();
    eventSendPort = null;
    _isolate?.kill(priority: Isolate.immediate);
    _isolate = null;
    await _eventController.close();
  }

  // ── Internal ─────────────────────────────────────────────────

  /// Sends a command to the engine isolate and awaits the result.
  ///
  /// Each command creates a dedicated [ReceivePort] for its reply,
  /// so concurrent commands don't interfere with each other.
  Future<int> _sendCommand(_CommandType type, [String? payload]) async {
    if (_commandPort == null) {
      throw StateError('EngineIsolate not initialised. Call init() first.');
    }

    final responsePort = ReceivePort();
    try {
      _commandPort!.send(
        _EngineCommand(type, responsePort.sendPort, payload),
      );

      final response = await responsePort.first as List;
      // response format: [resultCode, errorMessage?]
      final code = response[0] as int;
      final error = response.length > 1 ? response[1] as String? : null;

      if (error != null) {
        throw EngineException(code, error);
      }
      return code;
    } finally {
      responsePort.close();
    }
  }

  /// Sends a command that returns a string result (for generateInvite).
  Future<String> _sendCommandForString(
      _CommandType type, [String? payload]) async {
    if (_commandPort == null) {
      throw StateError('EngineIsolate not initialised. Call init() first.');
    }

    final responsePort = ReceivePort();
    try {
      _commandPort!.send(
        _EngineCommand(type, responsePort.sendPort, payload),
      );

      final response = await responsePort.first as List;
      // response format: [resultString, errorMessage?]
      final result = response[0];
      final error = response.length > 1 ? response[1] as String? : null;

      if (error != null) {
        throw EngineException(-1, error);
      }
      return result as String;
    } finally {
      responsePort.close();
    }
  }

  /// Entry point for the background isolate.
  ///
  /// Loads FFI bindings, then listens for commands. For each command
  /// it invokes the corresponding Rust function and sends the result
  /// back on the command's reply port.
  static void _isolateEntry(_IsolateInit init) {
    final bindings = RdcsBindings(libraryPath: init.libraryPath);
    final commandPort = ReceivePort();

    // Send our command port back to the main isolate.
    init.commandPort.send(commandPort.sendPort);

    Pointer<Void>? engineHandle;

    commandPort.listen((message) {
      if (message is! _EngineCommand) return;

      final cmd = message;
      int result = 0;
      String? error;
      String? stringResult;

      try {
        switch (cmd.type) {
          case _CommandType.create:
            final configPtr = (cmd.payload ?? '{}').toNativeUtf8();
            final handle = bindings.engineCreate(configPtr);
            malloc.free(configPtr);
            if (handle == nullptr) {
              throw EngineException(0, 'Failed to create engine');
            }
            engineHandle = handle;

            // Register a catch-all callback (event_id=0) so the engine
            // pushes events to us instead of us polling.
            final callbackPtr = Pointer.fromFunction<
                Void Function(Uint32, Pointer<Utf8>, IntPtr)>(
              _nativeEventCallback,
            );
            bindings.registerCallback(engineHandle!, 0, callbackPtr);

          case _CommandType.destroy:
            if (engineHandle != null) {
              bindings.engineDestroy(engineHandle!);
              engineHandle = null;
            }

          case _CommandType.startCapture:
            _requireHandle(engineHandle);
            final configPtr = (cmd.payload ?? '{}').toNativeUtf8();
            result = bindings.startCapture(engineHandle!, configPtr);
            malloc.free(configPtr);

          case _CommandType.stopCapture:
            _requireHandle(engineHandle);
            result = bindings.stopCapture(engineHandle!);

          case _CommandType.connect:
            _requireHandle(engineHandle);
            final codePtr = (cmd.payload ?? '').toNativeUtf8();
            // Rust returns a positive session ID on success, negative
            // error code on failure.
            result = bindings.connect(engineHandle!, codePtr);
            malloc.free(codePtr);

          case _CommandType.disconnect:
            _requireHandle(engineHandle);
            final sessionId = int.parse(cmd.payload ?? '0');
            result = bindings.disconnect(engineHandle!, sessionId);

          case _CommandType.sendInput:
            _requireHandle(engineHandle);
            final data =
                jsonDecode(cmd.payload ?? '{}') as Map<String, dynamic>;
            final sessionId = (data['session_id'] as int?) ?? 0;
            final eventPtr =
                ((data['event'] as String?) ?? '{}').toNativeUtf8();
            result = bindings.sendInput(engineHandle!, sessionId, eventPtr);
            malloc.free(eventPtr);

          case _CommandType.sendFile:
            _requireHandle(engineHandle);
            final data =
                jsonDecode(cmd.payload ?? '{}') as Map<String, dynamic>;
            final sessionId = (data['session_id'] as int?) ?? 0;
            final pathPtr =
                ((data['path'] as String?) ?? '').toNativeUtf8();
            final destPtr =
                ((data['dest'] as String?) ?? '').toNativeUtf8();
            result = bindings.sendFile(
                engineHandle!, sessionId, pathPtr, destPtr);
            malloc.free(pathPtr);
            malloc.free(destPtr);

          case _CommandType.sendMessage:
            _requireHandle(engineHandle);
            final data =
                jsonDecode(cmd.payload ?? '{}') as Map<String, dynamic>;
            final sessionId = (data['session_id'] as int?) ?? 0;
            final textPtr =
                ((data['text'] as String?) ?? '').toNativeUtf8();
            result =
                bindings.sendMessage(engineHandle!, sessionId, textPtr);
            malloc.free(textPtr);

          case _CommandType.setQuality:
            _requireHandle(engineHandle);
            final data =
                jsonDecode(cmd.payload ?? '{}') as Map<String, dynamic>;
            final sessionId = (data['session_id'] as int?) ?? 0;
            final mode = (data['mode'] as int?) ?? 0;
            result = bindings.setQuality(engineHandle!, sessionId, mode);

          case _CommandType.generateInvite:
            _requireHandle(engineHandle);
            print('🔍 Calling generateInvite with handle: $engineHandle');
            final codePtr = bindings.generateInvite(engineHandle!);
            print('🔍 generateInvite returned pointer: $codePtr');
            if (codePtr == nullptr) {
              print('❌ Pointer is null!');
              throw EngineException(-1, 'Failed to generate invite code');
            }

            // Manual conversion to avoid isolate issues
            try {
              final utf8Ptr = codePtr.cast<Utf8>();
              final units = <int>[];
              var i = 0;
              while (true) {
                final byte = utf8Ptr.cast<Uint8>().elementAt(i).value;
                if (byte == 0) break;
                units.add(byte);
                i++;
                if (i > 1000) break; // Safety limit
              }
              stringResult = String.fromCharCodes(units);
              print('✅ Invite code: $stringResult');
            } finally {
              bindings.rdcsFreeString(codePtr);
            }
        }
      } on EngineException catch (e) {
        result = e.code;
        error = e.message;
      } catch (e) {
        result = -1;
        error = e.toString();
      }

      // Send response back to the caller.
      if (stringResult != null) {
        // String result path (generateInvite).
        if (error != null) {
          cmd.replyTo.send([stringResult, error]);
        } else {
          cmd.replyTo.send([stringResult!]);
        }
      } else if (error != null) {
        cmd.replyTo.send([result, error]);
      } else {
        cmd.replyTo.send([result]);
      }
    });
  }

  static void _requireHandle(Pointer<Void>? handle) {
    if (handle == null) {
      throw EngineException(-1, 'Engine not created');
    }
  }
}

/// Exception thrown by the engine isolate.
class EngineException implements Exception {
  EngineException(this.code, this.message);

  final int code;
  final String message;

  @override
  String toString() => 'EngineException($code): $message';
}

/// Initialisation message sent to the engine isolate.
class _IsolateInit {
  _IsolateInit(this.commandPort, this.eventPort, this.libraryPath);
  final SendPort commandPort;
  final SendPort eventPort;
  final String? libraryPath;
}

// ── Riverpod provider ──────────────────────────────────────────

/// Provides the [EngineIsolate] singleton.
///
/// Initialise at app startup:
/// ```dart
/// await ref.read(engineProvider).init();
/// ```
final engineProvider = Provider<EngineIsolate>((ref) {
  final isolate = EngineIsolate._();

  ref.onDispose(() {
    isolate.dispose();
  });

  return isolate;
});
