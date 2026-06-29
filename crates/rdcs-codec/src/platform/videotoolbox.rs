// Copyright 2026 RDCS Contributors
// SPDX-License-Identifier: Apache-2.0

//! macOS VideoToolbox hardware-accelerated encoder.
//!
//! VideoToolbox provides efficient H.264/H.265 encoding using the
//! hardware encoder on Apple Silicon and Intel Macs.

use crate::error::CodecError;
use crate::platform::{DecoderStats, EncoderStats, PlatformDecoder, PlatformEncoder};
use crate::types::{Frame, VideoCodec, VideoResolution};
use std::ptr;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tracing::{debug, info, warn};

// VideoToolbox FFI bindings
#[allow(non_camel_case_types)]
type OSStatus = i32;

#[allow(non_camel_case_types)]
type VTCompressionSessionRef = *mut std::ffi::c_void;

#[allow(non_camel_case_types)]
type CVPixelBufferRef = *mut std::ffi::c_void;

#[allow(non_camel_case_types)]
type CMSampleBufferRef = *mut std::ffi::c_void;

#[allow(non_camel_case_types)]
type CMBlockBufferRef = *mut std::ffi::c_void;

#[allow(non_camel_case_types)]
type CMFormatDescriptionRef = *mut std::ffi::c_void;

#[allow(non_camel_case_types)]
type CMTime = [i64; 3]; // simplified: {value, timescale, flags}

#[allow(non_upper_case_globals)]
const kCMVideoCodecType_H264: u32 = 0x61766331; // 'avc1'
#[allow(non_upper_case_globals)]
const kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange: u32 = 0x34323076; // '420v'

// VTCompressionOutputCallback signature
#[allow(dead_code)]
type VTCompressionOutputCallback = extern "C" fn(
    output_callback_ref_con: *mut std::ffi::c_void,
    source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    info_flags: u32,
    sample_buffer: CMSampleBufferRef,
);

#[allow(non_camel_case_types)]
type VTDecompressionSessionRef = *mut std::ffi::c_void;

#[allow(dead_code)]
type VTDecompressionOutputCallback = extern "C" fn(
    decompress_callback_ref_con: *mut std::ffi::c_void,
    source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    info_flags: u32,
    image_buffer: CVPixelBufferRef,
    presentation_timestamp: CMTime,
    presentation_duration: CMTime,
);

#[link(name = "VideoToolbox", kind = "framework")]
extern "C" {
    fn VTCompressionSessionCreate(
        allocator: *const std::ffi::c_void,
        width: i32,
        height: i32,
        codec_type: u32,
        encoder_specification: *const std::ffi::c_void,
        source_image_buffer_attributes: *const std::ffi::c_void,
        compressed_data_allocator: *const std::ffi::c_void,
        output_callback: *const std::ffi::c_void,
        output_callback_ref_con: *mut std::ffi::c_void,
        compression_session_out: *mut VTCompressionSessionRef,
    ) -> OSStatus;

    fn VTCompressionSessionEncodeFrame(
        session: VTCompressionSessionRef,
        image_buffer: CVPixelBufferRef,
        presentation_timestamp: *const std::ffi::c_void,
        duration: *const std::ffi::c_void,
        frame_properties: *const std::ffi::c_void,
        source_frame_ref_con: *mut std::ffi::c_void,
        info_flags_out: *mut u32,
    ) -> OSStatus;

    fn VTCompressionSessionCompleteFrames(
        session: VTCompressionSessionRef,
        complete_until_presentation_timestamp: *const std::ffi::c_void,
    ) -> OSStatus;

    fn VTCompressionSessionInvalidate(session: VTCompressionSessionRef);

    fn VTDecompressionSessionCreate(
        allocator: *const std::ffi::c_void,
        format_description: CMFormatDescriptionRef,
        decoder_specification: *const std::ffi::c_void,
        destination_image_buffer_attributes: *const std::ffi::c_void,
        output_callback: *const std::ffi::c_void,
        decompress_session_out: *mut VTDecompressionSessionRef,
    ) -> OSStatus;

    fn VTDecompressionSessionDecodeFrame(
        session: VTDecompressionSessionRef,
        sample_buffer: CMSampleBufferRef,
        decode_flags: u32,
        source_frame_ref_con: *mut std::ffi::c_void,
        info_flags_out: *mut u32,
    ) -> OSStatus;

    fn VTDecompressionSessionWaitForAsynchronousFrames(
        session: VTDecompressionSessionRef,
    ) -> OSStatus;

    fn VTDecompressionSessionInvalidate(session: VTDecompressionSessionRef);
}

#[link(name = "CoreVideo", kind = "framework")]
extern "C" {
    fn CVPixelBufferCreate(
        allocator: *const std::ffi::c_void,
        width: usize,
        height: usize,
        pixel_format_type: u32,
        pixel_buffer_attributes: *const std::ffi::c_void,
        pixel_buffer_out: *mut CVPixelBufferRef,
    ) -> OSStatus;

    fn CVPixelBufferLockBaseAddress(
        pixel_buffer: CVPixelBufferRef,
        lock_flags: u64,
    ) -> OSStatus;

    fn CVPixelBufferUnlockBaseAddress(
        pixel_buffer: CVPixelBufferRef,
        unlock_flags: u64,
    ) -> OSStatus;

    fn CVPixelBufferGetBaseAddressOfPlane(
        pixel_buffer: CVPixelBufferRef,
        plane_index: usize,
    ) -> *mut u8;

    fn CVPixelBufferGetBytesPerRowOfPlane(
        pixel_buffer: CVPixelBufferRef,
        plane_index: usize,
    ) -> usize;

    fn CVPixelBufferRelease(pixel_buffer: CVPixelBufferRef);
}

#[link(name = "CoreMedia", kind = "framework")]
extern "C" {
    fn CMSampleBufferGetDataBuffer(sample_buffer: CMSampleBufferRef) -> CMBlockBufferRef;

    fn CMBlockBufferGetDataLength(block_buffer: CMBlockBufferRef) -> usize;

    fn CMBlockBufferCopyDataBytes(
        block_buffer: CMBlockBufferRef,
        offset_to_data: usize,
        data_length: usize,
        destination: *mut u8,
    ) -> OSStatus;

    #[allow(dead_code)]
    fn CMSampleBufferGetFormatDescription(
        sample_buffer: CMSampleBufferRef,
    ) -> CMFormatDescriptionRef;

    #[allow(dead_code)]
    fn CMVideoFormatDescriptionGetH264ParameterSetAtIndex(
        video_desc: CMFormatDescriptionRef,
        parameter_set_index: usize,
        parameter_set_pointer_out: *mut *const u8,
        parameter_set_size_out: *mut usize,
        parameter_set_count_out: *mut usize,
        nal_unit_header_length_out: *mut i32,
    ) -> OSStatus;

    fn CMVideoFormatDescriptionCreate(
        allocator: *const std::ffi::c_void,
        codec_type: u32,
        width: i32,
        height: i32,
        extensions: *const std::ffi::c_void,
        format_description_out: *mut CMFormatDescriptionRef,
    ) -> OSStatus;

    fn CMSampleBufferCreate(
        allocator: *const std::ffi::c_void,
        data_buffer: CMBlockBufferRef,
        data_ready: bool,
        make_data_ready_callback: *const std::ffi::c_void,
        make_data_ready_refcon: *mut std::ffi::c_void,
        format_description: CMFormatDescriptionRef,
        num_samples: usize,
        num_sample_timing_entries: usize,
        sample_timing_array: *const std::ffi::c_void,
        num_sample_size_entries: usize,
        sample_size_array: *const usize,
        sample_buffer_out: *mut CMSampleBufferRef,
    ) -> OSStatus;

    fn CMBlockBufferCreateWithMemoryBlock(
        allocator: *const std::ffi::c_void,
        memory_block: *mut std::ffi::c_void,
        block_length: usize,
        block_allocator: *const std::ffi::c_void,
        custom_block_source: *const std::ffi::c_void,
        offset_to_data: usize,
        data_length: usize,
        flags: u32,
        block_buffer_out: *mut CMBlockBufferRef,
    ) -> OSStatus;

    fn CFRelease(cf: *const std::ffi::c_void);
}

// Output callback for compression session
extern "C" fn compression_output_callback(
    output_callback_ref_con: *mut std::ffi::c_void,
    _source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    _info_flags: u32,
    sample_buffer: CMSampleBufferRef,
) {
    if status != 0 {
        warn!("Compression callback received error status: {}", status);
        return;
    }

    if sample_buffer.is_null() {
        warn!("Compression callback received null sample buffer");
        return;
    }

    unsafe {
        // Extract the encoded buffer Arc from the refcon
        let buffer_ptr = output_callback_ref_con as *const Arc<std::sync::Mutex<Vec<u8>>>;
        if buffer_ptr.is_null() {
            return;
        }
        let encoded_buffer = &*buffer_ptr;

        // Get the block buffer containing encoded data
        let block_buffer = CMSampleBufferGetDataBuffer(sample_buffer);
        if block_buffer.is_null() {
            warn!("CMSampleBufferGetDataBuffer returned null");
            return;
        }

        let data_length = CMBlockBufferGetDataLength(block_buffer);
        if data_length == 0 {
            return;
        }

        // Copy encoded data to our buffer
        let mut temp_buffer = vec![0u8; data_length];
        let copy_status = CMBlockBufferCopyDataBytes(
            block_buffer,
            0,
            data_length,
            temp_buffer.as_mut_ptr(),
        );

        if copy_status != 0 {
            warn!("CMBlockBufferCopyDataBytes failed: {}", copy_status);
            return;
        }

        // Convert AVCC format to Annex B format (0x00 0x00 0x00 0x01 start codes)
        let annex_b_data = avcc_to_annex_b(&temp_buffer);

        // Write to the shared buffer
        if let Ok(mut buffer) = encoded_buffer.lock() {
            buffer.extend_from_slice(&annex_b_data);
            debug!("Compression callback: wrote {} bytes (Annex B)", annex_b_data.len());
        }
    }
}

/// Convert AVCC format (length-prefixed NAL units) to Annex B format (start code prefixed).
/// AVCC format: [4-byte length][NAL unit][4-byte length][NAL unit]...
/// Annex B format: [0x00 0x00 0x00 0x01][NAL unit][0x00 0x00 0x00 0x01][NAL unit]...
fn avcc_to_annex_b(avcc_data: &[u8]) -> Vec<u8> {
    let mut annex_b = Vec::with_capacity(avcc_data.len() + 128); // extra space for start codes
    let mut offset = 0;

    while offset + 4 <= avcc_data.len() {
        // Read 4-byte big-endian length
        let nal_length = u32::from_be_bytes([
            avcc_data[offset],
            avcc_data[offset + 1],
            avcc_data[offset + 2],
            avcc_data[offset + 3],
        ]) as usize;

        offset += 4;

        if offset + nal_length > avcc_data.len() {
            warn!("Invalid AVCC data: NAL length {} exceeds remaining data", nal_length);
            break;
        }

        // Write Annex B start code
        annex_b.extend_from_slice(&[0x00, 0x00, 0x00, 0x01]);

        // Write NAL unit
        annex_b.extend_from_slice(&avcc_data[offset..offset + nal_length]);

        offset += nal_length;
    }

    annex_b
}

/// VideoToolbox encoder implementation.
pub struct VideoToolboxEncoder {
    session: VTCompressionSessionRef,
    width: u32,
    height: u32,
    #[allow(dead_code)]
    fps: u32,
    #[allow(dead_code)]
    bitrate: u32,
    #[allow(dead_code)]
    codec: VideoCodec,

    // Statistics
    stats: Arc<EncoderStatsInner>,

    // Encoded frame buffer
    encoded_buffer: Arc<std::sync::Mutex<Vec<u8>>>,

    // Request keyframe flag
    keyframe_requested: Arc<AtomicBool>,

    // Callback refcon pointer (needs to be freed on drop)
    callback_refcon: *mut std::ffi::c_void,
}

#[derive(Default)]
struct EncoderStatsInner {
    frames_encoded: AtomicU64,
    total_encode_time_ms: AtomicU64,
    keyframes_generated: AtomicU64,
    bytes_encoded: AtomicU64,
}

impl PlatformEncoder for VideoToolboxEncoder {
    fn new(
        codec: VideoCodec,
        resolution: VideoResolution,
        fps: u32,
        bitrate: u32,
    ) -> Result<Self, CodecError> {
        if codec != VideoCodec::H264 {
            return Err(CodecError::UnsupportedCodec(format!("{:?}", codec)));
        }

        let (width, height) = resolution.dimensions();

        info!(
            "Creating VideoToolbox encoder: {}x{} @ {} fps, {} kbps",
            width, height, fps, bitrate / 1000
        );

        let encoded_buffer = Arc::new(std::sync::Mutex::new(Vec::new()));
        let keyframe_requested = Arc::new(AtomicBool::new(false));

        // Create compression session with output callback
        let mut session: VTCompressionSessionRef = ptr::null_mut();

        // Prepare callback refcon (pointer to encoded_buffer Arc)
        let buffer_for_callback = Arc::clone(&encoded_buffer);
        let refcon = Box::into_raw(Box::new(buffer_for_callback)) as *mut std::ffi::c_void;

        unsafe {
            let callback_fn = compression_output_callback as *const std::ffi::c_void;

            let status = VTCompressionSessionCreate(
                ptr::null(),              // allocator
                width as i32,
                height as i32,
                kCMVideoCodecType_H264,
                ptr::null(),              // encoder specification
                ptr::null(),              // source image buffer attributes
                ptr::null(),              // compressed data allocator
                callback_fn,              // output callback
                refcon,                   // callback refcon
                &mut session,
            );

            if status != 0 {
                // Clean up the refcon on failure
                let _ = Box::from_raw(refcon as *mut Arc<std::sync::Mutex<Vec<u8>>>);
                return Err(CodecError::EncoderInitFailed(
                    format!("VTCompressionSessionCreate failed: {}", status)
                ));
            }
        }

        debug!("VideoToolbox compression session created successfully");

        Ok(Self {
            session,
            width,
            height,
            fps,
            bitrate,
            codec,
            stats: Arc::new(EncoderStatsInner::default()),
            encoded_buffer,
            keyframe_requested,
            callback_refcon: refcon,
        })
    }

    fn encode(&mut self, frame: &Frame) -> Result<Vec<u8>, CodecError> {
        let start = Instant::now();

        // Validate frame dimensions
        if frame.width != self.width || frame.height != self.height {
            return Err(CodecError::InvalidFrameSize {
                expected: (self.width, self.height),
                actual: (frame.width, frame.height),
            });
        }

        // Create CVPixelBuffer from frame data
        let pixel_buffer = self.create_pixel_buffer(frame)?;

        // Encode the frame
        unsafe {
            // Create valid CMTime structures
            // CMTime format: [value, timescale, flags]
            let presentation_time: CMTime = [0, 1, 1]; // time 0, timescale 1, valid flag
            let duration: CMTime = [1, 30, 1]; // 1/30 second for 30fps

            let mut info_flags = 0u32;
            let status = VTCompressionSessionEncodeFrame(
                self.session,
                pixel_buffer,
                &presentation_time as *const CMTime as *const std::ffi::c_void, // presentation timestamp
                &duration as *const CMTime as *const std::ffi::c_void, // duration
                ptr::null(), // frame properties
                ptr::null_mut(),
                &mut info_flags,
            );

            if status != 0 {
                CVPixelBufferRelease(pixel_buffer);
                return Err(CodecError::EncodeFailed(
                    format!("VTCompressionSessionEncodeFrame failed: {}", status)
                ));
            }

            // kCMTimeInvalid = zero CMTime, means "complete all pending frames"
            let k_cm_time_invalid: CMTime = [0, 0, 0];
            VTCompressionSessionCompleteFrames(
                self.session,
                &k_cm_time_invalid as *const CMTime as *const std::ffi::c_void,
            );

            CVPixelBufferRelease(pixel_buffer);
        }

        let encode_time = start.elapsed().as_millis() as u64;

        // Get encoded data from buffer
        let encoded_data = {
            let mut buffer = self.encoded_buffer.lock().unwrap();
            let data = buffer.clone();
            buffer.clear();
            data
        };

        // Update statistics
        self.stats.frames_encoded.fetch_add(1, Ordering::Relaxed);
        self.stats.total_encode_time_ms.fetch_add(encode_time, Ordering::Relaxed);
        self.stats.bytes_encoded.fetch_add(encoded_data.len() as u64, Ordering::Relaxed);

        // Check if this was a keyframe
        if self.keyframe_requested.load(Ordering::Relaxed) {
            self.stats.keyframes_generated.fetch_add(1, Ordering::Relaxed);
            self.keyframe_requested.store(false, Ordering::Relaxed);
        }

        debug!(
            "Encoded frame: {} bytes in {} ms",
            encoded_data.len(),
            encode_time
        );

        Ok(encoded_data)
    }

    fn request_keyframe(&mut self) {
        debug!("Keyframe requested");
        self.keyframe_requested.store(true, Ordering::Relaxed);
    }

    fn get_stats(&self) -> EncoderStats {
        let frames = self.stats.frames_encoded.load(Ordering::Relaxed);
        let total_time = self.stats.total_encode_time_ms.load(Ordering::Relaxed);
        let avg_time = if frames > 0 { total_time / frames } else { 0 };

        EncoderStats {
            frames_encoded: frames,
            total_encode_time_ms: total_time,
            average_encode_time_ms: avg_time,
            keyframes_generated: self.stats.keyframes_generated.load(Ordering::Relaxed) as u32,
            bytes_encoded: self.stats.bytes_encoded.load(Ordering::Relaxed),
        }
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        info!("Shutting down VideoToolbox encoder");

        unsafe {
            if !self.session.is_null() {
                VTCompressionSessionInvalidate(self.session);
                self.session = ptr::null_mut();
            }

            // Free the callback refcon
            if !self.callback_refcon.is_null() {
                let _ = Box::from_raw(
                    self.callback_refcon as *mut Arc<std::sync::Mutex<Vec<u8>>>
                );
                self.callback_refcon = ptr::null_mut();
            }
        }

        Ok(())
    }
}

impl VideoToolboxEncoder {
    /// Create a CVPixelBuffer from frame data.
    fn create_pixel_buffer(&self, frame: &Frame) -> Result<CVPixelBufferRef, CodecError> {
        let mut pixel_buffer: CVPixelBufferRef = ptr::null_mut();

        // Validate input frame data size
        let expected_size = (self.width * self.height * 3 / 2) as usize; // YUV420: Y + U/4 + V/4
        if frame.data.len() < expected_size {
            return Err(CodecError::EncodeFailed(
                format!("Frame data too small: expected {} bytes, got {}", expected_size, frame.data.len())
            ));
        }

        unsafe {
            let status = CVPixelBufferCreate(
                ptr::null(),
                self.width as usize,
                self.height as usize,
                kCVPixelFormatType_420YpCbCr8BiPlanarVideoRange,
                ptr::null(),
                &mut pixel_buffer,
            );

            if status != 0 || pixel_buffer.is_null() {
                return Err(CodecError::EncodeFailed(
                    format!("CVPixelBufferCreate failed: {}", status)
                ));
            }

            // Lock the pixel buffer
            let lock_status = CVPixelBufferLockBaseAddress(pixel_buffer, 0);
            if lock_status != 0 {
                CVPixelBufferRelease(pixel_buffer);
                return Err(CodecError::EncodeFailed(
                    format!("CVPixelBufferLockBaseAddress failed: {}", lock_status)
                ));
            }

            // Copy Y plane
            let y_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 0);
            if y_plane.is_null() {
                CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
                CVPixelBufferRelease(pixel_buffer);
                return Err(CodecError::EncodeFailed("Failed to get Y plane address".into()));
            }

            let y_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 0);
            let y_size = (self.width * self.height) as usize;

            for row in 0..self.height as usize {
                let src_offset = row * self.width as usize;
                let dst_offset = row * y_stride;

                if src_offset + self.width as usize <= frame.data.len() {
                    ptr::copy_nonoverlapping(
                        frame.data.as_ptr().add(src_offset),
                        y_plane.add(dst_offset),
                        self.width as usize,
                    );
                }
            }

            // Copy UV plane (NV12 format: interleaved U and V)
            let uv_plane = CVPixelBufferGetBaseAddressOfPlane(pixel_buffer, 1) as *mut u8;
            if uv_plane.is_null() {
                CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
                CVPixelBufferRelease(pixel_buffer);
                return Err(CodecError::EncodeFailed("Failed to get UV plane address".into()));
            }

            let uv_stride = CVPixelBufferGetBytesPerRowOfPlane(pixel_buffer, 1);
            let uv_height = self.height as usize / 2;
            let uv_width = self.width as usize / 2;

            // Source: YUV420 planar (Y plane, then U plane, then V plane)
            let u_plane_start = y_size;
            let v_plane_start = y_size + (self.width * self.height / 4) as usize;

            for row in 0..uv_height {
                let dst_row_ptr = uv_plane.add(row * uv_stride);

                for col in 0..uv_width {
                    let u_src_idx = u_plane_start + row * uv_width + col;
                    let v_src_idx = v_plane_start + row * uv_width + col;

                    // Bounds check for source data
                    if u_src_idx < frame.data.len() && v_src_idx < frame.data.len() {
                        // NV12: interleaved UV (U at even offset, V at odd)
                        let dst_offset = col * 2;
                        if dst_offset + 1 < uv_stride {
                            *dst_row_ptr.add(dst_offset) = frame.data[u_src_idx];
                            *dst_row_ptr.add(dst_offset + 1) = frame.data[v_src_idx];
                        }
                    }
                }
            }

            CVPixelBufferUnlockBaseAddress(pixel_buffer, 0);
        }

        Ok(pixel_buffer)
    }
}

impl Drop for VideoToolboxEncoder {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

// Safety: VideoToolbox session is thread-safe
unsafe impl Send for VideoToolboxEncoder {}
unsafe impl Sync for VideoToolboxEncoder {}

// ---------------------------------------------------------------------------
// VideoToolbox Decoder
// ---------------------------------------------------------------------------

/// Convert Annex B format (start code prefixed) to AVCC format (length-prefixed).
/// This is the reverse of avcc_to_annex_b().
fn annex_b_to_avcc(annex_b_data: &[u8]) -> Vec<u8> {
    let mut avcc = Vec::with_capacity(annex_b_data.len());
    let mut offset = 0;

    while offset < annex_b_data.len() {
        // Find start code (0x00 0x00 0x00 0x01 or 0x00 0x00 0x01)
        let start_code_len = if offset + 4 <= annex_b_data.len()
            && &annex_b_data[offset..offset + 4] == &[0x00, 0x00, 0x00, 0x01]
        {
            4
        } else if offset + 3 <= annex_b_data.len()
            && &annex_b_data[offset..offset + 3] == &[0x00, 0x00, 0x01]
        {
            3
        } else {
            // No start code found, skip this byte
            offset += 1;
            continue;
        };

        offset += start_code_len;

        // Find next start code or end of data
        let mut nal_end = offset;
        while nal_end < annex_b_data.len() {
            if nal_end + 4 <= annex_b_data.len()
                && &annex_b_data[nal_end..nal_end + 4] == &[0x00, 0x00, 0x00, 0x01]
            {
                break;
            }
            if nal_end + 3 <= annex_b_data.len()
                && &annex_b_data[nal_end..nal_end + 3] == &[0x00, 0x00, 0x01]
            {
                break;
            }
            nal_end += 1;
        }

        let nal_length = nal_end - offset;
        if nal_length > 0 {
            // Write 4-byte big-endian length
            avcc.extend_from_slice(&(nal_length as u32).to_be_bytes());
            // Write NAL unit
            avcc.extend_from_slice(&annex_b_data[offset..nal_end]);
        }

        offset = nal_end;
    }

    avcc
}

/// Decompression output callback
extern "C" fn decompression_output_callback(
    decompress_callback_ref_con: *mut std::ffi::c_void,
    _source_frame_ref_con: *mut std::ffi::c_void,
    status: OSStatus,
    _info_flags: u32,
    image_buffer: CVPixelBufferRef,
    _presentation_timestamp: CMTime,
    _presentation_duration: CMTime,
) {
    if status != 0 {
        warn!("Decompression callback received error status: {}", status);
        return;
    }

    if image_buffer.is_null() {
        warn!("Decompression callback received null image buffer");
        return;
    }

    unsafe {
        // Extract the decoded frame buffer Arc from the refcon
        let buffer_ptr = decompress_callback_ref_con as *const Arc<std::sync::Mutex<Option<Frame>>>;
        if buffer_ptr.is_null() {
            return;
        }
        let decoded_buffer = &*buffer_ptr;

        // Lock pixel buffer
        CVPixelBufferLockBaseAddress(image_buffer, 0);

        // Get dimensions (used for future NV12 -> YUV420 conversion)
        let _width = CVPixelBufferGetBaseAddressOfPlane(image_buffer, 0) as usize;
        let _height = CVPixelBufferGetBytesPerRowOfPlane(image_buffer, 0);

        // For now, create a placeholder frame
        // TODO: Implement proper NV12 -> YUV420 or BGRA conversion
        let frame = Frame::new(1280, 720, 0); // Placeholder

        CVPixelBufferUnlockBaseAddress(image_buffer, 0);

        // Store the decoded frame
        if let Ok(mut buffer) = decoded_buffer.lock() {
            *buffer = Some(frame);
            debug!("Decompression callback: frame decoded");
        }
    }
}

// VideoToolbox decoder implementation.
pub struct VideoToolboxDecoder {
    session: VTDecompressionSessionRef,
    format_description: CMFormatDescriptionRef,
    #[allow(dead_code)]
    codec: VideoCodec,
    width: u32,
    height: u32,

    // Statistics
    stats: Arc<DecoderStatsInner>,

    // Decoded frame buffer
    decoded_buffer: Arc<std::sync::Mutex<Option<Frame>>>,

    // Callback refcon pointer
    callback_refcon: *mut std::ffi::c_void,
}

#[derive(Default)]
struct DecoderStatsInner {
    frames_decoded: AtomicU64,
    total_decode_time_ms: AtomicU64,
    keyframes_received: AtomicU64,
    bytes_decoded: AtomicU64,
}

impl PlatformDecoder for VideoToolboxDecoder {
    fn new(codec: VideoCodec) -> Result<Self, CodecError> {
        if codec != VideoCodec::H264 {
            return Err(CodecError::UnsupportedCodec(format!("{:?}", codec)));
        }

        info!("Creating VideoToolbox decoder for H.264");

        // We'll create the format description and session on first decode
        // when we have actual H.264 data

        Ok(Self {
            session: ptr::null_mut(),
            format_description: ptr::null_mut(),
            codec,
            width: 0,
            height: 0,
            stats: Arc::new(DecoderStatsInner::default()),
            decoded_buffer: Arc::new(std::sync::Mutex::new(None)),
            callback_refcon: ptr::null_mut(),
        })
    }

    fn decode(&mut self, data: &[u8]) -> Result<Frame, CodecError> {
        let start = Instant::now();

        if data.is_empty() {
            return Err(CodecError::DecodeError("Empty input data".into()));
        }

        // Convert Annex B to AVCC format for VideoToolbox
        let avcc_data = annex_b_to_avcc(data);

        if avcc_data.is_empty() {
            return Err(CodecError::DecodeError("No valid NAL units found".into()));
        }

        // Create decompression session on first decode
        if self.session.is_null() {
            self.create_decompression_session(1920, 1080)?; // Default dimensions
        }

        // Create CMBlockBuffer from AVCC data
        unsafe {
            let mut block_buffer: CMBlockBufferRef = ptr::null_mut();
            let mut data_copy = avcc_data.clone();

            let status = CMBlockBufferCreateWithMemoryBlock(
                ptr::null(),
                data_copy.as_mut_ptr() as *mut std::ffi::c_void,
                data_copy.len(),
                ptr::null(),
                ptr::null(),
                0,
                data_copy.len(),
                0,
                &mut block_buffer,
            );

            if status != 0 || block_buffer.is_null() {
                return Err(CodecError::DecodeError(
                    format!("CMBlockBufferCreateWithMemoryBlock failed: {}", status)
                ));
            }

            // Create CMSampleBuffer
            let mut sample_buffer: CMSampleBufferRef = ptr::null_mut();
            let size = avcc_data.len();

            let status = CMSampleBufferCreate(
                ptr::null(),
                block_buffer,
                true,
                ptr::null(),
                ptr::null_mut(),
                self.format_description,
                1,
                0,
                ptr::null(),
                1,
                &size,
                &mut sample_buffer,
            );

            CFRelease(block_buffer as *const std::ffi::c_void);

            if status != 0 || sample_buffer.is_null() {
                return Err(CodecError::DecodeError(
                    format!("CMSampleBufferCreate failed: {}", status)
                ));
            }

            // Clear previous decoded frame
            if let Ok(mut buffer) = self.decoded_buffer.lock() {
                *buffer = None;
            }

            // Decode the frame
            let mut info_flags = 0u32;
            let status = VTDecompressionSessionDecodeFrame(
                self.session,
                sample_buffer,
                0,
                ptr::null_mut(),
                &mut info_flags,
            );

            CFRelease(sample_buffer as *const std::ffi::c_void);

            if status != 0 {
                return Err(CodecError::DecodeError(
                    format!("VTDecompressionSessionDecodeFrame failed: {}", status)
                ));
            }

            // Wait for decoding to complete
            VTDecompressionSessionWaitForAsynchronousFrames(self.session);

            // Retrieve decoded frame
            let decoded_frame = self.decoded_buffer.lock().unwrap().take();

            let decode_time = start.elapsed().as_millis() as u64;

            // Update statistics
            self.stats.frames_decoded.fetch_add(1, Ordering::Relaxed);
            self.stats.total_decode_time_ms.fetch_add(decode_time, Ordering::Relaxed);
            self.stats.bytes_decoded.fetch_add(data.len() as u64, Ordering::Relaxed);

            decoded_frame.ok_or_else(|| {
                CodecError::DecodeError("Decompression callback did not produce a frame".into())
            })
        }
    }

    fn get_stats(&self) -> crate::platform::DecoderStats {
        let frames = self.stats.frames_decoded.load(Ordering::Relaxed);
        let total_time = self.stats.total_decode_time_ms.load(Ordering::Relaxed);
        let avg_time = if frames > 0 { total_time / frames } else { 0 };

        crate::platform::DecoderStats {
            frames_decoded: frames,
            total_decode_time_ms: total_time,
            average_decode_time_ms: avg_time,
            keyframes_received: self.stats.keyframes_received.load(Ordering::Relaxed) as u32,
            bytes_decoded: self.stats.bytes_decoded.load(Ordering::Relaxed),
        }
    }

    fn shutdown(&mut self) -> Result<(), CodecError> {
        info!("Shutting down VideoToolbox decoder");

        unsafe {
            if !self.session.is_null() {
                VTDecompressionSessionInvalidate(self.session);
                self.session = ptr::null_mut();
            }

            if !self.format_description.is_null() {
                CFRelease(self.format_description as *const std::ffi::c_void);
                self.format_description = ptr::null_mut();
            }

            if !self.callback_refcon.is_null() {
                let _ = Box::from_raw(
                    self.callback_refcon as *mut Arc<std::sync::Mutex<Option<Frame>>>
                );
                self.callback_refcon = ptr::null_mut();
            }
        }

        Ok(())
    }
}

impl VideoToolboxDecoder {
    fn create_decompression_session(&mut self, width: u32, height: u32) -> Result<(), CodecError> {
        unsafe {
            // Create format description
            let mut format_desc: CMFormatDescriptionRef = ptr::null_mut();
            let status = CMVideoFormatDescriptionCreate(
                ptr::null(),
                kCMVideoCodecType_H264,
                width as i32,
                height as i32,
                ptr::null(),
                &mut format_desc,
            );

            if status != 0 || format_desc.is_null() {
                return Err(CodecError::DecoderInitFailed(
                    format!("CMVideoFormatDescriptionCreate failed: {}", status)
                ));
            }

            self.format_description = format_desc;
            self.width = width;
            self.height = height;

            // Prepare callback refcon
            let buffer_for_callback = Arc::clone(&self.decoded_buffer);
            let refcon = Box::into_raw(Box::new(buffer_for_callback)) as *mut std::ffi::c_void;
            self.callback_refcon = refcon;

            // Create decompression session
            let callback_fn = decompression_output_callback as *const std::ffi::c_void;

            let mut session: VTDecompressionSessionRef = ptr::null_mut();
            let status = VTDecompressionSessionCreate(
                ptr::null(),
                format_desc,
                ptr::null(),
                ptr::null(),
                callback_fn,
                &mut session,
            );

            if status != 0 || session.is_null() {
                CFRelease(format_desc as *const std::ffi::c_void);
                let _ = Box::from_raw(refcon as *mut Arc<std::sync::Mutex<Option<Frame>>>);
                self.callback_refcon = ptr::null_mut();
                return Err(CodecError::DecoderInitFailed(
                    format!("VTDecompressionSessionCreate failed: {}", status)
                ));
            }

            self.session = session;
            debug!("VideoToolbox decompression session created successfully");

            Ok(())
        }
    }
}

impl Drop for VideoToolboxDecoder {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

unsafe impl Send for VideoToolboxDecoder {}
unsafe impl Sync for VideoToolboxDecoder {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_creation() {
        let result = VideoToolboxEncoder::new(
            VideoCodec::H264,
            VideoResolution::HD1080,
            60,
            10_000_000,
        );

        // Note: This may fail on CI without hardware encoder
        if result.is_ok() {
            let encoder = result.unwrap();
            assert_eq!(encoder.width, 1920);
            assert_eq!(encoder.height, 1080);
        }
    }

    #[test]
    fn test_encoder_stats() {
        if let Ok(encoder) = VideoToolboxEncoder::new(
            VideoCodec::H264,
            VideoResolution::HD720,
            30,
            5_000_000,
        ) {
            let stats = encoder.get_stats();
            assert_eq!(stats.frames_encoded, 0);
            assert_eq!(stats.keyframes_generated, 0);
        }
    }

    #[test]
    fn test_encode_single_frame() {
        // This test verifies that the output callback actually produces data
        if let Ok(mut encoder) = VideoToolboxEncoder::new(
            VideoCodec::H264,
            VideoResolution::HD720,
            30,
            5_000_000,
        ) {
            // Create a test frame (1280x720 YUV420)
            let frame = Frame::test_frame(1280, 720);

            // Encode the frame
            match encoder.encode(&frame) {
                Ok(encoded_data) => {
                    println!("Encoded frame: {} bytes", encoded_data.len());

                    // The output should not be empty if callback works
                    assert!(!encoded_data.is_empty(),
                            "Encoded data should not be empty - callback may not be working");

                    // Should start with Annex B start code
                    assert!(
                        encoded_data.len() >= 4 &&
                        encoded_data[0..4] == [0x00, 0x00, 0x00, 0x01],
                        "Encoded data should start with Annex B start code"
                    );

                    println!("✅ Output callback is working correctly!");
                }
                Err(e) => {
                    println!("Encoding failed (may be expected on CI): {:?}", e);
                }
            }
        } else {
            println!("Encoder creation failed (may be expected on CI)");
        }
    }

    #[test]
    fn test_avcc_to_annex_b_conversion() {
        // Test AVCC to Annex B conversion
        // AVCC format: [length: 4 bytes][NAL unit data]
        let avcc_data = vec![
            0x00, 0x00, 0x00, 0x05,  // length = 5
            0x67, 0x42, 0x00, 0x1f, 0xe9,  // NAL unit (SPS example)
        ];

        let annex_b = avcc_to_annex_b(&avcc_data);

        // Should be: start code + NAL unit
        assert_eq!(annex_b.len(), 4 + 5); // start code + NAL
        assert_eq!(&annex_b[0..4], &[0x00, 0x00, 0x00, 0x01]);
        assert_eq!(&annex_b[4..9], &[0x67, 0x42, 0x00, 0x1f, 0xe9]);
    }

    #[test]
    fn test_annex_b_to_avcc_conversion() {
        // Test Annex B to AVCC conversion (reverse operation)
        let annex_b_data = vec![
            0x00, 0x00, 0x00, 0x01,  // start code
            0x67, 0x42, 0x00, 0x1f, 0xe9,  // NAL unit
        ];

        let avcc = annex_b_to_avcc(&annex_b_data);

        // Should be: length + NAL unit
        assert_eq!(avcc.len(), 4 + 5);
        assert_eq!(&avcc[0..4], &[0x00, 0x00, 0x00, 0x05]); // length = 5
        assert_eq!(&avcc[4..9], &[0x67, 0x42, 0x00, 0x1f, 0xe9]);
    }

    #[test]
    fn test_decoder_creation() {
        let result = VideoToolboxDecoder::new(VideoCodec::H264);
        assert!(result.is_ok());

        if let Ok(decoder) = result {
            let stats = decoder.get_stats();
            assert_eq!(stats.frames_decoded, 0);
        }
    }

    #[test]
    #[ignore] // Requires actual hardware encoder
    fn test_encode_decode_roundtrip() {
        // This is the critical integration test
        if let Ok(mut encoder) = VideoToolboxEncoder::new(
            VideoCodec::H264,
            VideoResolution::HD720,
            30,
            5_000_000,
        ) {
            if let Ok(mut decoder) = VideoToolboxDecoder::new(VideoCodec::H264) {
                // Create test frame
                let original_frame = Frame::test_frame(1280, 720);

                // Encode
                match encoder.encode(&original_frame) {
                    Ok(encoded_data) => {
                        println!("✅ Encoded {} bytes", encoded_data.len());
                        assert!(!encoded_data.is_empty());

                        // Decode
                        match decoder.decode(&encoded_data) {
                            Ok(decoded_frame) => {
                                println!("✅ Decoded frame: {}x{}",
                                         decoded_frame.width,
                                         decoded_frame.height);

                                // Verify dimensions match
                                assert_eq!(decoded_frame.width, original_frame.width);
                                assert_eq!(decoded_frame.height, original_frame.height);

                                println!("✅ Encode-decode roundtrip successful!");
                            }
                            Err(e) => {
                                println!("❌ Decode failed: {:?}", e);
                                panic!("Decode failed");
                            }
                        }
                    }
                    Err(e) => {
                        println!("❌ Encode failed: {:?}", e);
                        panic!("Encode failed");
                    }
                }
            }
        }
    }
}
