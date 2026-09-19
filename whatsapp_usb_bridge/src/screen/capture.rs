use crate::adb::AdbClient;
use crossbeam_channel::{bounded, Receiver};
use egui::ColorImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub struct ScreenStreamer {
    pub frame_receiver: Receiver<ColorImage>,
    pub running: Arc<AtomicBool>,
}

impl ScreenStreamer {
    pub fn new(adb: AdbClient, target_fps: u32) -> Self {
        let (tx, rx) = bounded::<ColorImage>(2);
        let running = Arc::new(AtomicBool::new(true));
        let running_clone = running.clone();

        thread::spawn(move || {
            let interval = Duration::from_millis((1000 / target_fps.max(1)) as u64);

            while running_clone.load(Ordering::Relaxed) {
                let start = Instant::now();

                if let Ok(bytes) = adb.capture_screen_bytes() {
                    if let Ok(img) = image::load_from_memory(&bytes) {
                        let rgba = img.to_rgba8();
                        let (w, h) = rgba.dimensions();
                        let pixels = rgba.into_raw();
                        let color_image = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], &pixels);

                        let _ = tx.try_send(color_image);
                    }
                }

                let elapsed = start.elapsed();
                if elapsed < interval {
                    thread::sleep(interval - elapsed);
                }
            }
        });

        Self {
            frame_receiver: rx,
            running,
        }
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
