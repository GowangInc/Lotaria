use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use std::path::Path;
use xcap::Monitor;

/// Capture the primary screen
pub struct ScreenCapture;

impl ScreenCapture {
    /// Capture the primary monitor and return (filename, png_bytes, base64)
    pub fn capture_primary(temp_dir: &Path) -> Result<CaptureResult> {
        let monitors = Monitor::all()?;
        
        let primary = monitors
            .into_iter()
            .find(|m| m.is_primary().unwrap_or(false))
            .or_else(|| Monitor::all().ok()?.into_iter().next())
            .ok_or_else(|| anyhow::anyhow!("No monitor found"))?;

        let image = primary.capture_image()?;
        
        // Convert to PNG
        let mut png_bytes: Vec<u8> = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut png_bytes, image.width(), image.height());
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            
            let mut writer = encoder.write_header()?;
            writer.write_image_data(bytemuck::cast_slice(image.as_raw()))?;
        }

        let timestamp = chrono::Local::now().timestamp();
        let filename = format!("capture_{}.png", timestamp);
        let filepath = temp_dir.join(&filename);
        
        // Save to file
        std::fs::write(&filepath, &png_bytes)?;
        
        // Convert to base64
        let base64 = STANDARD.encode(&png_bytes);

        Ok(CaptureResult {
            filename,
            png_bytes,
            base64,
            filepath,
        })
    }
}

pub struct CaptureResult {
    pub filename: String,
    pub png_bytes: Vec<u8>,
    pub base64: String,
    pub filepath: std::path::PathBuf,
}

/// Compute an 8x8 average perceptual hash from PNG bytes.
/// Returns a 16-char hex string representing a 64-bit hash.
pub fn compute_average_hash(png_bytes: &[u8]) -> String {
    use image::ImageReader;
    use std::io::Cursor;

    let img = match ImageReader::new(Cursor::new(png_bytes))
        .with_guessed_format()
        .and_then(|r| r.decode().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)))
    {
        Ok(img) => img,
        Err(_) => return "0000000000000000".to_string(),
    };

    let small = image::imageops::resize(
        &img.to_luma8(), 8, 8,
        image::imageops::FilterType::Triangle,
    );

    let pixels: Vec<u8> = small.pixels().map(|p| p.0[0]).collect();
    let mean: u64 = pixels.iter().map(|&p| p as u64).sum::<u64>() / 64;

    let mut hash: u64 = 0;
    for (i, &pixel) in pixels.iter().enumerate() {
        if pixel as u64 >= mean {
            hash |= 1 << i;
        }
    }

    format!("{:016x}", hash)
}

/// Compute similarity between two hex hash strings as percentage (0-100).
pub fn hash_similarity(hash_a: &str, hash_b: &str) -> u8 {
    let a = u64::from_str_radix(hash_a, 16).unwrap_or(0);
    let b = u64::from_str_radix(hash_b, 16).unwrap_or(0);
    let hamming = (a ^ b).count_ones();
    ((64 - hamming) as f32 / 64.0 * 100.0).round() as u8
}
