use crate::whatsapp::{ChatMessage, MessageDirection};
use colored::*;

pub struct ChatView;

impl ChatView {
    /// Renderiza en consola los mensajes de una conversación, usando colores exclusivamente en las burbujas de chat
    pub fn render_chat(chat_title: &str, messages: &[ChatMessage]) {
        println!("── Conversación: {} ───────────────────────────────", chat_title);
        println!();

        if messages.is_empty() {
            println!("  (No hay mensajes visibles en pantalla)");
        } else {
            for msg in messages {
                match msg.direction {
                    MessageDirection::Incoming => {
                        let sender_tag = match &msg.author {
                            Some(author) => format!("◀ [{}]", author).cyan().bold(),
                            None => "◀ [Mensaje]".cyan().bold(),
                        };
                        println!(" {} {}", sender_tag, msg.text.white());
                    }
                    MessageDirection::Outgoing => {
                        println!(" {} {}", "▶ [Tú]".green().bold(), msg.text.bright_green());
                    }
                    MessageDirection::Unknown => {
                        println!("   {}", msg.text.white());
                    }
                }
            }
        }

        println!();
        println!("────────────────────────────────────────────────────────────");
    }
}
