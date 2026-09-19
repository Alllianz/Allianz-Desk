use crate::adb::AdbClient;
use crate::screen::TouchHandler;
use egui::{Color32, Rect, Response, Sense, TextureHandle, Ui, Vec2};

pub struct ScreenViewWidget;

impl ScreenViewWidget {
    pub fn show(
        ui: &mut Ui,
        texture: Option<&TextureHandle>,
        touch_handler: &mut TouchHandler,
        device_resolution: (i32, i32),
        adb: &AdbClient,
    ) -> Response {
        let available_size = ui.available_size();
        let (dev_w, dev_h) = device_resolution;
        let aspect_ratio = dev_w as f32 / dev_h.max(1) as f32;

        let mut target_h = available_size.y - 45.0; // Espacio para barra de navegación inferior
        let mut target_w = target_h * aspect_ratio;

        if target_w > available_size.x {
            target_w = available_size.x;
            target_h = target_w / aspect_ratio;
        }

        let (rect, response) = ui.allocate_exact_size(Vec2::new(target_w, target_h), Sense::click_and_drag());

        // Dibujar borde y fondo del dispositivo
        ui.painter().rect_filled(rect, 8.0, Color32::from_rgb(15, 15, 20));

        if let Some(tex) = texture {
            ui.painter().image(
                tex.id(),
                rect,
                Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            ui.painter().text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Conectando pantalla por USB...",
                egui::FontId::proportional(14.0),
                Color32::GRAY,
            );
        }

        touch_handler.handle_interaction(&response, rect, device_resolution, adb);

        // Barra de botones de navegación de Android (Atrás, Inicio, Recientes)
        ui.horizontal(|ui| {
            if ui.button(" ◀ Atrás ").clicked() {
                let _ = adb.send_key_event(4); // KEYCODE_BACK
            }
            if ui.button(" ⌂ Inicio ").clicked() {
                let _ = adb.send_key_event(3); // KEYCODE_HOME
            }
            if ui.button(" ▢ Recientes ").clicked() {
                let _ = adb.send_key_event(187); // KEYCODE_APP_SWITCH
            }
            if ui.button(" ⚡ Desbloquear ").clicked() {
                let _ = adb.ensure_device_awake();
            }
        });

        response
    }
}
