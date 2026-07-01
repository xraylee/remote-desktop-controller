//! Resolution scaling utilities for screen capture optimization.
//!
//! Provides bilinear downscaling to reduce capture resolution without
//! changing the display configuration.

use rdcs_platform::PixelFormat;
use std::sync::Arc;

/// Downscale a captured frame using bilinear interpolation.
///
/// This is a software implementation suitable for modest resolution changes
/// (e.g., 1920x1080 → 1280x720). For larger changes or better quality,
/// consider using vImage or Accelerate framework.
///
/// # Arguments
///
/// * `src_data` - Source pixel data (BGRA format)
/// * `src_width` - Source width
/// * `src_height` - Source height
/// * `src_stride` - Source bytes per row
/// * `dst_width` - Destination width
/// * `dst_height` - Destination height
/// * `pixel_format` - Pixel format (must be Bgra or Rgba)
///
/// # Returns
///
/// Tuple of (scaled_data, dst_stride)
pub fn scale_frame(
    src_data: &[u8],
    src_width: u32,
    src_height: u32,
    src_stride: u32,
    dst_width: u32,
    dst_height: u32,
    pixel_format: PixelFormat,
) -> (Arc<[u8]>, u32) {
    // Only support 32-bit formats
    let bpp = match pixel_format {
        PixelFormat::Bgra | PixelFormat::Rgba => 4,
        PixelFormat::Nv12 => {
            // NV12 would need different scaling logic
            return (Arc::from(src_data.to_vec().into_boxed_slice()), src_stride);
        }
    };

    let dst_stride = dst_width * bpp;
    let mut dst_data = vec![0u8; (dst_stride * dst_height) as usize];

    let x_ratio = src_width as f32 / dst_width as f32;
    let y_ratio = src_height as f32 / dst_height as f32;

    for dst_y in 0..dst_height {
        for dst_x in 0..dst_width {
            let src_x = dst_x as f32 * x_ratio;
            let src_y = dst_y as f32 * y_ratio;

            let x0 = src_x.floor() as u32;
            let y0 = src_y.floor() as u32;
            let x1 = (x0 + 1).min(src_width - 1);
            let y1 = (y0 + 1).min(src_height - 1);

            let fx = src_x - x0 as f32;
            let fy = src_y - y0 as f32;

            // Get pixel offsets
            let offset_00 = (y0 * src_stride + x0 * bpp) as usize;
            let offset_10 = (y0 * src_stride + x1 * bpp) as usize;
            let offset_01 = (y1 * src_stride + x0 * bpp) as usize;
            let offset_11 = (y1 * src_stride + x1 * bpp) as usize;

            let dst_offset = (dst_y * dst_stride + dst_x * bpp) as usize;

            // Bilinear interpolation for each channel (BGRA)
            for ch in 0..bpp as usize {
                if offset_00 + ch < src_data.len()
                    && offset_10 + ch < src_data.len()
                    && offset_01 + ch < src_data.len()
                    && offset_11 + ch < src_data.len()
                {
                    let p00 = src_data[offset_00 + ch] as f32;
                    let p10 = src_data[offset_10 + ch] as f32;
                    let p01 = src_data[offset_01 + ch] as f32;
                    let p11 = src_data[offset_11 + ch] as f32;

                    let top = p00 * (1.0 - fx) + p10 * fx;
                    let bottom = p01 * (1.0 - fx) + p11 * fx;
                    let value = top * (1.0 - fy) + bottom * fy;

                    dst_data[dst_offset + ch] = value.round() as u8;
                }
            }
        }
    }

    (Arc::from(dst_data.into_boxed_slice()), dst_stride)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scale_frame_half() {
        // 4x4 → 2x2 downscale
        let src_data = vec![255u8; 4 * 4 * 4]; // White pixels
        let (scaled, stride) = scale_frame(
            &src_data,
            4,
            4,
            16,
            2,
            2,
            PixelFormat::Bgra,
        );

        assert_eq!(stride, 8);
        assert_eq!(scaled.len(), 2 * 2 * 4);
        // Should still be mostly white
        assert!(scaled[0] > 200);
    }

    #[test]
    fn test_scale_frame_no_change() {
        // Same size, should still work
        let src_data = vec![128u8; 2 * 2 * 4];
        let (scaled, stride) = scale_frame(
            &src_data,
            2,
            2,
            8,
            2,
            2,
            PixelFormat::Bgra,
        );

        assert_eq!(stride, 8);
        assert_eq!(scaled.len(), 2 * 2 * 4);
    }
}
