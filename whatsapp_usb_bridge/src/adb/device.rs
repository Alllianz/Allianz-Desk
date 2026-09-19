use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub serial: String,
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
        // Escapar caracteres para comando de shell adb
        let escaped = text
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace(' ', "%s")
            .replace('&', "\\&")
            .replace('<', "\\<")
            .replace('>', "\\>")
            .replace('(', "\\(")
            .replace(')', "\\)");

        self.execute_shell(&format!("input text \"{}\"", escaped))?;
        Ok(())
    }
}
