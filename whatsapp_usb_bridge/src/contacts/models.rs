#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contact {
    pub id: Option<String>,
    pub name: String,
    pub phone: String,
    pub raw_phone: String,
}

impl Contact {
    pub fn new(id: Option<String>, name: String, raw_phone: String) -> Self {
        let phone = Self::sanitize_phone(&raw_phone);
        Self {
            id,
            name,
            phone,
            raw_phone,
        }
    }

    /// Limpia y normaliza el número de teléfono dejando solo dígitos y '+'
    pub fn sanitize_phone(raw: &str) -> String {
        let mut cleaned = String::new();
        let trimmed = raw.trim();
        
        let has_plus = trimmed.starts_with('+');
        if has_plus {
            cleaned.push('+');
        }

        for ch in trimmed.chars() {
            if ch.is_ascii_digit() {
                cleaned.push(ch);
            }
        }

        cleaned
    }
}
