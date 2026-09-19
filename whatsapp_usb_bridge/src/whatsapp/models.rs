use chrono::{DateTime, Local};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct WhatsAppMessage {
    pub id: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub is_group: bool,
    pub group_name: Option<String>,
    pub notification_key: Option<String>,
    pub phone_number: Option<String>,
}

#[allow(dead_code)]
impl WhatsAppMessage {
    pub fn new(sender: String, content: String, notification_key: Option<String>) -> Self {
        let is_group = sender.contains('@') || sender.contains(':');
        let mut group_name = None;
        let mut actual_sender = sender.clone();

        if sender.contains(':') {
            let parts: Vec<&str> = sender.splitn(2, ':').collect();
            if parts.len() == 2 {
                group_name = Some(parts[0].trim().to_string());
                actual_sender = parts[1].trim().to_string();
            }
        }

        let id = format!("{}_{}_{}", actual_sender, content, notification_key.as_deref().unwrap_or(""));

        Self {
            id,
            sender: actual_sender,
            content,
            timestamp: Local::now(),
            is_group,
            group_name,
            notification_key,
            phone_number: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageDirection {
    Incoming,
    Outgoing,
    #[allow(dead_code)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatMessage {
    pub text: String,
    pub author: Option<String>,
    #[allow(dead_code)]
    pub time_str: Option<String>,
    pub direction: MessageDirection,
    #[allow(dead_code)]
    pub status: Option<String>,
}

impl ChatMessage {
    pub fn new(
        text: String,
        author: Option<String>,
        time_str: Option<String>,
        direction: MessageDirection,
        status: Option<String>,
    ) -> Self {
        Self {
            text,
            author,
            time_str,
            direction,
            status,
        }
    }
}
