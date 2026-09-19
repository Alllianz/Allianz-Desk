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

    /// Envía un mensaje a un número telefónico abriendo WhatsApp con el texto precargado y pulsando Enviar
    pub fn send_to_phone(&self, phone_number: &str, message: &str) -> Result<()> {
        let clean_phone = phone_number
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>();

        if clean_phone.is_empty() {
            return Err(anyhow!("El número de teléfono no es válido"));
        }

        self.adb.ensure_device_awake()?;

        let encoded_text = urlencoding_simple(message);
        let uri = format!("https://api.whatsapp.com/send?phone={}&text={}", clean_phone, encoded_text);

        // 1. Abrir WhatsApp con el texto ya precargado en la caja de entrada
        let cmd = format!("am start -a android.intent.action.VIEW -d \"{}\"", uri);
        self.adb.execute_shell(&cmd)?;

        // 2. Esperar a que cargue la vista (800ms)
        thread::sleep(Duration::from_millis(800));

        // 3. Ocultar teclado si está desplegado para que el botón esté despejado
        let _ = self.adb.dismiss_keyboard();
        thread::sleep(Duration::from_millis(150));

        // 4. Pulsar el botón verde de enviar
        self.tap_send_button()?;

        Ok(())
    }

    /// Escribe en el campo de texto de la conversación activa y pulsa enviar
    pub fn type_and_send_in_active_chat(&self, message: &str) -> Result<()> {
        self.adb.ensure_device_awake()?;
        let (width, height) = self.adb.get_screen_size().unwrap_or((1080, 2400));

        // 1. Tocar primero el campo de texto ("Escribe un mensaje") para asegurar el foco del cursor
        let input_box_x = (width as f64 * 0.35) as i32;
        let input_box_y = (height as f64 * 0.94) as i32;
        self.adb.tap(input_box_x, input_box_y)?;
        thread::sleep(Duration::from_millis(150));

        // 2. Escribir el texto mediante input text
        self.adb.input_text(message)?;
        thread::sleep(Duration::from_millis(250));

        // 3. Ocultar el teclado virtual para despejar la vista
        let _ = self.adb.dismiss_keyboard();
        thread::sleep(Duration::from_millis(150));

        // 4. Tocar el botón de enviar (que al haber texto ya es la flecha de enviar y no el micrófono)
        self.tap_send_button()?;

        Ok(())
    }

    /// Toca con precisión el botón verde de enviar en la esquina inferior derecha
    pub fn tap_send_button(&self) -> Result<()> {
        let (width, height) = self.adb.get_screen_size().unwrap_or((1080, 2400));
        
        let tap_x = (width as f64 * 0.92) as i32;
        let tap_y = (height as f64 * 0.94) as i32;

        self.adb.tap(tap_x, tap_y)?;

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
