use crate::adb::DeviceInfo;
use crate::whatsapp::WhatsAppMessage;
use colored::*;
use std::io::{self, Write};

pub struct TerminalUI;

impl TerminalUI {
    pub fn print_banner(device: Option<&DeviceInfo>) {
        println!("{}", "==========================================================".cyan());
        println!("{}", "      WHATSAPP USB-C BRIDGE (ANDROID <-> RUST CLI)       ".bold().green());
        println!("{}", "==========================================================".cyan());

        match device {
            Some(dev) => {
                let model_str = dev.model.as_deref().unwrap_or("Dispositivo Android");
                println!(
                    "{} {} ({}) [Estado: {}]",
                    "● Conectado por USB:".green().bold(),
                    model_str.yellow(),
                    dev.serial.white(),
                    dev.state.green()
                );
            }
            None => {
                println!("{}", "○ Ningún dispositivo conectado por USB-C.".red().bold());
            }
        }
        println!("{}", "----------------------------------------------------------".cyan());
    }

    pub fn render_messages(messages: &[WhatsAppMessage]) {
        println!("\n{}", "--- BANDEJA DE MENSAJES ENTRANTES ---".bold().yellow());

        if messages.is_empty() {
            println!("{}", "No hay mensajes nuevos detectados en las notificaciones.".dimmed());
            return;
        }

        for (idx, msg) in messages.iter().enumerate() {
            let time_str = msg.timestamp.format("%H:%M:%S").to_string();
            let prefix = if msg.is_group {
                format!("[Grupo: {}]", msg.group_name.as_deref().unwrap_or("Desconocido")).magenta()
            } else {
                "[Directo]".cyan()
            };

            println!(
                "[{}] {} {} ({}):",
                (idx + 1).to_string().bold().green(),
                prefix,
                msg.sender.bold().white(),
                time_str.dimmed()
            );
            println!("    💬 {}\n", msg.content.bright_white());
        }
    }

    pub fn print_menu() {
        println!("{}", "----------------------------------------------------------".cyan());
        println!("{}", "ACCIONES DISPONIBLES:".bold().white());
        println!("  {} Responder a un mensaje reciente", "[1]".green().bold());
        println!("  {} Enviar mensaje directo a un número", "[2]".green().bold());
        println!("  {} Refrescar bandeja de mensajes", "[3]".green().bold());
        println!("  {} Salir", "[4]".red().bold());
        println!("{}", "----------------------------------------------------------".cyan());
        print!("{}", "Selecciona una opción (1-4): ".bold().yellow());
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
