// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../core/signaling/models/signaling_message.dart';
import '../../core/signaling/signaling_provider.dart';
import 'connection_confirm_dialog.dart';

/// App-wide host that listens for incoming connection invitations and shows
/// the accept/reject dialog regardless of the current route.
///
/// Policy: single active dialog. If a new request arrives while one is
/// showing (or the request lacks a session_id), it is auto-rejected
/// immediately (accepted: false) without opening a second dialog.
class InvitationHost extends ConsumerStatefulWidget {
  const InvitationHost({
    super.key,
    required this.navigatorKey,
    required this.child,
  });

  /// Root navigator key, used to obtain a BuildContext for the dialog that is
  /// valid regardless of the current route.
  final GlobalKey<NavigatorState> navigatorKey;

  final Widget child;

  @override
  ConsumerState<InvitationHost> createState() => _InvitationHostState();
}

class _InvitationHostState extends ConsumerState<InvitationHost> {
  bool _dialogActive = false;

  Future<void> _handleInvite(ConnectRequestMessage req) async {
    final service = ref.read(signalingServiceProvider);
    final sessionId = req.sessionId ?? '';

    // Single-active policy: reject extras (and malformed requests) immediately.
    if (_dialogActive || sessionId.isEmpty) {
      service.respondToConnection(
        sessionId: sessionId,
        fromCode: req.fromCode,
        accepted: false,
      );
      return;
    }

    final context = widget.navigatorKey.currentContext;
    if (context == null) {
      service.respondToConnection(
        sessionId: sessionId,
        fromCode: req.fromCode,
        accepted: false,
      );
      return;
    }

    _dialogActive = true;
    try {
      final result = await ConnectionConfirmDialog.show(
        context,
        requesterName: req.fromCode,
        requesterCode: req.fromCode,
      );
      service.respondToConnection(
        sessionId: sessionId,
        fromCode: req.fromCode,
        accepted: result == ConnectionConfirmResult.accepted,
      );
    } finally {
      _dialogActive = false;
    }
  }

  @override
  Widget build(BuildContext context) {
    ref.listen<AsyncValue<ConnectRequestMessage>>(invitationsProvider,
        (prev, next) {
      next.whenData(_handleInvite);
    });
    return widget.child;
  }
}
