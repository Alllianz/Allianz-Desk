use crate::adb::AdbClient;
use anyhow::{anyhow, Result};
use std::thread;
use std::time::Duration;

pub struct WhatsAppSender<'a> {
    adb: &'a AdbClient,
}

impl<'a> WhatsAppSender<'a> {
    pub fn new(adb: &'a AdbClient) -> Self {
        Self { adb }
    }

    /// Envía un mensaje de WhatsApp a un número telefónico (con código de país ej. +54911...)
    pub fn send_to_phone(&self, phone_number: &str, message: &str) -> Result<()> {
        let clean_phone = phone_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        if clean_phone.is_empty() {
            return Err(anyhow!("El número de teléfono no es válido"));
        }

        let encoded_text = urlencoding_simple(message);
        let uri = format!("https://api.whatsapp.com/send?phone={}&text={}", clean_phone, encoded_text);

        // Lanzar Intent para abrir WhatsApp directamente en la conversación con el texto precargado
        let cmd = format!("am start -a android.intent.action.VIEW -d \"{}\"", uri);
        self.adb.execute_shell(&cmd)?;

        // Pequeña pausa para asegurar que la interfaz de WhatsApp cargue el botón de envío
        thread::sleep(Duration::from_millis(1800));

        // Simular pulsar Enter o pulsar sobre el botón de envío
        // En WhatsApp al abrir con texto predefinido por Intent, la tecla ENTER (66) o TAB + ENTER envía el mensaje
        self.adb.send_key_event(66)?;

        Ok(())
    }

    /// Responder directamente a una conversación escribiendo en pantalla y enviando
    pub fn type_and_send_in_active_chat(&self, message: &str) -> Result<()> {
        self.adb.input_text(message)?;
        thread::sleep(Duration::from_millis(300));
        self.adb.send_key_event(66)?; // Enter
        Ok(())
    }
}

fn urlencoding_simple(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
