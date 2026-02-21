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
