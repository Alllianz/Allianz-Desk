use crate::contacts::{Contact, ContactMatcher};
use crate::ui::terminal::TerminalUI;

pub struct ContactsView;

impl ContactsView {
    /// Muestra la lista de contactos en un formato limpio y sobrio
    pub fn select_contact(contacts: &[Contact]) -> Option<Contact> {
        if contacts.is_empty() {
            println!("  (No se encontraron contactos en la agenda)");
            return None;
        }

        let mut current_filter = String::new();

        loop {
            let filtered: Vec<&Contact> = ContactMatcher::search(contacts, &current_filter);

            println!();
            println!("── Contactos ───────────────────────────────────────────────");
            if !current_filter.is_empty() {
                println!(" Filtro: '{}' ({} encontrados)", current_filter, filtered.len());
            } else {
                println!(" Total: {} contactos", contacts.len());
            }
            println!("────────────────────────────────────────────────────────────");

            let display_limit = 20;
            for (idx, contact) in filtered.iter().take(display_limit).enumerate() {
                println!("   [{:2}] {:<25} {}", idx + 1, contact.name, contact.phone);
            }

            if filtered.len() > display_limit {
                println!("   ... y {} contactos más. Escribe para filtrar.", filtered.len() - display_limit);
            }

            println!("────────────────────────────────────────────────────────────");
            println!(" Ingrese número [1-{}], escriba para buscar, o 0 para volver:", filtered.len().min(display_limit));

            let input = TerminalUI::read_line(" > ");

            if input == "0" || input.eq_ignore_ascii_case("cancelar") || input.eq_ignore_ascii_case("volver") {
                return None;
            }

            if let Ok(choice) = input.parse::<usize>() {
                if choice >= 1 && choice <= filtered.len().min(display_limit) {
                    return Some(filtered[choice - 1].clone());
                } else {
                    println!(" Opción fuera de rango.");
                }
            } else {
                current_filter = input;
            }
        }
    }
}
