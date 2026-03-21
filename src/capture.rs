use anyhow::Result;
use nokhwa::pixel_format::RgbAFormat;
use nokhwa::utils::{ApiBackend, RequestedFormat, RequestedFormatType};
use nokhwa::Camera;

pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub struct Capture {
    camera: Camera,
}

impl Capture {
    pub fn new() -> Result<Self> {
        let devices = nokhwa::query(ApiBackend::Auto).unwrap_or_default();
        eprintln!("Available cameras: {devices:?}");

        // Try each camera with a permissive format request
        let requested = RequestedFormat::new::<RgbAFormat>(RequestedFormatType::None);

        for info in &devices {
            eprintln!(
                "Trying camera: {} ({})",
                info.human_name(),
                info.description()
            );
            match Camera::new(info.index().clone(), requested.clone()) {
                Ok(mut camera) => {
                    // Try to open the stream to verify it actually works
                    match camera.open_stream() {
                        Ok(()) => {
                            eprintln!("Opened camera: {}", info.human_name());
                            return Ok(Self { camera });
                        }
                        Err(e) => {
                            eprintln!("  Stream failed: {e}");
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  Failed: {e}");
                    continue;
                }
            }
        }

        anyhow::bail!("No working camera found")
    }

    pub fn try_pull_frame(&mut self) -> Option<Frame> {
        let frame = self.camera.frame().ok()?;
        let decoded = frame.decode_image::<RgbAFormat>().ok()?;
        let width = decoded.width();
        let height = decoded.height();
        let data = decoded.into_raw();

        Some(Frame {
            width,
            height,
            data,
        })
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        let _ = self.camera.stop_stream();
    }
}
