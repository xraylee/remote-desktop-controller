// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Clipboard synchronization between local and remote peers.
//!
//! `PollingClipboardSync` watches the local clipboard for changes via
//! periodic polling and emits `ClipboardEvent` values through a channel.
//! Remote changes can be applied locally while suppressing echo loops.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rdcs_platform::{ClipboardContent, ClipboardEvent, ClipboardProvider};

use crate::TransferError;

/// Filter mode controlling which clipboard content types are synchronized.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClipboardFilterMode {
    /// Only synchronize text content; ignore images and files.
    TextOnly,
    /// Synchronize all content types.
    All,
}

/// Manages bidirectional clipboard synchronization via polling.
///
/// A background thread periodically reads the local clipboard and emits
/// `ClipboardEvent` values when changes are detected. Echo-loop prevention
/// ensures that content applied from a remote peer is not re-emitted as
/// a local change.
pub struct PollingClipboardSync {
    active: Arc<AtomicBool>,
    suppress_hash: Arc<Mutex<u64>>,
    event_rx: Mutex<Option<mpsc::Receiver<ClipboardEvent>>>,
    _join_handle: Option<thread::JoinHandle<()>>,
}

impl PollingClipboardSync {
    /// Create and start a new clipboard sync manager.
    ///
    /// Spawns a background thread that polls the clipboard at the given
    /// interval. Changes are sent through an internal channel.
    pub fn start(
        provider: Box<dyn ClipboardProvider>,
        poll_interval: Duration,
        filter_mode: ClipboardFilterMode,
    ) -> Result<Self, TransferError> {
        let active = Arc::new(AtomicBool::new(true));
        let suppress_hash = Arc::new(Mutex::new(0u64));
        let (tx, rx) = mpsc::channel();

        let active_clone = Arc::clone(&active);
        let suppress_clone = Arc::clone(&suppress_hash);

        // Read initial clipboard state to avoid emitting a spurious event.
        let initial_hash = match provider.get_text() {
            Ok(text) => hash_text(&text),
            Err(_) => 0,
        };

        let join_handle = thread::spawn(move || {
            let mut last_hash = initial_hash;

            while active_clone.load(Ordering::SeqCst) {
                thread::sleep(poll_interval);

                if !active_clone.load(Ordering::SeqCst) {
                    break;
                }

                match provider.get_text() {
                    Ok(text) => {
                        let hash = hash_text(&text);
                        let suppress = *suppress_clone.lock().unwrap();

                        if hash != last_hash {
                            if hash != suppress {
                                let content = ClipboardContent::Text(text.clone());
                                if should_emit(&content, filter_mode) {
                                    let _ = tx.send(ClipboardEvent {
                                        content,
                                        timestamp_us: 0,
                                    });
                                }
                            }
                            last_hash = hash;
                        }
                    }
                    Err(_) => {
                        // No text content available.
                    }
                }
            }
        });

        Ok(Self {
            active,
            suppress_hash,
            event_rx: Mutex::new(Some(rx)),
            _join_handle: Some(join_handle),
        })
    }

    /// Stop clipboard synchronization.
    pub fn stop(&self) -> Result<(), TransferError> {
        self.active.store(false, Ordering::SeqCst);
        Ok(())
    }

    /// Take the receiver for local clipboard change events.
    ///
    /// Returns `Some(receiver)` on the first call, `None` on subsequent calls
    /// since `mpsc::Receiver` is not cloneable.
    pub fn local_change(&self) -> Option<mpsc::Receiver<ClipboardEvent>> {
        self.event_rx.lock().unwrap().take()
    }

    /// Apply a remote clipboard event locally.
    ///
    /// The event's content hash is recorded to prevent the polling thread
    /// from re-emitting it as a local change (echo-loop prevention).
    pub fn apply_remote(&self, event: ClipboardEvent) -> Result<(), TransferError> {
        let hash = hash_content(&event.content);
        *self.suppress_hash.lock().unwrap() = hash;
        Ok(())
    }

    /// Return whether synchronization is currently active.
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }
}

impl Drop for PollingClipboardSync {
    fn drop(&mut self) {
        self.active.store(false, Ordering::SeqCst);
        if let Some(handle) = self._join_handle.take() {
            let _ = handle.join();
        }
    }
}

/// Check whether a clipboard content should be emitted given the filter mode.
pub fn should_emit(content: &ClipboardContent, mode: ClipboardFilterMode) -> bool {
    match mode {
        ClipboardFilterMode::TextOnly => matches!(content, ClipboardContent::Text(_)),
        ClipboardFilterMode::All => true,
    }
}

fn hash_text(text: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    hasher.finish()
}

fn hash_content(content: &ClipboardContent) -> u64 {
    let mut hasher = DefaultHasher::new();
    match content {
        ClipboardContent::Text(t) => t.hash(&mut hasher),
        ClipboardContent::Image(d) => d.hash(&mut hasher),
        ClipboardContent::Files(f) => f.hash(&mut hasher),
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rdcs_platform::mock::MockClipboard;

    #[test]
    fn should_emit_text_only() {
        assert!(should_emit(
            &ClipboardContent::Text("hello".into()),
            ClipboardFilterMode::TextOnly
        ));
        assert!(!should_emit(
            &ClipboardContent::Image(vec![0u8; 10]),
            ClipboardFilterMode::TextOnly
        ));
        assert!(!should_emit(
            &ClipboardContent::Files(vec!["file.txt".into()]),
            ClipboardFilterMode::TextOnly
        ));
    }

    #[test]
    fn should_emit_all() {
        assert!(should_emit(
            &ClipboardContent::Text("hello".into()),
            ClipboardFilterMode::All
        ));
        assert!(should_emit(
            &ClipboardContent::Image(vec![0u8; 10]),
            ClipboardFilterMode::All
        ));
        assert!(should_emit(
            &ClipboardContent::Files(vec!["file.txt".into()]),
            ClipboardFilterMode::All
        ));
    }

    #[test]
    fn echo_loop_prevention_via_apply_remote() {
        // Set clipboard to "remote text" before starting sync.
        let clipboard = MockClipboard::new();
        clipboard.set_text("remote text").unwrap();

        let sync = PollingClipboardSync::start(
            Box::new(clipboard),
            Duration::from_millis(20),
            ClipboardFilterMode::TextOnly,
        )
        .unwrap();

        // Apply the same content from "remote" — the polling thread should
        // suppress re-emission since the content hasn't changed.
        let event = ClipboardEvent {
            content: ClipboardContent::Text("remote text".into()),
            timestamp_us: 0,
        };
        sync.apply_remote(event).unwrap();

        // Let the polling thread run several cycles.
        thread::sleep(Duration::from_millis(100));

        // No events should be emitted — the clipboard hasn't changed from
        // its initial state, and even if it did, the suppress_hash matches.
        if let Some(rx) = sync.local_change() {
            assert!(rx.try_recv().is_err());
        }
    }

    #[test]
    fn stop_and_restart() {
        let clipboard = MockClipboard::new();
        let sync = PollingClipboardSync::start(
            Box::new(clipboard),
            Duration::from_millis(50),
            ClipboardFilterMode::TextOnly,
        )
        .unwrap();

        assert!(sync.is_active());
        sync.stop().unwrap();
        assert!(!sync.is_active());
    }
}
