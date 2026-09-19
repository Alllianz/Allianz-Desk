use crate::adb::DeviceInfo;
use std::io::{self, Write};

pub struct TerminalUI;

impl TerminalUI {
    pub fn print_banner(device: Option<&DeviceInfo>, lm_online: bool) {
        println!("────────────────────────────────────────────────────────────");
        print!(" WhatsApp USB Bridge  ");
        match device {
            Some(dev) => {
                let model = dev.model.as_deref().unwrap_or("Android");
                print!("| Dispositivo: {} ({}) ", model, dev.serial);
            }
            None => {
                print!("| Sin dispositivo ");
            }
        }
        if lm_online {
            println!("| LM Studio: Conectado");
        } else {
            println!("| LM Studio: Desconectado");
        }
        println!("────────────────────────────────────────────────────────────");
    }

    pub fn print_main_menu() {
        println!();
        println!(" Opciones:");
        println!("   [1] Leer WhatsApp (Chats y Grupos)");
        println!("   [2] Mandar un mensaje");
        println!("   [3] Asistente IA (LM Studio)");
        println!("   [0] Salir");
        println!();
        print!(" Ingrese una opción: ");
        let _ = io::stdout().flush();
    }

    pub fn read_line(prompt: &str) -> String {
        print!("{}", prompt);
        let _ = io::stdout().flush();
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        input.trim().to_string()
    }
}
