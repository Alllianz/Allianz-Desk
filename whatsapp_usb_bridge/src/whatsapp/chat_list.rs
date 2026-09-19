use crate::adb::AdbClient;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ChatSummary {
    pub title: String,
    #[allow(dead_code)]
    pub last_message: Option<String>,
    #[allow(dead_code)]
    pub unread_count: Option<String>,
    pub bounds_center: (i32, i32),
}

pub struct ChatListExtractor<'a> {
    adb: &'a AdbClient,
}

impl<'a> ChatListExtractor<'a> {
    pub fn new(adb: &'a AdbClient) -> Self {
        Self { adb }
    }

    /// Abre la pantalla principal de WhatsApp y extrae los chats y grupos visibles
    pub fn get_visible_chats(&self) -> Result<Vec<ChatSummary>> {
        self.adb.execute_shell("am start -n com.whatsapp/.Main")?;
        thread::sleep(Duration::from_millis(1500));

        let xml = self.adb.dump_ui_hierarchy()?;
        if xml.trim().is_empty() {
            return Err(anyhow!("No se pudo leer la pantalla de WhatsApp"));
        }

        Ok(Self::parse_chats_from_xml(&xml))
    }

    pub fn parse_chats_from_xml(xml: &str) -> Vec<ChatSummary> {
        let mut chats = Vec::new();

        let name_re = Regex::new(
            r#"<node[^>]*?text="([^"]+)"[^>]*?resource-id="[^"]*?(?:conversations_row_contact_name|conversation_contact_name|chat_title)"[^>]*?bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]""#
        ).unwrap();

        for cap in name_re.captures_iter(xml) {
            let title = cap[1].trim().to_string();
            let x1: i32 = cap[2].parse().unwrap_or(0);
            let y1: i32 = cap[3].parse().unwrap_or(0);
            let x2: i32 = cap[4].parse().unwrap_or(0);
            let y2: i32 = cap[5].parse().unwrap_or(0);

            if !title.is_empty() && title != "WhatsApp" && title != "Buscar" && title != "Search" {
                let center_x = (x1 + x2) / 2;
                let center_y = (y1 + y2) / 2;

                chats.push(ChatSummary {
                    title,
                    last_message: None,
                    unread_count: None,
                    bounds_center: (center_x, center_y),
                });
            }
        }

        chats
    }
}
