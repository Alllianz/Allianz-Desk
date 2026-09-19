use super::models::Contact;

pub struct ContactMatcher;

impl ContactMatcher {
    /// Filtra la lista de contactos según un término de búsqueda (nombre o número)
    pub fn search<'a>(contacts: &'a [Contact], query: &str) -> Vec<&'a Contact> {
        let q = query.trim().to_lowercase();
        if q.is_empty() {
            return contacts.iter().collect();
        }

        let q_digits: String = q.chars().filter(|c| c.is_ascii_digit()).collect();

        contacts
            .iter()
            .filter(|c| {
                let name_match = c.name.to_lowercase().contains(&q);
                let phone_match = !q_digits.is_empty() && c.phone.contains(&q_digits);
                name_match || phone_match
            })
            .collect()
    }

    /// Asegura que el número de teléfono tenga un formato adecuado para WhatsApp
    pub fn format_for_whatsapp(phone: &str, default_country_code: Option<&str>) -> String {
        let mut digits: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();

        // Si no tiene código de país (ej. tiene menos de 10 dígitos o no empieza con prefijo conocido),
        // y se provee un código de país por defecto (ej. "54"), se le antepone.
        if let Some(cc) = default_country_code {
            let clean_cc: String = cc.chars().filter(|c| c.is_ascii_digit()).collect();
            if !digits.starts_with(&clean_cc) && digits.len() <= 10 {
                digits = format!("{}{}", clean_cc, digits);
            }
        }

        digits
    }
}
