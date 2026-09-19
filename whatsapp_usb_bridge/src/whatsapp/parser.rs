use super::models::WhatsAppMessage;
use regex::Regex;

pub struct WhatsAppParser {
    re_title: Regex,
    re_text: Regex,
    re_big_text: Regex,
    re_key: Regex,
}

impl WhatsAppParser {
    pub fn new() -> Self {
        Self {
            re_title: Regex::new(r"android\.title=String\s*\((.*?)\)").unwrap(),
            re_text: Regex::new(r"android\.text=String\s*\((.*?)\)").unwrap(),
            re_big_text: Regex::new(r"android\.bigText=String\s*\((.*?)\)").unwrap(),
            re_key: Regex::new(r"key=([^\s]+)").unwrap(),
        }
    }

    pub fn parse_dumpsys(&self, dumpsys_output: &str) -> Vec<WhatsAppMessage> {
        let mut messages = Vec::new();
        let records = dumpsys_output.split("NotificationRecord(");

        for record in records {
            // Verificar si pertenece a com.whatsapp o com.whatsapp.w4b (WhatsApp Business)
            let is_whatsapp = record.contains("pkg=com.whatsapp") || record.contains("com.whatsapp");
            if !is_whatsapp {
                continue;
            }

            let mut title = None;
            let mut text = None;
            let mut key = None;

            if let Some(caps) = self.re_title.captures(record) {
                if let Some(m) = caps.get(1) {
                    let t = m.as_str().trim();
                    if !t.is_empty() && t != "WhatsApp" && !t.starts_with("Comprobando si hay mensajes") {
                        title = Some(t.to_string());
                    }
                }
            }

            if let Some(caps) = self.re_big_text.captures(record) {
                if let Some(m) = caps.get(1) {
                    let b = m.as_str().trim();
                    if !b.is_empty() {
                        text = Some(b.to_string());
                    }
                }
            } else if let Some(caps) = self.re_text.captures(record) {
                if let Some(m) = caps.get(1) {
                    let t = m.as_str().trim();
                    if !t.is_empty() {
                        text = Some(t.to_string());
                    }
                }
            }

            if let Some(caps) = self.re_key.captures(record) {
                if let Some(m) = caps.get(1) {
                    key = Some(m.as_str().to_string());
                }
            }

            if let (Some(sender), Some(content)) = (title, text) {
                // Filtrar notificaciones del sistema de WhatsApp como copias de seguridad o servicios en segundo plano
                if sender == "WhatsApp Web" || content.contains("mensajes nuevos") && sender == "WhatsApp" {
                    continue;
                }

                let msg = WhatsAppMessage::new(sender, content, key);
                messages.push(msg);
            }
        }

        messages
    }
}
