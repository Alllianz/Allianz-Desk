use crate::adb::AdbClient;
use crate::contacts::{Contact, ContactExtractor};
use crate::gui::screen_view::ScreenViewWidget;
use crate::gui::side_panel::{SidePanelState, SidePanelView};
use crate::llm::LmStudioClient;
use crate::screen::{ScreenStreamer, TouchHandler};
use eframe::egui;
use egui::TextureHandle;

pub struct WhatsAppBridgeApp {
    adb: AdbClient,
    llm: LmStudioClient,
    country_code: Option<String>,
    contacts: Vec<Contact>,
    streamer: ScreenStreamer,
    touch_handler: TouchHandler,
    side_state: SidePanelState,
    screen_texture: Option<TextureHandle>,
    device_resolution: (i32, i32),
}

impl WhatsAppBridgeApp {
    pub fn new(
        _cc: &eframe::CreationContext<'_>,
        adb: AdbClient,
        llm: LmStudioClient,
        country_code: Option<String>,
    ) -> Self {
        let _ = adb.setup_always_on();
        let device_resolution = adb.get_screen_size().unwrap_or((1080, 2400));
        
        let extractor = ContactExtractor::new(&adb);
        let contacts = extractor.fetch_contacts().unwrap_or_default();

        let streamer = ScreenStreamer::new(adb.clone(), 20);
        let touch_handler = TouchHandler::default();
        let side_state = SidePanelState::default();

        Self {
            adb,
            llm,
            country_code,
            contacts,
            streamer,
            touch_handler,
            side_state,
            screen_texture: None,
            device_resolution,
        }
    }
}

impl eframe::App for WhatsAppBridgeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 1. Recibir nuevos fotogramas de pantalla del streamer
        while let Ok(frame) = self.streamer.frame_receiver.try_recv() {
            self.screen_texture = Some(ctx.load_texture("phone_screen", frame, egui::TextureOptions::LINEAR));
        }

        // 2. Panel lateral derecho de acciones y contactos
        egui::SidePanel::right("side_panel")
            .min_width(320.0)
            .max_width(450.0)
            .show(ctx, |ui| {
                SidePanelView::show(
                    ui,
                    &mut self.side_state,
                    &self.contacts,
                    &self.adb,
                    &self.llm,
                    self.country_code.as_deref(),
                );
            });

        // 3. Panel central con la pantalla del celular interactiva
        egui::CentralPanel::default().show(ctx, |ui| {
            ScreenViewWidget::show(
                ui,
                self.screen_texture.as_ref(),
                &mut self.touch_handler,
                self.device_resolution,
                &self.adb,
            );
        });

        // Forzar repintado continuo para animación fluida de pantalla
        ctx.request_repaint();
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.streamer.stop();
    }
}
