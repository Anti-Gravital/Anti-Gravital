//! Procesamiento de imagenes para el ecosistema Anti-Gravital.
//!
//! Soporta JPEG, PNG y WebP. AVIF pendiente como TECH-DEBT.
//!
//! # Uso
//!
//! ```no_run
//! use ag_storage::AgStorage;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! # let storage = AgStorage::new(ag_storage::StorageConfig::default()).await?;
//! let processor = storage.processor();
//! // redimensionar a maximo 800x600 preservando aspect ratio
//! // let resized = processor.resize(&bytes, 800, 600)?;
//! # Ok(())
//! # }
//! ```

use crate::StorageError;
use bytes::Bytes;
use image::{imageops::FilterType, DynamicImage, ImageFormat};
use std::io::Cursor;

/// Procesador de imagenes Anti-Gravital.
///
/// Obtener via [`crate::AgStorage::processor`].
pub struct ImageProcessor;

impl ImageProcessor {
    pub(crate) fn new() -> Self {
        Self
    }

    /// Redimensiona la imagen para que quepa dentro de `max_w x max_h`
    /// preservando el aspect ratio. Usa filtro Lanczos3 (alta calidad).
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

    /// Genera un thumbnail de la imagen con dimensiones maximas `max_w x max_h`.
    ///
    /// Preserva el aspect ratio. Usa filtro Nearest (rapido, menor calidad que resize).
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

    /// Convierte la imagen a WebP lossless.
    ///
    /// # TECH-DEBT
    ///
    /// `_quality` esta ignorado — `image` 0.25 solo expone WebP lossless.
    /// Para lossy con control de calidad usar el crate `webp` en la segunda
    /// iteracion de ag-storage.
    /// - motivo: lossy WebP con calidad configurable requiere crate `webp` separado.
    /// - impacto: archivos WebP son lossless (pueden ser mas grandes que JPEG equivalente).
    /// - eliminacion esperada: segunda iteracion ag-storage en Fase 4.
    pub fn to_webp(&self, data: impl AsRef<[u8]>, _quality: u8) -> Result<Bytes, StorageError> {
        let img = load(data.as_ref())?;
        encode(img, ImageFormat::WebP)
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

    /// Genera una imagen JPEG 100x100 en memoria para tests.
    fn test_jpeg_100x100() -> Vec<u8> {
        let img = DynamicImage::new_rgb8(100, 100);
        let mut buf = Cursor::new(Vec::new());
        img.write_to(&mut buf, ImageFormat::Jpeg).unwrap();
        buf.into_inner()
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
        // verificar que el resultado decodifica como imagen valida
        assert!(
            image::load_from_memory(&result).is_ok(),
            "WebP output no decodifica como imagen valida"
        );
    }
}
