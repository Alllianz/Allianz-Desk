mod adb;
mod ui;
mod whatsapp;

use adb::AdbClient;
use clap::Parser;
use colored::*;
use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use ui::TerminalUI;
use whatsapp::{WhatsAppMessage, WhatsAppParser, WhatsAppSender};

#[derive(Parser, Debug)]
#[command(
    name = "whatsapp-usb-bridge",
    author = "Fausto",
    version = "0.1.0",
    about = "Lector y gestor interactivo de WhatsApp vía USB-C (ADB) en Rust"
)]
struct Args {
    /// Ruta personalizada al ejecutable adb (opcional)
    #[arg(short, long)]
    adb_path: Option<String>,

    /// Serial del dispositivo específico si hay múltiples conectados
    #[arg(short, long)]
    device: Option<String>,

    /// Intervalo de sondeo en segundos (por defecto: 3s)
    #[arg(short, long, default_value_t = 3)]
    poll_secs: u64,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("{}", "Iniciando WhatsApp USB-C Bridge...".cyan());

    // 1. Inicializar cliente ADB
    let mut adb_client = match AdbClient::new(args.adb_path.as_deref()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("{} {}", "Error al inicializar ADB:".red().bold(), e);
            eprintln!(
                "{}",
                "Asegúrate de que 'adb' esté en el PATH o pasa la ruta con --adb-path <RUTA_A_ADB>".yellow()
            );
            return Ok(());
        }
    };

    // 2. Buscar dispositivos conectados por USB-C
    let devices = match adb_client.list_devices() {
        Ok(devs) => devs,
        Err(e) => {
            eprintln!("{} {}", "Error al consultar dispositivos ADB:".red().bold(), e);
            return Ok(());
        }
    };

    if devices.is_empty() {
        TerminalUI::print_banner(None);
        println!(
            "{}",
            "Asegúrate de conectar el celular por USB-C y tener habilitada la 'Depuración USB' en Ajustes de Desarrollador."
                .yellow()
        );
        return Ok(());
    }

    let selected_device = if let Some(serial) = args.device {
        devices.into_iter().find(|d| d.serial == serial)
    } else {
        devices.into_iter().next()
    };

    let device_info = match selected_device {
        Some(dev) => {
            adb_client.set_device(dev.serial.clone());
            dev
        }
        None => {
            eprintln!("{}", "No se encontró el dispositivo especificado.".red());
            return Ok(());
        }
    };

    let parser = WhatsAppParser::new();
    let mut stored_messages: Vec<WhatsAppMessage> = Vec::new();
    let mut known_ids: HashSet<String> = HashSet::new();

    // 3. Loop interactivo
    loop {
        // Limpiar pantalla de consola
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
        TerminalUI::print_banner(Some(&device_info));

        // Refrescar notificaciones
        if let Ok(raw_notifs) = adb_client.get_notifications_raw() {
            let new_msgs = parser.parse_dumpsys(&raw_notifs);
            for msg in new_msgs {
                if !known_ids.contains(&msg.id) {
                    known_ids.insert(msg.id.clone());
                    stored_messages.push(msg);
                }
            }
        }

        TerminalUI::render_messages(&stored_messages);
        TerminalUI::print_menu();

        let choice = TerminalUI::read_line("");

        match choice.as_str() {
            "1" => {
                if stored_messages.is_empty() {
                    println!("{}", "No hay mensajes disponibles para responder.".yellow());
                    thread::sleep(Duration::from_secs(2));
                    continue;
                }

                let num_str = TerminalUI::read_line("\nIngresa el número de mensaje a responder: ");
                if let Ok(idx) = num_str.parse::<usize>() {
                    if idx >= 1 && idx <= stored_messages.len() {
                        let target_msg = &stored_messages[idx - 1];
                        println!(
                            "Respondiendo a: {}",
                            target_msg.sender.bold().green()
                        );

                        let reply = TerminalUI::read_line("Escribe tu respuesta: ");
                        if !reply.is_empty() {
                            println!("{}", "Enviando respuesta vía USB...".cyan());
                            let sender = WhatsAppSender::new(&adb_client);
                            
                            // Si el remitente es o contiene un número de teléfono, usar Intent
                            if target_msg.sender.chars().any(|c| c.is_ascii_digit()) {
                                if let Err(e) = sender.send_to_phone(&target_msg.sender, &reply) {
                                    eprintln!("{} {}", "Error al enviar:".red(), e);
                                } else {
                                    println!("{}", "¡Mensaje enviado con éxito!".green().bold());
                                }
                            } else {
                                // Enviar escribiendo en el chat activo
                                if let Err(e) = sender.type_and_send_in_active_chat(&reply) {
                                    eprintln!("{} {}", "Error al escribir en pantalla:".red(), e);
                                } else {
                                    println!("{}", "¡Mensaje enviado con éxito!".green().bold());
                                }
                            }
                            thread::sleep(Duration::from_secs(2));
                        }
                    } else {
                        println!("{}", "Número de mensaje fuera de rango.".red());
                        thread::sleep(Duration::from_secs(1));
                    }
                }
            }
            "2" => {
                let phone = TerminalUI::read_line("\nIngresa el número de teléfono con código de país (ej. +54911xxxxxxxx): ");
                if !phone.is_empty() {
                    let msg_text = TerminalUI::read_line("Escribe el mensaje: ");
                    if !msg_text.is_empty() {
                        println!("{}", "Enviando mensaje vía USB...".cyan());
                        let sender = WhatsAppSender::new(&adb_client);
                        if let Err(e) = sender.send_to_phone(&phone, &msg_text) {
                            eprintln!("{} {}", "Error al enviar:".red(), e);
                        } else {
                            println!("{}", "¡Mensaje enviado con éxito!".green().bold());
                        }
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            "3" => {
                println!("{}", "Refrescando notificaciones...".cyan());
                thread::sleep(Duration::from_millis(500));
            }
            "4" => {
                println!("{}", "Saliendo de WhatsApp USB-C Bridge. ¡Hasta luego!".green());
                break;
            }
            _ => {
                println!("{}", "Opción inválida.".red());
                thread::sleep(Duration::from_millis(800));
            }
        }
    }

    Ok(())
}
