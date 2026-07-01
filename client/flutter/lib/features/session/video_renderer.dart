// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/ffi/engine_events.dart';
import 'session_providers.dart';

/// Video renderer widget for displaying remote desktop frames.
///
/// Receives decoded BGRA frames from the Rust engine via EVENT_FRAME_READY
/// events and renders them using RawImage with decodeImageFromPixels.
class VideoRenderer extends ConsumerStatefulWidget {
  const VideoRenderer({super.key});

  @override
  ConsumerState<VideoRenderer> createState() => _VideoRendererState();
}

class _VideoRendererState extends ConsumerState<VideoRenderer> {
  ui.Image? _currentImage;
  int _frameWidth = 0;
  int _frameHeight = 0;

  // FPS tracking
  int _frameCount = 0;
  DateTime? _lastFpsUpdate;
  double _currentFps = 0.0;

  // Latency tracking
  int _lastFrameTimestamp = 0;
  int _latencyMs = 0;

  StreamSubscription<EngineEvent>? _eventSubscription;

  @override
  void initState() {
    super.initState();
    _subscribeToFrameEvents();
  }

  @override
  void dispose() {
    _eventSubscription?.cancel();
    _currentImage?.dispose();
    super.dispose();
  }

  /// Subscribe to ENGINE_FRAME_READY events from the Rust engine.
  void _subscribeToFrameEvents() {
    // TODO: Get event stream from engine provider
    // For now, this is a placeholder structure
    // _eventSubscription = ref.read(engineProvider).eventStream.listen((event) {
    //   if (event.type == EngineEventId.frameReady) {
    //     final payload = FramePayload.fromJson(event.payload);
    //     _onFrameReceived(payload);
    //   }
    // });
  }

  /// Handle a received video frame.
  void _onFrameReceived(FramePayload payload) {
    // Decode base64 BGRA data
    final Uint8List bgraData;
    try {
      bgraData = base64Decode(payload.dataBase64);
    } catch (e) {
      debugPrint('❌ Failed to decode frame data: $e');
      return;
    }

    // Verify data size
    final expectedSize = payload.width * payload.height * 4;
    if (bgraData.length != expectedSize) {
      debugPrint(
        '⚠️ Frame data size mismatch: expected $expectedSize, got ${bgraData.length}',
      );
      return;
    }

    // Decode pixels to Image
    ui.decodeImageFromPixels(
      bgraData,
      payload.width,
      payload.height,
      ui.PixelFormat.bgra8888,
      (ui.Image image) {
        if (mounted) {
          setState(() {
            _currentImage?.dispose();
            _currentImage = image;
            _frameWidth = payload.width;
            _frameHeight = payload.height;
            _lastFrameTimestamp = payload.timestamp;

            _updateFps();
            _updateLatency();
          });
        } else {
          image.dispose();
        }
      },
    );
  }

  /// Update FPS counter.
  void _updateFps() {
    _frameCount++;
    final now = DateTime.now();

    if (_lastFpsUpdate != null) {
      final elapsed = now.difference(_lastFpsUpdate!).inMilliseconds;
      if (elapsed >= 1000) {
        _currentFps = _frameCount * 1000 / elapsed;
        _frameCount = 0;
        _lastFpsUpdate = now;
      }
    } else {
      _lastFpsUpdate = now;
    }
  }

  /// Update latency measurement.
  void _updateLatency() {
    if (_lastFrameTimestamp > 0) {
      final now = DateTime.now().microsecondsSinceEpoch;
      _latencyMs = ((now - _lastFrameTimestamp) / 1000).round();
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_currentImage == null) {
      return const _VideoPlaceholder();
    }

    return Stack(
      children: [
        // Video frame
        Positioned.fill(
          child: RawImage(
            image: _currentImage,
            fit: BoxFit.contain,
            filterQuality: FilterQuality.medium,
          ),
        ),

        // Stats overlay
        Positioned(
          top: 8,
          right: 8,
          child: _StatsOverlay(
            fps: _currentFps,
            latency: _latencyMs,
            resolution: '$_frameWidth×$_frameHeight',
          ),
        ),
      ],
    );
  }
}

/// Statistics overlay showing FPS, latency, and resolution.
class _StatsOverlay extends StatelessWidget {
  const _StatsOverlay({
    required this.fps,
    required this.latency,
    required this.resolution,
  });

  final double fps;
  final int latency;
  final String resolution;

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.6),
        borderRadius: BorderRadius.circular(6),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.end,
        mainAxisSize: MainAxisSize.min,
        children: [
          _StatRow(
            label: 'FPS',
            value: fps.toStringAsFixed(1),
            color: _getFpsColor(fps),
          ),
          const SizedBox(height: 2),
          _StatRow(
            label: 'Delay',
            value: '${latency}ms',
            color: _getLatencyColor(latency),
          ),
          const SizedBox(height: 2),
          _StatRow(
            label: 'Res',
            value: resolution,
            color: Colors.white70,
          ),
        ],
      ),
    );
  }

  Color _getFpsColor(double fps) {
    if (fps >= 24) return Colors.greenAccent;
    if (fps >= 15) return Colors.yellowAccent;
    return Colors.redAccent;
  }

  Color _getLatencyColor(int latency) {
    if (latency <= 100) return Colors.greenAccent;
    if (latency <= 200) return Colors.yellowAccent;
    return Colors.redAccent;
  }
}

class _StatRow extends StatelessWidget {
  const _StatRow({
    required this.label,
    required this.value,
    required this.color,
  });

  final String label;
  final String value;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          '$label: ',
          style: const TextStyle(
            color: Colors.white54,
            fontSize: 11,
            fontFamily: 'monospace',
          ),
        ),
        Text(
          value,
          style: TextStyle(
            color: color,
            fontSize: 11,
            fontWeight: FontWeight.w600,
            fontFamily: 'monospace',
          ),
        ),
      ],
    );
  }
}

/// Placeholder widget shown when no video frames are available.
class _VideoPlaceholder extends StatelessWidget {
  const _VideoPlaceholder();

  @override
  Widget build(BuildContext context) {
    return Container(
      width: double.infinity,
      height: double.infinity,
      decoration: const BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            Color(0xFF1A1A2E),
            Color(0xFF16213E),
            Color(0xFF0F3460),
          ],
        ),
      ),
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.desktop_windows,
              size: 64,
              color: Colors.white.withOpacity(0.15),
            ),
            const SizedBox(height: 12),
            Text(
              'Waiting for video...',
              style: TextStyle(
                color: Colors.white.withOpacity(0.25),
                fontSize: 14,
              ),
            ),
            const SizedBox(height: 8),
            SizedBox(
              width: 24,
              height: 24,
              child: CircularProgressIndicator(
                strokeWidth: 2,
                valueColor: AlwaysStoppedAnimation<Color>(
                  Colors.white.withOpacity(0.2),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
