mod adb;
mod contacts;
mod gui;
mod llm;
mod screen;
mod ui;
mod whatsapp;

use adb::AdbClient;
use clap::Parser;
use contacts::{Contact, ContactExtractor, ContactMatcher};
use eframe::egui;
use gui::WhatsAppBridgeApp;
use llm::LmStudioClient;
use std::thread;
use std::time::Duration;
use ui::{ChatView, ContactsView, TerminalUI};
use whatsapp::{ChatListExtractor, WhatsAppNavigator, WhatsAppReader, WhatsAppSender};

#[derive(Parser, Debug)]
#[command(
    name = "whatsapp-usb-bridge",
    author = "Allianz & Fausto",
    version = "0.5.0",
    about = "Lector, gestor y visor de pantalla interactivo de WhatsApp vía USB (ADB) con GUI Egui y LM Studio"
)]
struct Args {
    /// Ruta personalizada al ejecutable adb (opcional)
    #[arg(short, long)]
    adb_path: Option<String>,

    /// Serial del dispositivo específico si hay múltiples conectados
    #[arg(short, long)]
    device: Option<String>,

    /// Código de país por defecto para números locales (ej. 54 para Argentina)
    #[arg(short, long)]
    country_code: Option<String>,

    /// URL base de la API de LM Studio (por defecto: http://127.0.0.1:1234/v1)
    #[arg(long, default_value = "http://127.0.0.1:1234/v1")]
    lm_studio_url: String,

    /// Nombre del modelo cargado en LM Studio
    #[arg(long, default_value = "local-model")]
    lm_model: String,

    /// Ejecutar en modo consola / terminal CLI tradicional en lugar de abrir la GUI de Egui
    #[arg(long)]
    cli: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    println!("Iniciando WhatsApp USB Bridge...");

    // 1. Inicializar cliente ADB
    let mut adb_client = match AdbClient::new(args.adb_path.as_deref()) {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error al inicializar ADB: {}", e);
            eprintln!("Asegúrate de que 'adb' esté en el PATH o pasa la ruta con --adb-path <RUTA_A_ADB>");
            return Ok(());
        }
    };

    // 2. Buscar dispositivos conectados por USB
    let devices = match adb_client.list_devices() {
        Ok(devs) => devs,
        Err(e) => {
            eprintln!("Error al consultar dispositivos ADB: {}", e);
            return Ok(());
        }
    };

    if devices.is_empty() {
        TerminalUI::print_banner(None, false);
        println!("Asegúrate de conectar el celular por USB y tener habilitada la 'Depuración USB' en Ajustes.");
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
            let _ = adb_client.setup_always_on();
            dev
        }
        None => {
            eprintln!("No se encontró el dispositivo especificado.");
            return Ok(());
        }
    };

    let llm_client = LmStudioClient::new(Some(args.lm_studio_url.clone()), Some(args.lm_model.clone()));

    // 3. Si se solicita CLI (--cli), ejecutar modo consola; sino, abrir GUI en Egui
    if args.cli {
        run_cli_mode(adb_client, device_info, llm_client, args.country_code.as_deref())?;
    } else {
        println!("Abriendo interfaz gráfica Egui con Screen Mirroring en vivo...");
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 720.0])
                .with_min_inner_size([650.0, 480.0])
                .with_title("WhatsApp USB Bridge (Screen Mirror & AI Controller)"),
            ..Default::default()
        };

        let country_code = args.country_code;
        eframe::run_native(
            "WhatsApp USB Bridge",
            native_options,
            Box::new(move |cc| {
                Ok(Box::new(WhatsAppBridgeApp::new(
                    cc,
                    adb_client,
                    llm_client,
                    country_code,
                )))
            }),
        )
        .map_err(|e| anyhow::anyhow!("Error en GUI de Egui: {}", e))?;
    }

    Ok(())
}

/// Modo consola / Terminal tradicional
fn run_cli_mode(
    adb_client: AdbClient,
    device_info: adb::DeviceInfo,
    llm_client: LmStudioClient,
    country_code: Option<&str>,
) -> anyhow::Result<()> {
    let extractor = ContactExtractor::new(&adb_client);
    let mut cached_contacts: Vec<Contact> = Vec::new();

    loop {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
        let lm_online = llm_client.is_online();
        TerminalUI::print_banner(Some(&device_info), lm_online);
        TerminalUI::print_main_menu();

        let main_choice = TerminalUI::read_line("");

        match main_choice.as_str() {
            "1" => {
                handle_read_whatsapp(
                    &adb_client,
                    &extractor,
                    &mut cached_contacts,
                    &llm_client,
                    country_code,
                )?;
            }
            "2" => {
                handle_send_message(&adb_client, &extractor, &mut cached_contacts, country_code)?;
            }
            "3" => {
                handle_lm_studio_chat(&llm_client)?;
            }
            "0" => {
                println!("\nSaliendo de WhatsApp USB Bridge. ¡Hasta luego Allianz!");
                break;
            }
            _ => {
                println!("Opción inválida.");
                thread::sleep(Duration::from_millis(500));
            }
        }
    }

    Ok(())
}

/// Flujo para leer una conversación o grupo de WhatsApp
fn handle_read_whatsapp(
    adb: &AdbClient,
    extractor: &ContactExtractor,
    cached_contacts: &mut Vec<Contact>,
    llm: &LmStudioClient,
    country_code: Option<&str>,
) -> anyhow::Result<()> {
    println!();
    println!("── Leer WhatsApp ───────────────────────────────────────────");
    println!("   [1] Ver chats y grupos recientes en pantalla");
    println!("   [2] Buscar chat o grupo por Nombre");
    println!("   [3] Seleccionar de la Agenda de Contactos");
    println!("   [0] Volver");
    println!("────────────────────────────────────────────────────────────");

    let choice = TerminalUI::read_line(" Selecciona una opción (0-3): ");

    match choice.as_str() {
        "1" => {
            println!(" Leyendo lista de chats y grupos en WhatsApp...");
            let list_extractor = ChatListExtractor::new(adb);
            let chats = list_extractor.get_visible_chats().unwrap_or_default();

            if chats.is_empty() {
                println!(" No se detectaron chats en pantalla. Desbloquea el teléfono si está bloqueado.");
                thread::sleep(Duration::from_secs(2));
                return Ok(());
            }

            println!();
            println!("── Chats y Grupos Recientes ────────────────────────────────");
            for (idx, chat) in chats.iter().enumerate() {
                println!("   [{:2}] {}", idx + 1, chat.title);
            }
            println!("────────────────────────────────────────────────────────────");

            let sel = TerminalUI::read_line(" Ingrese número para abrir (o 0 para volver): ");
            if let Ok(idx) = sel.parse::<usize>() {
                if idx >= 1 && idx <= chats.len() {
                    let target_chat = &chats[idx - 1];
                    let (x, y) = target_chat.bounds_center;
                    adb.tap(x, y)?;
                    thread::sleep(Duration::from_millis(1000));

                    chat_reading_loop(adb, &target_chat.title, llm)?;
                }
            }
        }
        "2" => {
            let search_term = TerminalUI::read_line(" Escribe el nombre del contacto o grupo: ");
            if !search_term.is_empty() {
                println!(" Buscando '{}' en WhatsApp...", search_term);
                let navigator = WhatsAppNavigator::new(adb);
                if let Err(e) = navigator.open_chat_by_name(&search_term) {
                    eprintln!(" Error al abrir chat: {}", e);
                    thread::sleep(Duration::from_secs(2));
                } else {
                    chat_reading_loop(adb, &search_term, llm)?;
                }
            }
        }
        "3" => {
            if cached_contacts.is_empty() {
                println!(" Cargando contactos...");
                *cached_contacts = extractor.fetch_contacts().unwrap_or_default();
            }

            if let Some(contact) = ContactsView::select_contact(cached_contacts) {
                println!(" Abriendo conversación de {}...", contact.name);
                let reader = WhatsAppReader::new(adb);
                let formatted_phone = ContactMatcher::format_for_whatsapp(&contact.phone, country_code);
                let (title, _) = reader.read_conversation_by_phone(&formatted_phone).unwrap_or((contact.name.clone(), Vec::new()));
                chat_reading_loop(adb, &title, llm)?;
            }
        }
        _ => {}
    }

    Ok(())
}

/// Loop interactivo para leer, hacer scroll, responder o usar IA dentro de un chat abierto
fn chat_reading_loop(adb: &AdbClient, chat_title: &str, llm: &LmStudioClient) -> anyhow::Result<()> {
    let reader = WhatsAppReader::new(adb);
    let navigator = WhatsAppNavigator::new(adb);
    let sender = WhatsAppSender::new(adb);

    loop {
        print!("{}[2J{}[1;1H", 27 as char, 27 as char);
        let (title, messages) = reader.read_active_chat().unwrap_or((chat_title.to_string(), Vec::new()));
        ChatView::render_chat(&title, &messages);

        println!(" Acciones en este chat:");
        println!("   [1] Escribir y enviar mensaje");
        println!("   [2] Cargar mensajes anteriores (Scroll arriba)");
        println!("   [3] Refrescar pantalla");
        println!("   [4] Sugerir respuesta con IA (LM Studio)");
        println!("   [5] Resumir conversación con IA (LM Studio)");
        println!("   [0] Volver");
        println!("────────────────────────────────────────────────────────────");

        let opt = TerminalUI::read_line(" Opción (0-5): ");

        match opt.as_str() {
            "1" => {
                let msg = TerminalUI::read_line(" Mensaje a enviar: ");
                if !msg.is_empty() {
                    if let Err(e) = sender.type_and_send_in_active_chat(&msg) {
                        eprintln!(" Error al enviar: {}", e);
                    } else {
                        println!(" Mensaje enviado.");
                    }
                    thread::sleep(Duration::from_millis(600));
                }
            }
            "2" => {
                let _ = navigator.scroll_up_in_chat();
                thread::sleep(Duration::from_millis(400));
            }
            "3" => {
                thread::sleep(Duration::from_millis(200));
            }
            "4" => {
                println!(" Consultando a LM Studio para redactar respuesta...");
                let instruction = TerminalUI::read_line(" Instrucción específica (opcional, Enter para auto): ");
                let custom = if instruction.is_empty() { None } else { Some(instruction.as_str()) };

                match llm.generate_reply(&title, &messages, custom) {
                    Ok(suggested_reply) => {
                        println!();
                        println!("── Sugerencia de LM Studio ─────────────────────────────────");
                        println!(" {}", suggested_reply);
                        println!("────────────────────────────────────────────────────────────");
                        println!(" [1] Enviar esta respuesta | [2] Editar antes de enviar | [0] Descartar");
                        let sub_opt = TerminalUI::read_line(" > ");

                        if sub_opt == "1" {
                            if let Err(e) = sender.type_and_send_in_active_chat(&suggested_reply) {
                                eprintln!(" Error al enviar: {}", e);
                            } else {
                                println!(" Mensaje enviado.");
                            }
                            thread::sleep(Duration::from_millis(600));
                        } else if sub_opt == "2" {
                            let edited = TerminalUI::read_line(" Edita el mensaje: ");
                            if !edited.is_empty() {
                                if let Err(e) = sender.type_and_send_in_active_chat(&edited) {
                                    eprintln!(" Error al enviar: {}", e);
                                } else {
                                    println!(" Mensaje enviado.");
                                }
                                thread::sleep(Duration::from_millis(600));
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(" Error de LM Studio: {}", e);
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            "5" => {
                println!(" Generando resumen del chat con LM Studio...");
                match llm.summarize_chat(&title, &messages) {
                    Ok(summary) => {
                        println!();
                        println!("── Resumen de la Conversación (LM Studio) ──────────────────");
                        println!("{}", summary);
                        println!("────────────────────────────────────────────────────────────");
                        let _ = TerminalUI::read_line(" Presiona Enter para continuar...");
                    }
                    Err(e) => {
                        eprintln!(" Error de LM Studio: {}", e);
                        thread::sleep(Duration::from_secs(2));
                    }
                }
            }
            "0" | "" => {
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

/// Flujo para mandar un mensaje
fn handle_send_message(
    adb: &AdbClient,
    extractor: &ContactExtractor,
    cached_contacts: &mut Vec<Contact>,
    country_code: Option<&str>,
) -> anyhow::Result<()> {
    println!();
    println!("── Mandar un Mensaje ───────────────────────────────────────");
    println!("   [1] Seleccionar de la Agenda de Contactos");
    println!("   [2] Buscar Chat o Grupo por Nombre");
    println!("   [3] Escribir Número directo (+549...)");
    println!("   [0] Volver");
    println!("────────────────────────────────────────────────────────────");

    let choice = TerminalUI::read_line(" Selecciona destinatario (0-3): ");
    let sender = WhatsAppSender::new(adb);

    match choice.as_str() {
        "1" => {
            if cached_contacts.is_empty() {
                println!(" Cargando contactos...");
                *cached_contacts = extractor.fetch_contacts().unwrap_or_default();
            }

            if let Some(contact) = ContactsView::select_contact(cached_contacts) {
                println!(" Destinatario: {} ({})", contact.name, contact.phone);
                let msg = TerminalUI::read_line(" Mensaje a enviar: ");
                if !msg.is_empty() {
                    println!(" Enviando...");
                    let formatted_phone = ContactMatcher::format_for_whatsapp(&contact.phone, country_code);
                    if let Err(e) = sender.send_to_phone(&formatted_phone, &msg) {
                        eprintln!(" Error al enviar: {}", e);
                    } else {
                        println!(" Mensaje enviado.");
                    }
                    thread::sleep(Duration::from_millis(800));
                }
            }
        }
        "2" => {
            let name = TerminalUI::read_line(" Nombre del contacto o grupo: ");
            if !name.is_empty() {
                let msg = TerminalUI::read_line(" Mensaje a enviar: ");
                if !msg.is_empty() {
                    println!(" Buscando '{}' y enviando...", name);
                    let navigator = WhatsAppNavigator::new(adb);
                    if let Err(e) = navigator.open_chat_by_name(&name) {
                        eprintln!(" Error al buscar: {}", e);
                    } else {
                        if let Err(e) = sender.type_and_send_in_active_chat(&msg) {
                            eprintln!(" Error al enviar: {}", e);
                        } else {
                            println!(" Mensaje enviado.");
                        }
                    }
                    thread::sleep(Duration::from_millis(800));
                }
            }
        }
        "3" => {
            let phone = TerminalUI::read_line(" Número con código de país (ej. +54911...): ");
            if !phone.is_empty() {
                let msg = TerminalUI::read_line(" Mensaje: ");
                if !msg.is_empty() {
                    println!(" Enviando...");
                    let formatted_phone = ContactMatcher::format_for_whatsapp(&phone, country_code);
                    if let Err(e) = sender.send_to_phone(&formatted_phone, &msg) {
                        eprintln!(" Error al enviar: {}", e);
                    } else {
                        println!(" Mensaje enviado.");
                    }
                    thread::sleep(Duration::from_millis(800));
                }
            }
        }
        _ => {}
    }

    Ok(())
}

/// Chat directo con LM Studio
fn handle_lm_studio_chat(llm: &LmStudioClient) -> anyhow::Result<()> {
    println!();
    println!("── Asistente IA (LM Studio) ────────────────────────────────");
    println!(" Conectado a: {}", llm.base_url);
    println!(" Escribe tu consulta para la LLM (o '0' para volver):");
    println!("────────────────────────────────────────────────────────────");

    loop {
        let prompt = TerminalUI::read_line(" IA > ");
        if prompt == "0" || prompt.eq_ignore_ascii_case("salir") || prompt.eq_ignore_ascii_case("volver") {
            break;
        }

        if !prompt.is_empty() {
            println!(" Pensando...");
            let messages = vec![
                llm::LlmMessage::system("Eres un asistente inteligente, conciso y preciso."),
                llm::LlmMessage::user(prompt),
            ];

            match llm.chat_completion(messages, Some(0.7)) {
                Ok(response) => {
                    println!();
                    println!("{}", response);
                    println!();
                }
                Err(e) => {
                    eprintln!(" Error de LM Studio: {}", e);
                }
            }
        }
    }

    Ok(())
}
