// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:window_manager/window_manager.dart';

import 'app.dart';

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

  runApp(
    const ProviderScope(
      child: RdcsApp(),
    ),
  );
}
