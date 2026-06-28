// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:convert';

/// Event IDs emitted by the Rust core engine.
///
/// These IDs MUST stay in sync with the Rust constants in
/// `rdcs_ffi::lib.rs` (EVENT_CONNECTION_REQUEST, etc.).
enum EngineEventId {
  /// Incoming connection request from a remote device.
  connectionRequest(1),

  /// Connection to remote device successfully established.
  connectionEstablished(2),

  /// Connection to remote device was lost.
  connectionLost(3),

  /// Connection to remote device was restored after interruption.
  connectionRestored(4),

  /// A new video frame is ready from the remote side.
  frameReady(5),

  /// An input event was received from the remote side.
  inputReceived(6),

  /// File transfer progress update.
  fileTransferProgress(7),

  /// File transfer completed.
  fileTransferComplete(8),

  /// A chat message was received from the remote side.
  chatMessage(9),

  /// Quality mode changed.
  qualityChanged(10),

  /// A nearby device was discovered.
  nearbyDeviceFound(11),

  /// A nearby device was lost.
  nearbyDeviceLost(12);

  const EngineEventId(this.value);
  final int value;

  static EngineEventId? fromValue(int v) {
    for (final id in EngineEventId.values) {
      if (id.value == v) return id;
    }
    return null;
  }
}

/// An event emitted by the Rust core engine.
///
/// Events are delivered via native callbacks registered through
/// `rdcs_register_callback`. Each callback receives an event ID and
/// a JSON payload string.
class EngineEvent {
  EngineEvent({
    required this.eventId,
    required this.payloadJson,
  });

  /// Numeric event identifier (see [EngineEventId]).
  final int eventId;

  /// Raw JSON payload string from the engine.
  final String payloadJson;

  /// Parsed event type, or `null` if the ID is unknown.
  EngineEventId? get type => EngineEventId.fromValue(eventId);

  /// Decode the payload JSON into a Dart map.
  Map<String, dynamic> get payload =>
      jsonDecode(payloadJson) as Map<String, dynamic>;

  /// Create an [EngineEvent] from a callback delivery list.
  ///
  /// The native callback sends `[eventId, payloadJson]` through the
  /// isolate [SendPort]. This factory reconstructs the event.
  factory EngineEvent.fromCallback(List message) {
    return EngineEvent(
      eventId: message[0] as int,
      payloadJson: message[1] as String,
    );
  }

  /// Create an [EngineEvent] from a raw JSON envelope.
  ///
  /// Expected format: `{"event_id": N, "payload": {...}}`.
  factory EngineEvent.fromJson(String json) {
    final map = jsonDecode(json) as Map<String, dynamic>;
    return EngineEvent(
      eventId: map['event_id'] as int,
      payloadJson: jsonEncode(map['payload'] ?? {}),
    );
  }

  @override
  String toString() =>
      'EngineEvent(id: $eventId, type: ${type?.name ?? "unknown"})';
}

// ── Typed payload models ───────────────────────────────────────

/// Payload for [EngineEventId.frameReady].
class FramePayload {
  FramePayload({
    required this.width,
    required this.height,
    required this.format,
    required this.dataBase64,
    required this.timestamp,
  });

  final int width;
  final int height;
  final String format;
  final String dataBase64;
  final int timestamp;

  factory FramePayload.fromJson(Map<String, dynamic> json) {
    return FramePayload(
      width: json['width'] as int,
      height: json['height'] as int,
      format: json['format'] as String,
      dataBase64: json['data'] as String,
      timestamp: json['timestamp'] as int,
    );
  }
}

/// Payload for [EngineEventId.inputReceived].
///
/// Represents either a keyboard or mouse event received from the
/// remote side. The `inputType` field distinguishes between them:
/// `"key"` for keyboard, `"mouse"` for mouse.
class InputPayload {
  InputPayload({
    required this.inputType,
    required this.data,
  });

  /// `"key"` or `"mouse"`.
  final String inputType;

  /// Raw event data — keys depend on [inputType].
  final Map<String, dynamic> data;

  factory InputPayload.fromJson(Map<String, dynamic> json) {
    return InputPayload(
      inputType: json['type'] as String? ?? 'key',
      data: json,
    );
  }
}

/// Payload for [EngineEventId.fileTransferProgress] and
/// [EngineEventId.fileTransferComplete].
class FileTransferPayload {
  FileTransferPayload({
    required this.transferId,
    required this.fileName,
    required this.bytesTransferred,
    required this.totalBytes,
  });

  final String transferId;
  final String fileName;
  final int bytesTransferred;
  final int totalBytes;

  double get progress =>
      totalBytes > 0 ? bytesTransferred / totalBytes : 0;

  factory FileTransferPayload.fromJson(Map<String, dynamic> json) {
    return FileTransferPayload(
      transferId: json['transfer_id'] as String,
      fileName: json['file_name'] as String,
      bytesTransferred: json['bytes_transferred'] as int,
      totalBytes: json['total_bytes'] as int,
    );
  }
}

/// Payload for [EngineEventId.chatMessage].
class ChatMessagePayload {
  ChatMessagePayload({
    required this.text,
    this.sender = '',
  });

  final String text;
  final String sender;

  factory ChatMessagePayload.fromJson(Map<String, dynamic> json) {
    return ChatMessagePayload(
      text: json['text'] as String? ?? '',
      sender: json['sender'] as String? ?? '',
    );
  }
}

/// Payload for [EngineEventId.connectionEstablished].
class ConnectionEstablishedPayload {
  ConnectionEstablishedPayload({
    required this.sessionId,
  });

  final int sessionId;

  factory ConnectionEstablishedPayload.fromJson(Map<String, dynamic> json) {
    return ConnectionEstablishedPayload(
      sessionId: json['session_id'] as int,
    );
  }
}

/// Payload for [EngineEventId.connectionLost].
class ConnectionLostPayload {
  ConnectionLostPayload({
    required this.sessionId,
  });

  final int sessionId;

  factory ConnectionLostPayload.fromJson(Map<String, dynamic> json) {
    return ConnectionLostPayload(
      sessionId: json['session_id'] as int? ?? 0,
    );
  }
}

/// Payload for [EngineEventId.qualityChanged].
class QualityChangedPayload {
  QualityChangedPayload({
    required this.mode,
  });

  /// 0 = auto, 1 = clarity priority, 2 = fluidity priority.
  final int mode;

  factory QualityChangedPayload.fromJson(Map<String, dynamic> json) {
    return QualityChangedPayload(
      mode: json['mode'] as int? ?? 0,
    );
  }
}

/// Payload for [EngineEventId.nearbyDeviceFound] and
/// [EngineEventId.nearbyDeviceLost].
class NearbyDevicePayload {
  NearbyDevicePayload({
    required this.deviceCode,
    this.deviceName = '',
  });

  final String deviceCode;
  final String deviceName;

  factory NearbyDevicePayload.fromJson(Map<String, dynamic> json) {
    return NearbyDevicePayload(
      deviceCode: json['device_code'] as String? ?? '',
      deviceName: json['device_name'] as String? ?? '',
    );
  }
}
