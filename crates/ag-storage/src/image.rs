//! Image processing for the Anti-Gravital ecosystem.
//!
//! Supports JPEG, PNG and WebP. AVIF pending (issue #145).
//!
//! # Usage
//!
//! ```no_run
//! use ag_storage::AgStorage;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! # let storage = AgStorage::new(ag_storage::StorageConfig::default()).await?;
//! let processor = storage.processor();
//! // resize to a maximum of 800x600 preserving aspect ratio
//! // let resized = processor.resize(&bytes, 800, 600)?;
//! # Ok(())
//! # }
//! ```

use crate::StorageError;
use bytes::Bytes;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io::Cursor;

/// Anti-Gravital image processor.
///
/// Obtain it via [`crate::AgStorage::processor`].
pub struct ImageProcessor;

impl ImageProcessor {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Resizes the image to fit within `max_w x max_h` preserving the
    /// aspect ratio. Uses the Lanczos3 filter (high quality).
    pub fn resize(
        &self,
        data: impl AsRef<[u8]>,
        max_w: u32,
        max_h: u32,
    ) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        let fmt = detect_format(data.as_ref());
        let resized = img.resize(max_w, max_h, FilterType::Lanczos3);
        encode(resized, fmt)
    }

    /// Generates a thumbnail of the image with maximum dimensions `max_w x max_h`.
    ///
    /// Preserves the aspect ratio. Uses the Nearest filter (fast, lower quality than resize).
    pub fn thumbnail(
        &self,
        data: impl AsRef<[u8]>,
        max_w: u32,
        max_h: u32,
    ) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        let fmt = detect_format(data.as_ref());
        let thumb = img.thumbnail(max_w, max_h);
        encode(thumb, fmt)
    }

    /// Converts the image to WebP.
    ///
    /// Without the `webp-lossy` feature, emits lossless WebP via the `image`
    /// crate and `quality` is ignored (`image` 0.25 only exposes lossless WebP).
    ///
    /// With the `webp-lossy` feature, emits lossy WebP at the given `quality`
    /// (0..=100) via the native `webp` encoder; higher means better quality and
    /// a larger file. A `quality` of 100 is treated as lossless.
    pub fn to_webp(&self, data: impl AsRef<[u8]>, quality: u8) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;

        #[cfg(feature = "webp-lossy")]
        {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            let encoder = webp::Encoder::from_rgba(rgba.as_raw(), width, height);
            let memory = if quality >= 100 {
                encoder.encode_lossless()
            } else {
                encoder.encode(f32::from(quality))
            };
            Ok(Bytes::copy_from_slice(&memory))
        }

        #[cfg(not(feature = "webp-lossy"))]
        {
            let _ = quality;
            encode(img, ImageFormat::WebP)
        }
    }
}

fn load(data: &[u8]) -> Result<DynamicImage, StorageError> {
    image::load_from_memory(data).map_err(|e| StorageError::Image(e.to_string()))
}

fn detect_format(data: &[u8]) -> ImageFormat {
    image::guess_format(data).unwrap_or(ImageFormat::Jpeg)
}

fn encode(img: DynamicImage, fmt: ImageFormat) -> Result<Bytes, StorageError> {
    let mut buf = Cursor::new(Vec::new());
    img.write_to(&mut buf, fmt)
        .map_err(|e| StorageError::Image(e.to_string()))?;
    Ok(Bytes::from(buf.into_inner()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generates a 100x100 JPEG image in memory for tests.
    fn test_jpeg_100x100() -> Vec<u8> {
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        buf.into_inner()
    }

    #[cfg(feature = "webp-lossy")]
    #[test]
    fn webp_lossy_quality_changes_size() {
        // A noisy source so quality has a measurable effect on encoded size.
        let mut rgba = image::RgbaImage::new(64, 64);
        for (x, y, px) in rgba.enumerate_pixels_mut() {
            let v = ((x * 7 + y * 13) % 256) as u8;
            *px = image::Rgba([v, v.wrapping_mul(3), v.wrapping_add(40), 255]);
        }
        let mut buf = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(rgba)
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        let png = buf.into_inner();

        let processor = ImageProcessor::new();
        let low = processor.to_webp(&png, 10).unwrap();
        let high = processor.to_webp(&png, 90).unwrap();
        assert!(!low.is_empty() && !high.is_empty());
        assert!(
            low.len() < high.len(),
            "lower quality should be smaller: {} vs {}",
            low.len(),
            high.len()
        );
    }

    #[test]
    fn image_resize_reduces_dimensions() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.resize(&src, 50, 50).unwrap();
        let resized = image::load_from_memory(&result).unwrap();
        assert!(
            resized.width() <= 50,
            "ancho esperado <= 50, obtenido: {}",
            resized.width()
        );
        assert!(
            resized.height() <= 50,
            "alto esperado <= 50, obtenido: {}",
            resized.height()
        );
    }

    #[test]
    fn image_thumbnail_max_dimensions() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.thumbnail(&src, 30, 30).unwrap();
        let thumb = image::load_from_memory(&result).unwrap();
        assert!(
            thumb.width() <= 30,
            "ancho esperado <= 30, obtenido: {}",
            thumb.width()
        );
        assert!(
            thumb.height() <= 30,
            "alto esperado <= 30, obtenido: {}",
            thumb.height()
        );
    }

    #[test]
    fn image_to_webp_produces_valid_bytes() {
        let processor = ImageProcessor::new();
        let src = test_jpeg_100x100();
        let result = processor.to_webp(&src, 85).unwrap();
        assert!(!result.is_empty());
        // verify that the result decodes as a valid image
        assert!(
            image::load_from_memory(&result).is_ok(),
            "WebP output no decodifica como imagen valida"
        );
    }
}
