// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';

import 'app.dart';
import 'core/config/config_provider.dart';
import 'core/ffi/engine_providers.dart';
import 'core/signaling/signaling_provider.dart';

void main() async {
  WidgetsFlutterBinding.ensureInitialized();

  // Initialise window_manager before the widget tree is built so we
  // can configure the window (size, title, minimum constraints)
  // before it becomes visible.
  await windowManager.ensureInitialized();

  const windowOptions = WindowOptions(
    title: 'RDCS 远程桌面',
    minimumSize: Size(900, 600),
    center: true,
    skipTaskbar: false,
    titleBarStyle: TitleBarStyle.normal,
  );

  await windowManager.waitUntilReadyToShow(windowOptions, () async {
    await windowManager.show();
    await windowManager.focus();
  });

  // Create provider container and initialize engine isolate
  final container = ProviderContainer();

  // Initialize the FFI engine isolate in the background
  final engine = container.read(engineIsolateProvider);
  await engine.init();

  // Initialize configuration (generates device code if not exists)
  final config = container.read(configProvider.notifier);
  await config.init();
  print('✅ Configuration initialized. Device code: ${container.read(configProvider).deviceCode}');

  // Create the engine instance
  final createResult = await engine.create('{}');
  if (createResult != 0) {
    print('❌ Failed to create engine: $createResult');
  } else {
    print('✅ Engine created successfully');
  }

  // Initialize signaling service (auto-connect)
  container.read(signalingAutoConnectProvider);
  print('✅ Signaling service initialized');

  runApp(
    UncontrolledProviderScope(
      container: container,
      child: const RdcsApp(),
    ),
  );
}
