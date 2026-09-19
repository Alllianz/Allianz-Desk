use super::models::{ChatMessage, MessageDirection};
use crate::adb::AdbClient;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
struct ParsedNode {
    text: String,
    res_id: String,
    x1: i32,
    y1: i32,
    x2: i32,
    #[allow(dead_code)]
    y2: i32,
}

pub struct WhatsAppReader<'a> {
    adb: &'a AdbClient,
}

impl<'a> WhatsAppReader<'a> {
    pub fn new(adb: &'a AdbClient) -> Self {
        Self { adb }
    }

    /// Abre la conversación de WhatsApp asociada a un número de teléfono y lee los mensajes
    pub fn read_conversation_by_phone(&self, phone_number: &str) -> Result<(String, Vec<ChatMessage>)> {
        let clean_phone: String = phone_number.chars().filter(|c| c.is_ascii_digit()).collect();
        if clean_phone.is_empty() {
            return Err(anyhow!("Número de teléfono inválido"));
        }

        let uri = format!("https://api.whatsapp.com/send?phone={}", clean_phone);
        let cmd = format!("am start -a android.intent.action.VIEW -d \"{}\"", uri);
        self.adb.execute_shell(&cmd)?;

        // Pequeña pausa rápida para que abra la interfaz
        thread::sleep(Duration::from_millis(1000));

        self.read_active_chat()
    }

    /// Lee los mensajes y el título del chat activo en la pantalla de WhatsApp
    pub fn read_active_chat(&self) -> Result<(String, Vec<ChatMessage>)> {
        let (screen_w, _) = self.adb.get_screen_size().unwrap_or((1080, 2400));
        let xml = self.adb.dump_ui_hierarchy()?;

        if xml.trim().is_empty() {
            return Err(anyhow!("No se pudo obtener la jerarquía visual de la pantalla."));
        }

        let title = Self::extract_chat_title(&xml).unwrap_or_else(|| "Conversación Actual".to_string());
        let messages = Self::parse_chat_xml(&xml, screen_w);
        Ok((title, messages))
    }

    /// Extrae el título del chat o grupo de la barra superior de WhatsApp
    fn extract_chat_title(xml: &str) -> Option<String> {
        let title_re = Regex::new(
            r#"<node[^>]*?text="([^"]+)"[^>]*?resource-id="[^"]*?(?:conversation_contact_name|conversation_title|title)"[^>]*?>"#
        ).ok()?;

        for cap in title_re.captures_iter(xml) {
            let t = cap[1].trim();
            if !t.is_empty() && t != "WhatsApp" {
                return Some(t.to_string());
            }
        }
        None
    }

    /// Parsea el XML de UI Automator extrayendo burbujas de chat y autores en grupos
    pub fn parse_chat_xml(xml: &str, screen_width: i32) -> Vec<ChatMessage> {
        let mut raw_nodes = Vec::new();

        let node_re = Regex::new(
            r#"<node[^>]*?text="([^"]*)"[^>]*?resource-id="([^"]*)"[^>]*?bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]""#
        ).unwrap();

        for cap in node_re.captures_iter(xml) {
            let text = cap[1].trim().to_string();
            let res_id = cap[2].to_string();
            let x1: i32 = cap[3].parse().unwrap_or(0);
            let y1: i32 = cap[4].parse().unwrap_or(0);
            let x2: i32 = cap[5].parse().unwrap_or(0);
            let y2: i32 = cap[6].parse().unwrap_or(0);

            if text.is_empty() {
                continue;
            }

            // Filtrar encabezados fijos de la barra superior o barra inferior de entrada
            if res_id.contains("entry")
                || (y1 < 160 && (res_id.contains("title") || res_id.contains("conversation_contact_name")))
                || res_id.contains("conversation_contact_status")
                || text == "Escribe un mensaje"
                || text == "Mensaje"
                || text == "Type a message"
            {
                continue;
            }

            raw_nodes.push(ParsedNode {
                text,
                res_id,
                x1,
                y1,
                x2,
                y2,
            });
        }

        // Ordenar nodos verticalmente por su posición en pantalla
        raw_nodes.sort_by_key(|n| n.y1);

        let mut messages = Vec::new();
        let mut pending_author: Option<String> = None;

        for node in &raw_nodes {
            let center_x = (node.x1 + node.x2) / 2;
            let is_outgoing = center_x > (screen_width / 2);

            // Detectar nombre del autor en grupos (suele ser un id con 'name' o 'author' a la izquierda arriba del mensaje)
            let is_author_node = (node.res_id.contains("name") || node.res_id.contains("author") || node.res_id.contains("sender"))
                && !is_outgoing
                && !node.res_id.contains("message_text");

            if is_author_node {
                pending_author = Some(node.text.clone());
                continue;
            }

            // Si es texto de mensaje
            if node.res_id.contains("message_text") || node.res_id.contains("caption") || node.res_id.is_empty() {
                let direction = if is_outgoing {
                    MessageDirection::Outgoing
                } else {
                    MessageDirection::Incoming
                };

                let author = if is_outgoing {
                    None
                } else {
                    pending_author.take()
                };

                messages.push(ChatMessage::new(
                    node.text.clone(),
                    author,
                    None,
                    direction,
                    None,
                ));
            }
        }

        // Fallback si no se detectaron por resource-id
        if messages.is_empty() {
            for node in &raw_nodes {
                let center_x = (node.x1 + node.x2) / 2;
                let direction = if center_x > (screen_width / 2) {
                    MessageDirection::Outgoing
                } else {
                    MessageDirection::Incoming
                };

                messages.push(ChatMessage::new(
                    node.text.clone(),
                    None,
                    None,
                    direction,
                    None,
                ));
            }
        }

        messages
    }
}
