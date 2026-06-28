// Copyright 2026 RDCS Contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.

//! Video frame transmission over WebRTC DataChannel.

use bytes::Bytes;
use std::sync::Arc;
use tracing::debug;
use webrtc::data_channel::RTCDataChannel;

use crate::ConnectionError;

/// Maximum chunk size for DataChannel messages.
/// Most implementations support at least 16KB, we use a conservative value.
const MAX_CHUNK_SIZE: usize = 16_384; // 16KB

/// Video frame transmission over DataChannel.
pub struct VideoChannel {
    data_channel: Arc<RTCDataChannel>,
    max_chunk_size: usize,
}

impl VideoChannel {
    /// Create a new VideoChannel wrapping a DataChannel.
    pub fn new(data_channel: Arc<RTCDataChannel>) -> Self {
        Self {
            data_channel,
            max_chunk_size: MAX_CHUNK_SIZE,
        }
    }

    /// Create with custom max chunk size (for testing).
    pub fn with_chunk_size(data_channel: Arc<RTCDataChannel>, max_chunk_size: usize) -> Self {
        Self {
            data_channel,
            max_chunk_size,
        }
    }

    /// Send a video frame through the DataChannel.
    ///
    /// Large frames are automatically split into chunks.
    pub async fn send_frame(&self, frame: &[u8]) -> Result<(), ConnectionError> {
        if frame.len() <= self.max_chunk_size {
            // Small frame: send directly
            self.data_channel
                .send(&Bytes::from(frame.to_vec()))
                .await
                .map_err(|e| ConnectionError::IceError(format!("Failed to send frame: {}", e)))?;

            debug!("Sent frame: {} bytes", frame.len());
        } else {
            // Large frame: split into chunks
            let num_chunks = (frame.len() + self.max_chunk_size - 1) / self.max_chunk_size;

            for (i, chunk) in frame.chunks(self.max_chunk_size).enumerate() {
                self.data_channel
                    .send(&Bytes::from(chunk.to_vec()))
                    .await
                    .map_err(|e| {
                        ConnectionError::IceError(format!("Failed to send chunk {}/{}: {}", i + 1, num_chunks, e))
                    })?;
            }

            debug!("Sent frame: {} bytes in {} chunks", frame.len(), num_chunks);
        }

        Ok(())
    }

    /// Set up a callback for received frames.
    ///
    /// The callback is invoked for each received message (chunk).
    /// For multi-chunk frames, use FrameReassembler to reconstruct complete frames.
    pub fn on_message<F>(&self, callback: F)
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let callback = Arc::new(callback);
        self.data_channel.on_message(Box::new(move |msg| {
            let data = msg.data.to_vec();
            callback(data);
            Box::pin(async {})
        }));
    }

    /// Check current buffered amount (bytes waiting to be sent).
    /// Use this to implement backpressure control.
    pub async fn buffered_amount(&self) -> usize {
        self.data_channel.buffered_amount().await
    }

    /// Get the label of the underlying DataChannel.
    pub fn label(&self) -> String {
        self.data_channel.label().to_string()
    }

    /// Check if the DataChannel is open and ready to send.
    pub fn ready_state(&self) -> webrtc::data_channel::data_channel_state::RTCDataChannelState {
        self.data_channel.ready_state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full DataChannel tests require a PeerConnection setup.
    // These are basic structural tests.

    #[test]
    fn test_max_chunk_size_constant() {
        assert_eq!(MAX_CHUNK_SIZE, 16_384);
    }

    #[test]
    fn test_custom_chunk_size() {
        // We can't create a real DataChannel without PeerConnection,
        // but we can verify the structure compiles
        assert!(MAX_CHUNK_SIZE > 0);
    }
}
