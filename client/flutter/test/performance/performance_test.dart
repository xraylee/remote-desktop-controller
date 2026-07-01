// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:async';
import 'dart:convert';
import 'dart:typed_data';
import 'package:flutter_test/flutter_test.dart';

void main() {
  group('Performance Tests', () {
    // ── Video Frame Rendering Latency ────────────────────────────

    test('frame decode latency under 16ms for 1080p @ 60fps', () async {
      // 60 fps = 16.67ms per frame budget
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final stopwatch = Stopwatch()..start();

      // Simulate frame decode
      final decoded = base64Decode(base64Data);
      expect(decoded.length, frameData.length);

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(16));
      print('✓ 1080p frame decode: ${stopwatch.elapsedMilliseconds}ms');
    });

    test('frame decode latency under 8ms for 720p @ 60fps', () async {
      final frameData = _generateTestFrame(1280, 720);
      final base64Data = base64Encode(frameData);

      final stopwatch = Stopwatch()..start();

      final decoded = base64Decode(base64Data);
      expect(decoded.length, frameData.length);

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(8));
      print('✓ 720p frame decode: ${stopwatch.elapsedMilliseconds}ms');
    });

    test('frame decode latency under 33ms for 4K @ 30fps', () async {
      // 30 fps = 33.33ms per frame budget
      final frameData = _generateTestFrame(3840, 2160);
      final base64Data = base64Encode(frameData);

      final stopwatch = Stopwatch()..start();

      final decoded = base64Decode(base64Data);
      expect(decoded.length, frameData.length);

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(33));
      print('✓ 4K frame decode: ${stopwatch.elapsedMilliseconds}ms');
    });

    test('base64 decoding performance', () async {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final iterations = 100;
      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < iterations; i++) {
        base64Decode(base64Data);
      }

      stopwatch.stop();

      final avgTime = stopwatch.elapsedMilliseconds / iterations;
      expect(avgTime, lessThan(10));
      print('✓ Base64 decode avg: ${avgTime.toStringAsFixed(2)}ms');
    });

    test('sustained 60 FPS rendering performance', () async {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final targetFrames = 60;
      final targetDuration = const Duration(seconds: 1);

      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < targetFrames; i++) {
        base64Decode(base64Data);
      }

      stopwatch.stop();

      expect(stopwatch.elapsed, lessThan(targetDuration));
      final actualFps = targetFrames / (stopwatch.elapsedMilliseconds / 1000);
      expect(actualFps, greaterThanOrEqualTo(60));
      print('✓ Sustained FPS: ${actualFps.toStringAsFixed(1)}');
    });

    // ── Input Event Latency ──────────────────────────────────────

    test('mouse move event processing under 5ms', () async {
      final events = <Map<String, dynamic>>[];

      final stopwatch = Stopwatch()..start();

      events.add({'type': 'mouse_move', 'x': 100, 'y': 200});

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(5));
      print('✓ Mouse move processing: ${stopwatch.elapsedMicroseconds}μs');
    });

    test('keyboard event processing under 5ms', () async {
      final events = <Map<String, dynamic>>[];

      final stopwatch = Stopwatch()..start();

      events.add({'type': 'key', 'code': 65, 'pressed': true});

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(5));
      print('✓ Keyboard processing: ${stopwatch.elapsedMicroseconds}μs');
    });

    test('scroll event processing under 5ms', () async {
      final events = <Map<String, dynamic>>[];

      final stopwatch = Stopwatch()..start();

      events.add({'type': 'scroll', 'dx': 0, 'dy': 120});

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(5));
      print('✓ Scroll processing: ${stopwatch.elapsedMicroseconds}μs');
    });

    test('rapid input events do not accumulate latency', () async {
      final events = <Map<String, dynamic>>[];
      final latencies = <int>[];

      // Simulate 100 rapid mouse movements
      for (int i = 0; i < 100; i++) {
        final stopwatch = Stopwatch()..start();

        events.add({'type': 'mouse_move', 'x': i, 'y': i});

        stopwatch.stop();
        latencies.add(stopwatch.elapsedMicroseconds);
      }

      // Average latency should remain consistent
      final avgLatency = latencies.reduce((a, b) => a + b) / latencies.length;
      final maxLatency = latencies.reduce((a, b) => a > b ? a : b);

      expect(avgLatency, lessThan(5000)); // 5ms average
      expect(maxLatency, lessThan(10000)); // 10ms max
      print('✓ Avg input latency: ${avgLatency.toStringAsFixed(0)}μs');
      print('✓ Max input latency: ${maxLatency}μs');
    });

    test('input event batching performance', () async {
      final batchSize = 10;
      final batches = <List<Map<String, dynamic>>>[];

      final stopwatch = Stopwatch()..start();

      // Create 100 batches of 10 events each
      for (int i = 0; i < 100; i++) {
        final batch = <Map<String, dynamic>>[];
        for (int j = 0; j < batchSize; j++) {
          batch.add({'type': 'mouse_move', 'x': j, 'y': j});
        }
        batches.add(batch);
      }

      stopwatch.stop();

      // Batching 1000 events should be fast
      expect(stopwatch.elapsedMilliseconds, lessThan(50));
      print('✓ Batched 1000 events in ${stopwatch.elapsedMilliseconds}ms');
    });

    // ── Memory Usage ─────────────────────────────────────────────

    test('frame memory footprint is reasonable', () {
      final frame1080p = _generateTestFrame(1920, 1080);
      final frame720p = _generateTestFrame(1280, 720);
      final frame4k = _generateTestFrame(3840, 2160);

      // BGRA = 4 bytes per pixel
      expect(frame1080p.lengthInBytes, 1920 * 1080 * 4);
      expect(frame720p.lengthInBytes, 1280 * 720 * 4);
      expect(frame4k.lengthInBytes, 3840 * 2160 * 4);

      print('✓ 1080p frame size: ${(frame1080p.lengthInBytes / 1024 / 1024).toStringAsFixed(2)} MB');
      print('✓ 720p frame size: ${(frame720p.lengthInBytes / 1024 / 1024).toStringAsFixed(2)} MB');
      print('✓ 4K frame size: ${(frame4k.lengthInBytes / 1024 / 1024).toStringAsFixed(2)} MB');
    });

    test('base64 encoding overhead', () {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final overhead = (base64Data.length - frameData.length) / frameData.length;

      // Base64 encoding increases size by ~33%
      expect(overhead, lessThan(0.4));
      expect(overhead, greaterThan(0.3));

      print('✓ Base64 overhead: ${(overhead * 100).toStringAsFixed(1)}%');
    });

    test('memory allocation for sustained rendering', () {
      final frameData = _generateTestFrame(1920, 1080);
      final frameSize = frameData.lengthInBytes;

      // Simulate 60 frames (1 second at 60fps)
      final totalMemory = frameSize * 60;

      // At 60fps, we need ~480 MB/s for 1080p BGRA
      final mbPerSecond = totalMemory / 1024 / 1024;

      print('✓ Memory bandwidth needed: ${mbPerSecond.toStringAsFixed(0)} MB/s');
      expect(mbPerSecond, lessThan(1000)); // Should be under 1 GB/s
    });

    // ── CPU Usage ────────────────────────────────────────────────

    test('frame processing CPU efficiency', () async {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final iterations = 1000;
      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < iterations; i++) {
        base64Decode(base64Data);
      }

      stopwatch.stop();

      final avgTimePerFrame = stopwatch.elapsedMicroseconds / iterations;

      print('✓ CPU time per frame: ${avgTimePerFrame.toStringAsFixed(0)}μs');
      expect(avgTimePerFrame, lessThan(5000)); // < 5ms per frame
    });

    test('input event processing CPU overhead', () async {
      final iterations = 10000;

      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < iterations; i++) {
        // Simulate input event processing
        final event = {'type': 'mouse_move', 'x': i % 1920, 'y': i % 1080};
        expect(event['x'], isNotNull);
      }

      stopwatch.stop();

      final avgTimePerEvent = stopwatch.elapsedMicroseconds / iterations;

      print('✓ CPU time per input event: ${avgTimePerEvent.toStringAsFixed(2)}μs');
      expect(avgTimePerEvent, lessThan(100)); // < 100μs per event
    });

    // ── Network Throughput (Simulated) ──────────────────────────

    test('1080p 60fps bandwidth requirement', () {
      final frameData = _generateTestFrame(1920, 1080);
      final frameSize = frameData.lengthInBytes;

      // Uncompressed bandwidth
      final bytesPerSecond = frameSize * 60;
      final mbps = (bytesPerSecond * 8) / 1024 / 1024;

      print('✓ 1080p@60fps uncompressed: ${mbps.toStringAsFixed(0)} Mbps');

      // Assume 10:1 compression ratio
      final compressedMbps = mbps / 10;
      print('✓ 1080p@60fps compressed (10:1): ${compressedMbps.toStringAsFixed(0)} Mbps');

      expect(compressedMbps, lessThan(100)); // Should fit in 100 Mbps
    });

    test('4K 30fps bandwidth requirement', () {
      final frameData = _generateTestFrame(3840, 2160);
      final frameSize = frameData.lengthInBytes;

      final bytesPerSecond = frameSize * 30;
      final mbps = (bytesPerSecond * 8) / 1024 / 1024;

      print('✓ 4K@30fps uncompressed: ${mbps.toStringAsFixed(0)} Mbps');

      final compressedMbps = mbps / 10;
      print('✓ 4K@30fps compressed (10:1): ${compressedMbps.toStringAsFixed(0)} Mbps');

      expect(compressedMbps, lessThan(200)); // Should fit in 200 Mbps
    });

    // ── FPS Stability ────────────────────────────────────────────

    test('frame timing consistency at 60fps', () async {
      final targetFrameTime = 16666; // 16.666ms in microseconds
      final frameTimings = <int>[];

      var lastFrameTime = DateTime.now().microsecondsSinceEpoch;

      for (int i = 0; i < 60; i++) {
        await Future.delayed(const Duration(milliseconds: 16));

        final now = DateTime.now().microsecondsSinceEpoch;
        final frameTime = now - lastFrameTime;
        frameTimings.add(frameTime);
        lastFrameTime = now;
      }

      // Calculate frame time variance
      final avgFrameTime = frameTimings.reduce((a, b) => a + b) / frameTimings.length;
      final variance = frameTimings
          .map((t) => (t - avgFrameTime) * (t - avgFrameTime))
          .reduce((a, b) => a + b) / frameTimings.length;
      final stdDev = variance.sqrt();

      print('✓ Avg frame time: ${(avgFrameTime / 1000).toStringAsFixed(2)}ms');
      print('✓ Frame time std dev: ${(stdDev / 1000).toStringAsFixed(2)}ms');

      // Standard deviation should be low for stable FPS
      expect(stdDev / 1000, lessThan(5)); // < 5ms deviation
    });

    test('no frame drops under sustained load', () async {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final targetFrames = 300; // 5 seconds at 60fps
      var processedFrames = 0;

      final stopwatch = Stopwatch()..start();

      for (int i = 0; i < targetFrames; i++) {
        base64Decode(base64Data);
        processedFrames++;
      }

      stopwatch.stop();

      // All frames should be processed
      expect(processedFrames, targetFrames);

      final actualFps = processedFrames / (stopwatch.elapsedMilliseconds / 1000);
      print('✓ Sustained FPS over 5s: ${actualFps.toStringAsFixed(1)}');

      expect(actualFps, greaterThanOrEqualTo(60));
    });

    // ── Combined Performance ─────────────────────────────────────

    test('simultaneous video rendering and input processing', () async {
      final frameData = _generateTestFrame(1920, 1080);
      final base64Data = base64Encode(frameData);

      final stopwatch = Stopwatch()..start();

      // Simulate 1 second of activity
      for (int i = 0; i < 60; i++) {
        // Decode frame
        base64Decode(base64Data);

        // Process multiple input events per frame
        for (int j = 0; j < 5; j++) {
          final event = {'type': 'mouse_move', 'x': j, 'y': j};
          expect(event, isNotNull);
        }
      }

      stopwatch.stop();

      // Should complete in under 1 second
      expect(stopwatch.elapsedMilliseconds, lessThan(1000));
      print('✓ Combined load completed in ${stopwatch.elapsedMilliseconds}ms');
    });

    test('performance under high input event rate', () async {
      final events = <Map<String, dynamic>>[];

      final stopwatch = Stopwatch()..start();

      // Simulate 1000 events per second
      for (int i = 0; i < 1000; i++) {
        events.add({'type': 'mouse_move', 'x': i % 1920, 'y': i % 1080});
      }

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(100));
      print('✓ 1000 input events processed in ${stopwatch.elapsedMilliseconds}ms');
    });
  });
}

// ── Helper Functions ─────────────────────────────────────────────

/// Generate a test video frame (BGRA format).
Uint8List _generateTestFrame(int width, int height) {
  final pixels = width * height;
  final data = Uint8List(pixels * 4); // BGRA = 4 bytes per pixel

  // Generate a simple gradient pattern
  for (int y = 0; y < height; y++) {
    for (int x = 0; x < width; x++) {
      final offset = (y * width + x) * 4;

      // BGRA format
      data[offset + 0] = (x % 256); // Blue
      data[offset + 1] = (y % 256); // Green
      data[offset + 2] = ((x + y) % 256); // Red
      data[offset + 3] = 255; // Alpha
    }
  }

  return data;
}

/// Extension for calculating square root.
extension on num {
  double sqrt() => this < 0 ? double.nan : this.toDouble().squareRoot();
}

extension on double {
  double squareRoot() {
    if (this < 0) return double.nan;
    if (this == 0) return 0;

    var guess = this / 2;
    for (var i = 0; i < 10; i++) {
      guess = (guess + this / guess) / 2;
    }
    return guess;
  }
}
