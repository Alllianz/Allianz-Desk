use crate::adb::AdbClient;
use crate::contacts::{Contact, ContactMatcher};
use crate::llm::LmStudioClient;
use crate::whatsapp::{WhatsAppNavigator, WhatsAppReader, WhatsAppSender};
use egui::{Color32, RichText, ScrollArea, Ui};

pub struct SidePanelState {
    pub message_input: String,
    pub contact_search: String,
    pub selected_contact: Option<Contact>,
    #[allow(dead_code)]
    pub llm_prompt: String,
    pub llm_response: String,
    #[allow(dead_code)]
    pub is_llm_loading: bool,
    pub status_message: String,
}

impl Default for SidePanelState {
    fn default() -> Self {
        Self {
            message_input: String::new(),
            contact_search: String::new(),
            selected_contact: None,
            llm_prompt: String::new(),
            llm_response: String::new(),
            is_llm_loading: false,
            status_message: "Listo".to_string(),
        }
    }
}

pub struct SidePanelView;

impl SidePanelView {
    pub fn show(
        ui: &mut Ui,
        state: &mut SidePanelState,
        contacts: &[Contact],
        adb: &AdbClient,
        llm: &LmStudioClient,
        country_code: Option<&str>,
    ) {
        ui.heading("WhatsApp Bridge");
        ui.separator();

        // 1. Sección de Envío Rápido
        ui.label(RichText::new("Enviar Mensaje").strong());
        
        let mut clear_contact = false;
        if let Some(ref c) = state.selected_contact {
            let contact_label = format!("Para: {} ({})", c.name, c.phone);
            ui.horizontal(|ui| {
                ui.label(contact_label);
                if ui.small_button("✕").clicked() {
                    clear_contact = true;
                }
            });
        }
        if clear_contact {
            state.selected_contact = None;
        }

        ui.add(
            egui::TextEdit::multiline(&mut state.message_input)
                .hint_text("Escribe tu mensaje aquí...")
                .desired_rows(3)
                .desired_width(f32::INFINITY),
        );

        ui.horizontal(|ui| {
            if ui.button(" ✉ Enviar en Chat Activo ").clicked() {
                if !state.message_input.is_empty() {
                    let sender = WhatsAppSender::new(adb);
                    if let Err(e) = sender.type_and_send_in_active_chat(&state.message_input) {
                        state.status_message = format!("Error: {}", e);
                    } else {
                        state.status_message = "Mensaje enviado.".to_string();
                        state.message_input.clear();
                    }
                }
            }

            if let Some(ref c) = state.selected_contact {
                if ui.button(" 🚀 Abrir y Enviar ").clicked() {
                    if !state.message_input.is_empty() {
                        let sender = WhatsAppSender::new(adb);
                        let phone = ContactMatcher::format_for_whatsapp(&c.phone, country_code);
                        if let Err(e) = sender.send_to_phone(&phone, &state.message_input) {
                            state.status_message = format!("Error: {}", e);
                        } else {
                            state.status_message = "Mensaje enviado a contacto.".to_string();
                            state.message_input.clear();
                        }
                    }
                }
            }
        });

        ui.add_space(10.0);
        ui.separator();

        // 2. Sección de Asistente IA (LM Studio)
        ui.label(RichText::new("Asistente IA (LM Studio)").strong());
        
        ui.horizontal(|ui| {
            if ui.button(" 🤖 Sugerir Respuesta ").clicked() {
                let reader = WhatsAppReader::new(adb);
                if let Ok((title, msgs)) = reader.read_active_chat() {
                    match llm.generate_reply(&title, &msgs, None) {
                        Ok(reply) => {
                            state.message_input = reply;
                            state.status_message = "Respuesta generada en el cuadro de texto.".to_string();
                        }
                        Err(e) => {
                            state.status_message = format!("Error LM Studio: {}", e);
                        }
                    }
                }
            }

            if ui.button(" 📋 Resumir Pantalla ").clicked() {
                let reader = WhatsAppReader::new(adb);
                if let Ok((title, msgs)) = reader.read_active_chat() {
                    match llm.summarize_chat(&title, &msgs) {
                        Ok(summary) => {
                            state.llm_response = summary;
                            state.status_message = "Resumen generado.".to_string();
                        }
                        Err(e) => {
                            state.status_message = format!("Error LM Studio: {}", e);
                        }
                    }
                }
            }
        });

        if !state.llm_response.is_empty() {
            ui.add_space(5.0);
            ui.label(RichText::new("Resumen IA:").color(Color32::LIGHT_BLUE));
            ui.label(&state.llm_response);
        }

        ui.add_space(10.0);
        ui.separator();

        // 3. Agenda de Contactos y Grupos
        ui.label(RichText::new("Contactos y Grupos").strong());
        ui.add(
            egui::TextEdit::singleline(&mut state.contact_search)
                .hint_text("Buscar contacto o grupo...")
                .desired_width(f32::INFINITY),
        );

        let filtered_contacts = ContactMatcher::search(contacts, &state.contact_search);

        ScrollArea::vertical()
            .max_height(220.0)
            .show(ui, |ui| {
                for contact in filtered_contacts.iter().take(30) {
                    ui.horizontal(|ui| {
                        if ui.link(&contact.name).clicked() {
                            state.selected_contact = Some((*contact).clone());
                        }
                        ui.label(RichText::new(&contact.phone).weak());

                        if ui.small_button("Abrir").clicked() {
                            let navigator = WhatsAppNavigator::new(adb);
                            let _ = navigator.open_chat_by_name(&contact.name);
                        }
                    });
                }
            });

        ui.add_space(10.0);
        ui.label(RichText::new(&state.status_message).weak());
    }
}
