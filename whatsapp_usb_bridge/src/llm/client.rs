use super::models::{LlmChatRequest, LlmChatResponse, LlmMessage};
use crate::whatsapp::{ChatMessage, MessageDirection};
use anyhow::{anyhow, Result};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct LmStudioClient {
    pub base_url: String,
    pub model_name: String,
}

impl Default for LmStudioClient {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:1234/v1".to_string(),
            model_name: "local-model".to_string(),
        }
    }
}

impl LmStudioClient {
    pub fn new(base_url: Option<String>, model_name: Option<String>) -> Self {
        Self {
            base_url: base_url.unwrap_or_else(|| "http://127.0.0.1:1234/v1".to_string()),
            model_name: model_name.unwrap_or_else(|| "local-model".to_string()),
        }
    }

    /// Verifica si el servidor de LM Studio está activo y respondiendo
    pub fn is_online(&self) -> bool {
        let endpoint = format!("{}/models", self.base_url.trim_end_matches('/'));
        ureq::get(&endpoint)
            .timeout(Duration::from_millis(1500))
            .call()
            .is_ok()
    }

    /// Envía una solicitud de chat completion a LM Studio
    pub fn chat_completion(&self, messages: Vec<LlmMessage>, temperature: Option<f32>) -> Result<String> {
        let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let request_body = LlmChatRequest {
            model: self.model_name.clone(),
            messages,
            temperature,
            max_tokens: Some(1024),
        };

        let response = ureq::post(&endpoint)
            .timeout(Duration::from_secs(60))
            .set("Content-Type", "application/json")
            .send_json(request_body)
            .map_err(|e| anyhow!("Error conectando a LM Studio ({}): {}", self.base_url, e))?;

        let parsed: LlmChatResponse = response.into_json()
            .map_err(|e| anyhow!("Error al parsear respuesta de LM Studio: {}", e))?;

        parsed.choices
            .into_iter()
            .next()
            .map(|c| c.message.content.trim().to_string())
            .ok_or_else(|| anyhow!("LM Studio no devolvió ninguna respuesta."))
    }

    /// Genera una sugerencia de respuesta automática para una conversación de WhatsApp
    pub fn generate_reply(&self, chat_title: &str, history: &[ChatMessage], custom_instruction: Option<&str>) -> Result<String> {
        let mut messages = Vec::new();

        let system_prompt = "Eres un asistente inteligente que redacta respuestas para WhatsApp en español. \
            Mantén un tono conciso, natural y directo, acorde a una conversación de mensajería instantánea. \
            Devuelve ÚNICAMENTE el texto que el usuario va a enviar, sin explicaciones ni comillas adicionales.";
        messages.push(LlmMessage::system(system_prompt));

        let mut context_text = format!("Conversación de WhatsApp con: {}\nHistorial reciente:\n", chat_title);
        for msg in history {
            match msg.direction {
                MessageDirection::Incoming => {
                    let author = msg.author.as_deref().unwrap_or("Contacto");
                    context_text.push_str(&format!("{}: {}\n", author, msg.text));
                }
                MessageDirection::Outgoing => {
                    context_text.push_str(&format!("Yo: {}\n", msg.text));
                }
                MessageDirection::Unknown => {
                    context_text.push_str(&format!("Mensaje: {}\n", msg.text));
                }
            }
        }

        if let Some(instruction) = custom_instruction {
            context_text.push_str(&format!("\nInstrucción adicional para la respuesta: {}\n", instruction));
        } else {
            context_text.push_str("\nPor favor redacta la respuesta adecuada para responder al último mensaje recibido:\n");
        }

        messages.push(LlmMessage::user(context_text));

        self.chat_completion(messages, Some(0.7))
    }

    /// Genera un resumen conciso de los mensajes recientes de un grupo o conversación
    pub fn summarize_chat(&self, chat_title: &str, history: &[ChatMessage]) -> Result<String> {
        let mut messages = Vec::new();

        let system_prompt = "Eres un asistente que analiza conversaciones y grupos de WhatsApp en español. \
            Tu objetivo es proporcionar un resumen claro, con viñetas de los puntos clave discutidos y quién dijo qué.";
        messages.push(LlmMessage::system(system_prompt));

        let mut context_text = format!("Grupo/Chat: {}\nHistorial de mensajes:\n", chat_title);
        for msg in history {
            let author = if let Some(ref a) = msg.author {
                a.as_str()
            } else if msg.direction == MessageDirection::Outgoing {
                "Yo"
            } else {
                "Participante"
            };
            context_text.push_str(&format!("{}: {}\n", author, msg.text));
        }
        context_text.push_str("\nResume los temas principales tratados en esta conversación:");

        messages.push(LlmMessage::user(context_text));

        self.chat_completion(messages, Some(0.3))
    }
}
