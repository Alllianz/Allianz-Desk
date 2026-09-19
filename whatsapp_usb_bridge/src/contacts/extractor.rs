use super::models::Contact;
use crate::adb::AdbClient;
use anyhow::{anyhow, Result};
use std::collections::HashSet;

pub struct ContactExtractor<'a> {
    adb: &'a AdbClient,
}

impl<'a> ContactExtractor<'a> {
    pub fn new(adb: &'a AdbClient) -> Self {
        Self { adb }
    }

    /// Extrae la lista de contactos del dispositivo Android utilizando 'content query'
    pub fn fetch_contacts(&self) -> Result<Vec<Contact>> {
        let mut contacts = Vec::new();
        let mut seen_phones: HashSet<String> = HashSet::new();

        // 1. Intentar con el Content Provider moderno de Contactos
        let output = self.adb.execute_shell(
            "content query --uri content://com.android.contacts/data/phones --projection display_name:data1"
        );

        let raw_text = match output {
            Ok(ref text) if !text.trim().is_empty() && !text.contains("Error") => text.clone(),
            _ => {
                // 2. Fallback a la URI de contactos clásica
                self.adb.execute_shell(
                    "content query --uri content://contacts/phones/ --projection display_name:number"
                ).unwrap_or_default()
            }
        };

        if raw_text.trim().is_empty() {
            return Err(anyhow!("No se pudieron obtener contactos del teléfono. Verifica los permisos de depuración USB."));
        }

        for line in raw_text.lines() {
            let line = line.trim();
            if !line.starts_with("Row:") {
                continue;
            }

            if let Some(contact) = Self::parse_row(line) {
                if !contact.phone.is_empty() && !seen_phones.contains(&contact.phone) {
                    seen_phones.insert(contact.phone.clone());
                    contacts.push(contact);
                }
            }
        }

        // Ordenar alfabéticamente por nombre
        contacts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        Ok(contacts)
    }

    /// Parsea una línea con formato: Row: 0 display_name=Nombre, data1=+123456
    fn parse_row(row: &str) -> Option<Contact> {
        let after_row = if let Some(pos) = row.find(' ') {
            &row[pos + 1..]
        } else {
            return None;
        };

        let mut name: Option<String> = None;
        let mut phone: Option<String> = None;

        // Separar campos delimitados por comas
        let fields = after_row.split(", ");
        for field in fields {
            let parts: Vec<&str> = field.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim();

                if key == "display_name" {
                    name = Some(val.to_string());
                } else if key == "data1" || key == "number" {
                    phone = Some(val.to_string());
                }
            }
        }

        match (name, phone) {
            (Some(n), Some(p)) if !n.is_empty() && !p.is_empty() => {
                Some(Contact::new(None, n, p))
            }
            _ => None,
        }
    }
}
