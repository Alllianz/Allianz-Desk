use crate::adb::AdbClient;
use anyhow::Result;
use regex::Regex;
use std::thread;
use std::time::Duration;

pub struct WhatsAppNavigator<'a> {
    adb: &'a AdbClient,
}

impl<'a> WhatsAppNavigator<'a> {
    pub fn new(adb: &'a AdbClient) -> Self {
        Self { adb }
    }

    /// Abre WhatsApp y busca un contacto o grupo por nombre, entrando a la conversación
    pub fn open_chat_by_name(&self, name_or_group: &str) -> Result<()> {
        // 1. Abrir pantalla principal de WhatsApp
        self.adb.execute_shell("am start -n com.whatsapp/.Main")?;
        thread::sleep(Duration::from_millis(1200));

        // 2. Buscar el botón de búsqueda (icono de lupa)
        let xml = self.adb.dump_ui_hierarchy().unwrap_or_default();
        if let Some((x, y)) = Self::find_search_button(&xml) {
            self.adb.tap(x, y)?;
        } else {
            // Fallback: Presionar tecla de búsqueda de Android
            let _ = self.adb.send_key_event(84); // KEYCODE_SEARCH
        }

        thread::sleep(Duration::from_millis(800));

        // 3. Escribir el término de búsqueda
        self.adb.input_text(name_or_group)?;
        thread::sleep(Duration::from_millis(1500));

        // 4. Obtener la jerarquía de resultados y hacer tap en el primer chat / resultado coincidente
        let search_xml = self.adb.dump_ui_hierarchy()?;
        if let Some((x, y)) = Self::find_first_chat_result(&search_xml) {
            self.adb.tap(x, y)?;
            thread::sleep(Duration::from_millis(1500));
            return Ok(());
        }

        // Fallback: hacer tap en la primera fila de resultados debajo de la barra de búsqueda
        let (w, _) = self.adb.get_screen_size().unwrap_or((1080, 2400));
        self.adb.tap(w / 2, 350)?;
        thread::sleep(Duration::from_millis(1500));

        Ok(())
    }

    /// Hace scroll hacia arriba en la conversación para cargar mensajes más antiguos
    pub fn scroll_up_in_chat(&self) -> Result<()> {
        let (w, h) = self.adb.get_screen_size().unwrap_or((1080, 2400));
        let center_x = w / 2;
        let start_y = (h as f64 * 0.35) as i32;
        let end_y = (h as f64 * 0.80) as i32;

        self.adb.swipe(center_x, start_y, center_x, end_y, 400)?;
        thread::sleep(Duration::from_millis(800));
        Ok(())
    }

    fn find_search_button(xml: &str) -> Option<(i32, i32)> {
        let search_re = Regex::new(
            r#"<node[^>]*?(?:resource-id="[^"]*?(?:menuitem_search|search_holder|search_src_text)"|content-desc="(?:Buscar|Search)")[^>]*?bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]""#
        ).ok()?;

        if let Some(caps) = search_re.captures(xml) {
            let x1: i32 = caps[1].parse().ok()?;
            let y1: i32 = caps[2].parse().ok()?;
            let x2: i32 = caps[3].parse().ok()?;
            let y2: i32 = caps[4].parse().ok()?;
            return Some(((x1 + x2) / 2, (y1 + y2) / 2));
        }

        None
    }

    fn find_first_chat_result(xml: &str) -> Option<(i32, i32)> {
        let result_re = Regex::new(
            r#"<node[^>]*?resource-id="[^"]*?(?:conversations_row_contact_name|conversation_contact_name|name)"[^>]*?bounds="\[(\d+),(\d+)\]\[(\d+),(\d+)\]""#
        ).ok()?;

        if let Some(caps) = result_re.captures(xml) {
            let x1: i32 = caps[1].parse().ok()?;
            let y1: i32 = caps[2].parse().ok()?;
            let x2: i32 = caps[3].parse().ok()?;
            let y2: i32 = caps[4].parse().ok()?;
            return Some(((x1 + x2) / 2, (y1 + y2) / 2));
        }

        None
    }
}
