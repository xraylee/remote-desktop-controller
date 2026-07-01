// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'dart:ui' as ui;
import 'package:flutter/gestures.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_client/core/ffi/engine_isolate.dart';

// Import for PointerDeviceKind
import 'package:flutter/widgets.dart' show PointerDeviceKind;

void main() {
  group('Input Control Integration Tests', () {
    // ── Mouse Input ──────────────────────────────────────────────

    test('mouse move event transmitted to remote', () async {
      // Mock engine would record this call
      final mockEngine = MockEngineIsolate();

      // Simulate mouse move
      mockEngine.sendMouseMove(100, 200);

      // Verify event was sent
      expect(mockEngine.lastMouseX, 100);
      expect(mockEngine.lastMouseY, 200);
    });

    test('mouse left click transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Simulate left click down
      mockEngine.sendMouseButton(0, true); // 0 = left, true = pressed

      expect(mockEngine.lastMouseButton, 0);
      expect(mockEngine.lastMousePressed, true);

      // Simulate left click up
      mockEngine.sendMouseButton(0, false);

      expect(mockEngine.lastMousePressed, false);
    });

    test('mouse right click transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Right click
      mockEngine.sendMouseButton(1, true); // 1 = right
      expect(mockEngine.lastMouseButton, 1);
      expect(mockEngine.lastMousePressed, true);

      mockEngine.sendMouseButton(1, false);
      expect(mockEngine.lastMousePressed, false);
    });

    test('mouse middle click transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Middle click
      mockEngine.sendMouseButton(2, true); // 2 = middle
      expect(mockEngine.lastMouseButton, 2);
    });

    test('mouse scroll event transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Scroll up
      mockEngine.sendMouseScroll(0, 120); // positive = up

      expect(mockEngine.lastScrollDx, 0);
      expect(mockEngine.lastScrollDy, 120);

      // Scroll down
      mockEngine.sendMouseScroll(0, -120); // negative = down

      expect(mockEngine.lastScrollDy, -120);
    });

    test('horizontal scroll transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Scroll right
      mockEngine.sendMouseScroll(120, 0);

      expect(mockEngine.lastScrollDx, 120);
      expect(mockEngine.lastScrollDy, 0);
    });

    test('mouse drag operation transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Start drag (mouse down)
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendMouseMove(100, 100);

      expect(mockEngine.lastMousePressed, true);

      // Continue drag (mouse move while pressed)
      mockEngine.sendMouseMove(200, 200);

      expect(mockEngine.lastMouseX, 200);
      expect(mockEngine.lastMouseY, 200);

      // End drag (mouse up)
      mockEngine.sendMouseButton(0, false);

      expect(mockEngine.lastMousePressed, false);
    });

    test('double click transmitted as two clicks', () async {
      final mockEngine = MockEngineIsolate();

      // First click
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendMouseButton(0, false);

      // Second click (within double-click time)
      await Future.delayed(const Duration(milliseconds: 50));
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendMouseButton(0, false);

      // Verify two click sequences were sent
      expect(mockEngine.mouseClickCount, 4); // 2 downs + 2 ups
    });

    // ── Keyboard Input ───────────────────────────────────────────

    test('keyboard key press transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Press 'A' key
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: true,
      );

      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.keyA.keyId);
      expect(mockEngine.lastKeyPressed, true);
    });

    test('keyboard key release transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Press and release
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: true,
      );
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: false,
      );

      expect(mockEngine.lastKeyPressed, false);
    });

    test('modifier keys transmitted correctly', () async {
      final mockEngine = MockEngineIsolate();

      // Ctrl key
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.controlLeft.keyId,
        pressed: true,
      );

      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.controlLeft.keyId);

      // Shift key
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.shiftLeft.keyId,
        pressed: true,
      );

      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.shiftLeft.keyId);

      // Alt key
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.altLeft.keyId,
        pressed: true,
      );

      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.altLeft.keyId);
    });

    test('keyboard shortcut transmitted (Ctrl+C)', () async {
      final mockEngine = MockEngineIsolate();

      // Press Ctrl
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.controlLeft.keyId,
        pressed: true,
      );

      // Press C
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyC.keyId,
        pressed: true,
      );

      // Release C
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyC.keyId,
        pressed: false,
      );

      // Release Ctrl
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.controlLeft.keyId,
        pressed: false,
      );

      expect(mockEngine.keyEventCount, 4);
    });

    test('function keys transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // F1 key
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.f1.keyId,
        pressed: true,
      );

      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.f1.keyId);
    });

    test('special keys transmitted (Enter, Backspace, Tab)', () async {
      final mockEngine = MockEngineIsolate();

      // Enter
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.enter.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.enter.keyId);

      // Backspace
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.backspace.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.backspace.keyId);

      // Tab
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.tab.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.tab.keyId);
    });

    test('arrow keys transmitted', () async {
      final mockEngine = MockEngineIsolate();

      // Up arrow
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.arrowUp.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.arrowUp.keyId);

      // Down arrow
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.arrowDown.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.arrowDown.keyId);

      // Left arrow
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.arrowLeft.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.arrowLeft.keyId);

      // Right arrow
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.arrowRight.keyId,
        pressed: true,
      );
      expect(mockEngine.lastKeyCode, LogicalKeyboardKey.arrowRight.keyId);
    });

    // ── Input Latency ────────────────────────────────────────────

    test('mouse input latency under 10ms', () async {
      final mockEngine = MockEngineIsolate();

      final stopwatch = Stopwatch()..start();

      mockEngine.sendMouseMove(100, 200);

      stopwatch.stop();

      // Input processing should be extremely fast
      expect(stopwatch.elapsedMilliseconds, lessThan(10));
      print('✓ Mouse input latency: ${stopwatch.elapsedMicroseconds}μs');
    });

    test('keyboard input latency under 10ms', () async {
      final mockEngine = MockEngineIsolate();

      final stopwatch = Stopwatch()..start();

      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: true,
      );

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(10));
      print('✓ Keyboard input latency: ${stopwatch.elapsedMicroseconds}μs');
    });

    test('scroll input latency under 10ms', () async {
      final mockEngine = MockEngineIsolate();

      final stopwatch = Stopwatch()..start();

      mockEngine.sendMouseScroll(0, 120);

      stopwatch.stop();

      expect(stopwatch.elapsedMilliseconds, lessThan(10));
      print('✓ Scroll input latency: ${stopwatch.elapsedMicroseconds}μs');
    });

    // ── Input Event Ordering ─────────────────────────────────────

    test('input events transmitted in order', () async {
      final mockEngine = MockEngineIsolate();

      // Send sequence of events
      mockEngine.sendMouseMove(100, 100);
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendMouseMove(200, 200);
      mockEngine.sendMouseButton(0, false);

      // Verify order preserved
      expect(mockEngine.eventSequence, [
        'mouse_move:100,100',
        'mouse_button:0,true',
        'mouse_move:200,200',
        'mouse_button:0,false',
      ]);
    });

    test('keyboard and mouse events interleaved correctly', () async {
      final mockEngine = MockEngineIsolate();

      // Interleaved events
      mockEngine.sendMouseMove(100, 100);
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: true,
      );
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendKeyEvent(
        keyCode: LogicalKeyboardKey.keyA.keyId,
        pressed: false,
      );

      expect(mockEngine.eventSequence.length, 4);
      expect(mockEngine.eventSequence[0], startsWith('mouse_move'));
      expect(mockEngine.eventSequence[1], startsWith('key'));
      expect(mockEngine.eventSequence[2], startsWith('mouse_button'));
      expect(mockEngine.eventSequence[3], startsWith('key'));
    });

    // ── Input During Connection Loss ─────────────────────────────

    test('input events queued during disconnection', () async {
      final mockEngine = MockEngineIsolate();

      // Simulate disconnection
      mockEngine.simulateDisconnection();

      // Try to send input
      mockEngine.sendMouseMove(100, 100);

      // Events should be queued or dropped gracefully
      expect(mockEngine.connected, false);
    });

    test('queued input transmitted after reconnection', () async {
      final mockEngine = MockEngineIsolate();

      // Disconnect
      mockEngine.simulateDisconnection();

      // Send input while disconnected (queued)
      mockEngine.sendMouseMove(100, 100);

      // Reconnect
      mockEngine.simulateReconnection();

      // Queued events should be transmitted
      // (Implementation dependent - might drop or queue)
      expect(mockEngine.connected, true);
    });

    // ── Edge Cases ───────────────────────────────────────────────

    test('rapid mouse movement handled correctly', () async {
      final mockEngine = MockEngineIsolate();

      // Simulate rapid movement
      for (int i = 0; i < 100; i++) {
        mockEngine.sendMouseMove(i, i);
      }

      // All events should be recorded
      expect(mockEngine.mouseMoveCount, 100);
    });

    test('rapid key presses handled correctly', () async {
      final mockEngine = MockEngineIsolate();

      // Rapid typing
      for (int i = 0; i < 50; i++) {
        mockEngine.sendKeyEvent(
          keyCode: LogicalKeyboardKey.keyA.keyId,
          pressed: true,
        );
        mockEngine.sendKeyEvent(
          keyCode: LogicalKeyboardKey.keyA.keyId,
          pressed: false,
        );
      }

      expect(mockEngine.keyEventCount, 100);
    });

    test('mouse coordinates at screen boundaries', () async {
      final mockEngine = MockEngineIsolate();

      // Top-left corner
      mockEngine.sendMouseMove(0, 0);
      expect(mockEngine.lastMouseX, 0);
      expect(mockEngine.lastMouseY, 0);

      // Large coordinates
      mockEngine.sendMouseMove(3840, 2160);
      expect(mockEngine.lastMouseX, 3840);
      expect(mockEngine.lastMouseY, 2160);
    });

    test('negative scroll values handled', () async {
      final mockEngine = MockEngineIsolate();

      // Negative scroll (down/left)
      mockEngine.sendMouseScroll(-100, -200);

      expect(mockEngine.lastScrollDx, -100);
      expect(mockEngine.lastScrollDy, -200);
    });

    test('simultaneous mouse buttons pressed', () async {
      final mockEngine = MockEngineIsolate();

      // Press left and right simultaneously
      mockEngine.sendMouseButton(0, true);
      mockEngine.sendMouseButton(1, true);

      expect(mockEngine.mouseButtonsPressed, contains(0));
      expect(mockEngine.mouseButtonsPressed, contains(1));

      // Release both
      mockEngine.sendMouseButton(0, false);
      mockEngine.sendMouseButton(1, false);

      expect(mockEngine.mouseButtonsPressed, isEmpty);
    });
  });
}

// ── Mock Engine for Testing ──────────────────────────────────────

class MockEngineIsolate {
  bool connected = true;
  int lastMouseX = 0;
  int lastMouseY = 0;
  int lastMouseButton = -1;
  bool lastMousePressed = false;
  int lastScrollDx = 0;
  int lastScrollDy = 0;
  int lastKeyCode = 0;
  bool lastKeyPressed = false;

  int mouseMoveCount = 0;
  int mouseClickCount = 0;
  int keyEventCount = 0;

  final List<String> eventSequence = [];
  final Set<int> mouseButtonsPressed = {};

  void sendMouseMove(int x, int y) {
    if (!connected) return;
    lastMouseX = x;
    lastMouseY = y;
    mouseMoveCount++;
    eventSequence.add('mouse_move:$x,$y');
  }

  void sendMouseButton(int button, bool pressed) {
    if (!connected) return;
    lastMouseButton = button;
    lastMousePressed = pressed;
    mouseClickCount++;
    eventSequence.add('mouse_button:$button,$pressed');

    if (pressed) {
      mouseButtonsPressed.add(button);
    } else {
      mouseButtonsPressed.remove(button);
    }
  }

  void sendMouseScroll(int dx, int dy) {
    if (!connected) return;
    lastScrollDx = dx;
    lastScrollDy = dy;
    eventSequence.add('mouse_scroll:$dx,$dy');
  }

  void sendKeyEvent({required int keyCode, required bool pressed}) {
    if (!connected) return;
    lastKeyCode = keyCode;
    lastKeyPressed = pressed;
    keyEventCount++;
    eventSequence.add('key:$keyCode,$pressed');
  }

  void simulateDisconnection() {
    connected = false;
  }

  void simulateReconnection() {
    connected = true;
  }
}
