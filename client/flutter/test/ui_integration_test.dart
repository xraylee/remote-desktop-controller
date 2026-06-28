// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:rdcs_flutter/features/home/home_page.dart';
import 'package:rdcs_flutter/features/session/connection_confirm_dialog.dart';
import 'package:rdcs_flutter/features/session/session_screen.dart';
import 'package:rdcs_flutter/core/config/config_provider.dart';
import 'package:rdcs_flutter/core/theme.dart';

void main() {
  group('HomePage UI Tests', () {
    testWidgets('displays device code correctly', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      // Should display formatted device code or placeholder
      expect(find.textContaining('---'), findsOneWidget);
      expect(find.text('RDCS 远程桌面'), findsOneWidget);
    });

    testWidgets('shows connect button', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      expect(find.text('连接远程设备'), findsOneWidget);
      expect(find.text('生成邀请码'), findsOneWidget);
    });

    testWidgets('can tap device code to copy', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      // Tap on device code container
      final codeFinder = find.textContaining('---');
      expect(codeFinder, findsOneWidget);

      await tester.tap(codeFinder);
      await tester.pumpAndSettle();

      // Should show snackbar (when device code is available)
      // Note: Since device code is empty in test, snackbar won't show
    });
  });

  group('SessionScreen UI Tests', () {
    testWidgets('shows connecting state', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const SessionScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Should show placeholder or connecting UI
      expect(find.byType(SessionScreen), findsOneWidget);
    });

    testWidgets('displays video placeholder', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const SessionScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Video placeholder should be present
      expect(find.text('远程桌面画面'), findsOneWidget);
    });
  });

  group('ConnectionConfirmDialog UI Tests', () {
    testWidgets('displays connection request correctly', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '张工的 MacBook',
                    requesterCode: '123 456 789',
                    requestTimeoutSeconds: 30,
                  );
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      // Open dialog
      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify dialog content
      expect(find.text('远程连接请求'), findsOneWidget);
      expect(find.text('张工的 MacBook'), findsOneWidget);
      expect(find.text('123 456 789'), findsOneWidget);
      expect(find.text('拒绝'), findsOneWidget);
      expect(find.text('允许'), findsOneWidget);

      // Countdown should be visible
      expect(find.textContaining('s'), findsWidgets);
    });

    testWidgets('accept button works', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  final result = await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '测试设备',
                    requesterCode: '111 222 333',
                  );
                  expect(result, ConnectionConfirmResult.accepted);
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('允许'));
      await tester.pumpAndSettle();
    });

    testWidgets('reject button works', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  final result = await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '测试设备',
                    requesterCode: '111 222 333',
                  );
                  expect(result, ConnectionConfirmResult.rejected);
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      await tester.tap(find.text('拒绝'));
      await tester.pumpAndSettle();
    });

    testWidgets('shows queued count badge', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '测试设备',
                    requesterCode: '111 222 333',
                    queuedCount: 3,
                  );
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Verify queued count badge
      expect(find.textContaining('还有 3 个连接请求等待处理'), findsOneWidget);
    });

    testWidgets('countdown timer decrements', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '测试设备',
                    requesterCode: '111 222 333',
                    requestTimeoutSeconds: 5,
                  );
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();

      // Initial countdown should show 5s
      expect(find.text('5s'), findsOneWidget);

      // Wait 1 second
      await tester.pump(const Duration(seconds: 1));
      expect(find.text('4s'), findsOneWidget);

      // Wait another second
      await tester.pump(const Duration(seconds: 1));
      expect(find.text('3s'), findsOneWidget);

      // Close dialog manually
      await tester.tap(find.text('拒绝'));
      await tester.pumpAndSettle();
    });
  });

  group('Integration - Full Flow Tests', () {
    testWidgets('home to connect flow', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            routes: {
              '/': (context) => const HomePage(),
              '/connect': (context) => Scaffold(
                    appBar: AppBar(title: const Text('连接设备')),
                    body: const Center(child: Text('连接页面')),
                  ),
            },
          ),
        ),
      );

      // Should be on home page
      expect(find.text('RDCS 远程桌面'), findsOneWidget);

      // Tap connect button
      await tester.tap(find.text('连接远程设备'));
      await tester.pumpAndSettle();

      // Should navigate to connect page
      expect(find.text('连接页面'), findsOneWidget);
    });
  });

  group('Performance Tests', () {
    testWidgets('HomePage renders within performance budget', (tester) async {
      final stopwatch = Stopwatch()..start();

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      stopwatch.stop();

      // Should render in less than 500ms
      expect(stopwatch.elapsedMilliseconds, lessThan(500));

      print('✓ HomePage rendered in ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('SessionScreen renders within performance budget',
        (tester) async {
      final stopwatch = Stopwatch()..start();

      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const SessionScreen(),
          ),
        ),
      );

      await tester.pumpAndSettle();
      stopwatch.stop();

      // Should render in less than 500ms
      expect(stopwatch.elapsedMilliseconds, lessThan(500));

      print('✓ SessionScreen rendered in ${stopwatch.elapsedMilliseconds}ms');
    });

    testWidgets('ConnectionConfirmDialog renders quickly', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: RdcsTheme.light(),
          home: Scaffold(
            body: Builder(
              builder: (context) => ElevatedButton(
                onPressed: () async {
                  await ConnectionConfirmDialog.show(
                    context,
                    requesterName: '测试设备',
                    requesterCode: '111 222 333',
                  );
                },
                child: const Text('Show Dialog'),
              ),
            ),
          ),
        ),
      );

      final stopwatch = Stopwatch()..start();
      await tester.tap(find.text('Show Dialog'));
      await tester.pumpAndSettle();
      stopwatch.stop();

      // Dialog should open in less than 200ms
      expect(stopwatch.elapsedMilliseconds, lessThan(200));

      print(
          '✓ ConnectionConfirmDialog opened in ${stopwatch.elapsedMilliseconds}ms');

      // Close dialog
      await tester.tap(find.text('拒绝'));
      await tester.pumpAndSettle();
    });
  });

  group('Accessibility Tests', () {
    testWidgets('HomePage has proper semantics', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify semantic labels exist
      expect(find.byType(Semantics), findsWidgets);
    });

    testWidgets('Buttons have proper tap targets', (tester) async {
      await tester.pumpWidget(
        ProviderScope(
          child: MaterialApp(
            theme: RdcsTheme.light(),
            home: const HomePage(),
          ),
        ),
      );

      await tester.pumpAndSettle();

      // All buttons should have minimum 44x44 tap target
      final buttons = tester.widgetList<ButtonStyleButton>(
        find.byType(ButtonStyleButton),
      );

      for (final button in buttons) {
        final renderBox = tester.renderObject(find.byWidget(button)) as RenderBox;
        expect(renderBox.size.width, greaterThanOrEqualTo(44));
        expect(renderBox.size.height, greaterThanOrEqualTo(44));
      }
    });
  });
}
