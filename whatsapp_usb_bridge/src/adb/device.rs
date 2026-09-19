use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub serial: String,
    #[allow(dead_code)]
    pub state: String,
    pub model: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AdbClient {
    pub adb_path: PathBuf,
    pub selected_device: Option<String>,
}

impl AdbClient {
    pub fn new(custom_path: Option<&str>) -> Result<Self> {
        let adb_path = Self::find_adb_executable(custom_path)?;
        Ok(Self {
            adb_path,
            selected_device: None,
        })
    }

    fn find_adb_executable(custom_path: Option<&str>) -> Result<PathBuf> {
        if let Some(path_str) = custom_path {
            let p = PathBuf::from(path_str);
            if p.exists() {
                return Ok(p);
            }
        }

        // Buscar en PATH del sistema
        if let Ok(output) = Command::new("adb").arg("version").output() {
            if output.status.success() {
                return Ok(PathBuf::from("adb"));
            }
        }

        // Rutas habituales en Windows y Android SDK
        let local_appdata = std::env::var("LOCALAPPDATA").unwrap_or_default();
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();

        let possible_locations = [
            r"platform-tools\adb.exe".to_string(),
            r".\platform-tools\adb.exe".to_string(),
            r"..\platform-tools\adb.exe".to_string(),
            format!(r"{}\Android\Sdk\platform-tools\adb.exe", local_appdata),
            format!(r"{}\Android\platform-tools\adb.exe", local_appdata),
            format!(r"{}\AppData\Local\Android\Sdk\platform-tools\adb.exe", user_profile),
            format!(r"{}\platform-tools\adb.exe", user_profile),
            r"C:\platform-tools\adb.exe".to_string(),
            r"C:\Android\platform-tools\adb.exe".to_string(),
        ];

        for loc in &possible_locations {
            let p = Path::new(loc);
            if p.exists() {
                return Ok(p.to_path_buf());
            }
        }

        Err(anyhow!(
            "No se encontró el ejecutable 'adb'. Asegúrate de tener Android SDK / Platform-Tools instalado o especifica la ruta con --adb-path."
        ))
    }

    pub fn list_devices(&self) -> Result<Vec<DeviceInfo>> {
        let output = Command::new(&self.adb_path)
            .arg("devices")
            .arg("-l")
            .output()
            .with_context(|| "Fallo al ejecutar adb devices")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Error al listar dispositivos: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut devices = Vec::new();

        for line in stdout.lines().skip(1) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let serial = parts[0].to_string();
                let state = parts[1].to_string();

                let mut model = None;
                for part in &parts[2..] {
                    if let Some(m) = part.strip_prefix("model:") {
                        model = Some(m.to_string());
                    }
                }

                devices.push(DeviceInfo {
                    serial,
                    state,
                    model,
                });
            }
        }

        Ok(devices)
    }

    pub fn set_device(&mut self, serial: String) {
        self.selected_device = Some(serial);
    }

    pub fn execute_shell(&self, shell_command: &str) -> Result<String> {
        let mut cmd = Command::new(&self.adb_path);
        
        if let Some(ref serial) = self.selected_device {
            cmd.arg("-s").arg(serial);
        }

        cmd.arg("shell").arg(shell_command);

        let output = cmd.output().with_context(|| {
            format!("Error al ejecutar comando en dispositivo: {}", shell_command)
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && !stderr.is_empty() && stdout.is_empty() {
            return Err(anyhow!("Error en adb shell: {}", stderr));
        }

        Ok(stdout)
    }

    /// Captura un fotograma de la pantalla del dispositivo en formato PNG binario
    pub fn capture_screen_bytes(&self) -> Result<Vec<u8>> {
        let mut cmd = Command::new(&self.adb_path);
        
        if let Some(ref serial) = self.selected_device {
            cmd.arg("-s").arg(serial);
        }

        cmd.arg("exec-out").arg("screencap").arg("-p");

        let output = cmd.output().with_context(|| "Error al capturar pantalla vía adb exec-out screencap")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Fallo al capturar pantalla: {}", stderr));
        }

        Ok(output.stdout)
    }

    #[allow(dead_code)]
    pub fn get_notifications_raw(&self) -> Result<String> {
        self.execute_shell("dumpsys notification --noredact")
    }

    pub fn send_key_event(&self, keycode: i32) -> Result<()> {
        self.execute_shell(&format!("input keyevent {}", keycode))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn tap(&self, x: i32, y: i32) -> Result<()> {
        self.execute_shell(&format!("input tap {} {}", x, y))?;
        Ok(())
    }

    pub fn input_text(&self, text: &str) -> Result<()> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\'', "\\'")
            .replace(' ', "%s")
            .replace('&', "\\&")
            .replace('<', "\\<")
            .replace('>', "\\>")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace(';', "\\;")
            .replace('|', "\\|")
            .replace('$', "\\$");

        self.execute_shell(&format!("input text \"{}\"", escaped))?;
        Ok(())
    }

    /// Configura el dispositivo para operar de forma autónoma sin apagarse ni bloquearse mientras esté conectado por USB
    pub fn setup_always_on(&self) -> Result<()> {
        // 1. Despertar pantalla si está apagada
        let _ = self.execute_shell("input keyevent 224"); // KEYCODE_WAKEUP
        // 2. Descartar bloqueo de pantalla deslizable
        let _ = self.execute_shell("wm dismiss-keyguard");
        // 3. Mantener pantalla encendida siempre conectado a USB
        let _ = self.execute_shell("svc power stayon true");
        let _ = self.execute_shell("settings put global stay_on_while_plugged_in 3");
        let _ = self.execute_shell("settings put system screen_off_timeout 2147483647");
        Ok(())
    }

    /// Despierta el dispositivo, quita la pantalla de bloqueo y configura para no apagarse mientras esté por USB
    pub fn ensure_device_awake(&self) -> Result<()> {
        let _ = self.execute_shell("input keyevent 224"); // KEYCODE_WAKEUP
        let _ = self.execute_shell("wm dismiss-keyguard");
        let _ = self.execute_shell("svc power stayon true");
        Ok(())
    }

    /// Inyecta texto instantáneamente mediante el portapapeles de Android y la acción PEGAR (admite emojis y caracteres especiales)
    #[allow(dead_code)]
    pub fn set_clipboard_and_paste(&self, text: &str) -> Result<()> {
        // Escapar comillas dobles y barras para cmd clipboard
        let escaped = text.replace('\\', "\\\\").replace('"', "\\\"");
        let cmd = format!("cmd clipboard set text \"{}\"", escaped);
        
        let res = self.execute_shell(&cmd);
        let clipboard_ok = match res {
            Ok(ref out) if !out.contains("Error") && !out.contains("Exception") => true,
            _ => false,
        };

        if clipboard_ok {
            // Pegar contenido del portapapeles
            let _ = self.send_key_event(279); // KEYCODE_PASTE
        } else {
            // Fallback a escritura estándar por ADB
            self.input_text(text)?;
        }

        Ok(())
    }

    /// Oculta el teclado virtual (Soft Keyboard) para que no tape los botones ni interfiera con el texto
    pub fn dismiss_keyboard(&self) -> Result<()> {
        let _ = self.execute_shell("input keyevent 111"); // KEYCODE_ESCAPE
        Ok(())
    }

    /// Obtiene la resolución de la pantalla del dispositivo (ancho, alto)
    pub fn get_screen_size(&self) -> Result<(i32, i32)> {
        let output = self.execute_shell("wm size")?;
        // Formato esperado: "Physical size: 1080x2400" o "Override size: 1080x2400"
        for line in output.lines() {
            if let Some(pos) = line.find("size:") {
                let size_str = line[pos + 5..].trim();
                let dims: Vec<&str> = size_str.split('x').collect();
                if dims.len() == 2 {
                    if let (Ok(w), Ok(h)) = (dims[0].trim().parse::<i32>(), dims[1].trim().parse::<i32>()) {
                        return Ok((w, h));
                    }
                }
            }
        }
        // Resolución por defecto habitual si no se puede determinar
        Ok((1080, 2400))
    }

    /// Vuelca la jerarquía visual de la pantalla actual en formato XML usando uiautomator
    pub fn dump_ui_hierarchy(&self) -> Result<String> {
        let temp_xml = "/data/local/tmp/window_dump.xml";
        let _ = self.execute_shell(&format!("uiautomator dump {}", temp_xml));
        let xml_content = self.execute_shell(&format!("cat {}", temp_xml))?;
        Ok(xml_content)
    }

    /// Simula un desplazamiento (swipe) en pantalla
    #[allow(dead_code)]
    pub fn swipe(&self, x1: i32, y1: i32, x2: i32, y2: i32, duration_ms: u32) -> Result<()> {
        self.execute_shell(&format!("input swipe {} {} {} {} {}", x1, y1, x2, y2, duration_ms))?;
        Ok(())
    }
}
