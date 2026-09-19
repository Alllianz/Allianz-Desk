use crate::adb::AdbClient;
use egui::{Pos2, Rect, Response};
use std::time::Instant;

pub struct TouchHandler {
    drag_start_pos: Option<Pos2>,
    drag_start_time: Option<Instant>,
}

impl Default for TouchHandler {
    fn default() -> Self {
        Self {
            drag_start_pos: None,
            drag_start_time: None,
        }
    }
}

impl TouchHandler {
    pub fn handle_interaction(
        &mut self,
        response: &Response,
        screen_rect: Rect,
        device_resolution: (i32, i32),
        adb: &AdbClient,
    ) {
        let (dev_w, dev_h) = device_resolution;
        let rect_w = screen_rect.width();
        let rect_h = screen_rect.height();

        if rect_w <= 0.0 || rect_h <= 0.0 {
            return;
        }

        // 1. Inicio de toque / arrastre
        if response.drag_started() {
            if let Some(pos) = response.interact_pointer_pos() {
                if screen_rect.contains(pos) {
                    self.drag_start_pos = Some(pos);
                    self.drag_start_time = Some(Instant::now());
                }
            }
        }

        // 2. Final de toque / clic / arrastre
        if response.drag_stopped() || response.clicked() {
            if let Some(start_pos) = self.drag_start_pos.take() {
                let end_pos = response.interact_pointer_pos().unwrap_or(start_pos);
                let duration_ms = self.drag_start_time
                    .map(|t| t.elapsed().as_millis() as u32)
                    .unwrap_or(100)
                    .max(80)
                    .min(1000);

                let start_rel_x = ((start_pos.x - screen_rect.min.x) / rect_w).clamp(0.0, 1.0);
                let start_rel_y = ((start_pos.y - screen_rect.min.y) / rect_h).clamp(0.0, 1.0);
                let end_rel_x = ((end_pos.x - screen_rect.min.x) / rect_w).clamp(0.0, 1.0);
                let end_rel_y = ((end_pos.y - screen_rect.min.y) / rect_h).clamp(0.0, 1.0);

                let x1 = (start_rel_x * dev_w as f32) as i32;
                let y1 = (start_rel_y * dev_h as f32) as i32;
                let x2 = (end_rel_x * dev_w as f32) as i32;
                let y2 = (end_rel_y * dev_h as f32) as i32;

                let distance_sq = (x1 - x2).pow(2) + (y1 - y2).pow(2);

                if distance_sq < 400 {
                    // Clic simple -> Tap
                    let _ = adb.tap(x1, y1);
                } else {
                    // Desplazamiento -> Swipe
                    let _ = adb.swipe(x1, y1, x2, y2, duration_ms);
                }
            } else if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if screen_rect.contains(pos) {
                        let rel_x = ((pos.x - screen_rect.min.x) / rect_w).clamp(0.0, 1.0);
                        let rel_y = ((pos.y - screen_rect.min.y) / rect_h).clamp(0.0, 1.0);
                        let x = (rel_x * dev_w as f32) as i32;
                        let y = (rel_y * dev_h as f32) as i32;
                        let _ = adb.tap(x, y);
                    }
                }
            }
        }
    }
}
