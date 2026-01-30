use std::path::Path;

use async_trait::async_trait;

use crate::error::{TelegramError, TelegramResult};

/// Trait abstracting Telegram bot operations for testability.
///
/// Production code uses [`TelegramBot`]; tests can provide a mock implementation.
#[async_trait]
pub trait BotApi: Send + Sync {
    /// Send a text message to the given chat.
    ///
    /// Returns the Telegram message ID of the sent message.
    async fn send_message(&self, chat_id: i64, text: &str) -> TelegramResult<i32>;

    /// Send a document (file) to the given chat with an optional caption.
    ///
    /// Returns the Telegram message ID of the sent message.
    async fn send_document(
        &self,
        chat_id: i64,
        file_path: &Path,
        caption: Option<&str>,
    ) -> TelegramResult<i32>;

    /// Send a photo to the given chat with an optional caption.
    ///
    /// Returns the Telegram message ID of the sent message.
    async fn send_photo(
        &self,
        chat_id: i64,
        file_path: &Path,
        caption: Option<&str>,
    ) -> TelegramResult<i32>;
}

/// Wraps a `teloxide::Bot` and provides formatted messaging for Ralph.
pub struct TelegramBot {
    bot: teloxide::Bot,
}

impl TelegramBot {
    /// Create a new TelegramBot from a bot token.
    pub fn new(token: &str) -> Self {
        Self {
            bot: teloxide::Bot::new(token),
        }
    }

    /// Format an outgoing question message using Telegram HTML.
    ///
    /// Includes emoji, hat name, iteration number, and the question text.
    /// The question body is escaped to prevent HTML injection.
    pub fn format_question(hat: &str, iteration: u32, loop_id: &str, question: &str) -> String {
        let escaped_hat = escape_html(hat);
        let escaped_loop = escape_html(loop_id);
        let escaped_question = escape_html(question);
        format!(
            "❓ <b>{escaped_hat}</b> (iteration {iteration}, loop <code>{escaped_loop}</code>)\n\n{escaped_question}",
        )
    }

    /// Format a greeting message sent when the bot starts.
    pub fn format_greeting(loop_id: &str) -> String {
        let escaped = escape_html(loop_id);
        format!("🤖 Ralph bot online — monitoring loop <code>{escaped}</code>")
    }

    /// Format a farewell message sent when the bot shuts down.
    pub fn format_farewell(loop_id: &str) -> String {
        let escaped = escape_html(loop_id);
        format!("👋 Ralph bot shutting down — loop <code>{escaped}</code> complete")
    }
}

/// Escape special HTML characters for Telegram's HTML parse mode.
///
/// Telegram requires `<`, `>`, and `&` to be escaped in HTML-formatted messages.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[async_trait]
impl BotApi for TelegramBot {
    async fn send_message(&self, chat_id: i64, text: &str) -> TelegramResult<i32> {
        use teloxide::payloads::SendMessageSetters;
        use teloxide::prelude::*;
        use teloxide::types::ParseMode;

        let result = self
            .bot
            .send_message(teloxide::types::ChatId(chat_id), text)
            .parse_mode(ParseMode::Html)
            .await
            .map_err(|e| TelegramError::Send {
                attempts: 1,
                reason: e.to_string(),
            })?;

        Ok(result.id.0)
    }

    async fn send_document(
        &self,
        chat_id: i64,
        file_path: &Path,
        caption: Option<&str>,
    ) -> TelegramResult<i32> {
        use teloxide::payloads::SendDocumentSetters;
        use teloxide::prelude::*;
        use teloxide::types::{InputFile, ParseMode};

        let input_file = InputFile::file(file_path);
        let mut request = self
            .bot
            .send_document(teloxide::types::ChatId(chat_id), input_file);

        if let Some(cap) = caption {
            request = request.caption(cap).parse_mode(ParseMode::Html);
        }

        let result = request.await.map_err(|e| TelegramError::Send {
            attempts: 1,
            reason: e.to_string(),
        })?;

        Ok(result.id.0)
    }

    async fn send_photo(
        &self,
        chat_id: i64,
        file_path: &Path,
        caption: Option<&str>,
    ) -> TelegramResult<i32> {
        use teloxide::payloads::SendPhotoSetters;
        use teloxide::prelude::*;
        use teloxide::types::{InputFile, ParseMode};

        let input_file = InputFile::file(file_path);
        let mut request = self
            .bot
            .send_photo(teloxide::types::ChatId(chat_id), input_file);

        if let Some(cap) = caption {
            request = request.caption(cap).parse_mode(ParseMode::Html);
        }

        let result = request.await.map_err(|e| TelegramError::Send {
            attempts: 1,
            reason: e.to_string(),
        })?;

        Ok(result.id.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// A mock BotApi for testing that records sent messages.
    struct MockBot {
        sent: Arc<Mutex<Vec<(i64, String)>>>,
        next_id: Arc<Mutex<i32>>,
        should_fail: bool,
    }

    impl MockBot {
        fn new() -> Self {
            Self {
                sent: Arc::new(Mutex::new(Vec::new())),
                next_id: Arc::new(Mutex::new(1)),
                should_fail: false,
            }
        }

        fn failing() -> Self {
            Self {
                sent: Arc::new(Mutex::new(Vec::new())),
                next_id: Arc::new(Mutex::new(1)),
                should_fail: true,
            }
        }

        fn sent_messages(&self) -> Vec<(i64, String)> {
            self.sent.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl BotApi for MockBot {
        async fn send_message(&self, chat_id: i64, text: &str) -> TelegramResult<i32> {
            if self.should_fail {
                return Err(TelegramError::Send {
                    attempts: 1,
                    reason: "mock failure".to_string(),
                });
            }
            self.sent.lock().unwrap().push((chat_id, text.to_string()));
            let mut id = self.next_id.lock().unwrap();
            let current = *id;
            *id += 1;
            Ok(current)
        }

        async fn send_document(
            &self,
            chat_id: i64,
            file_path: &Path,
            caption: Option<&str>,
        ) -> TelegramResult<i32> {
            if self.should_fail {
                return Err(TelegramError::Send {
                    attempts: 1,
                    reason: "mock failure".to_string(),
                });
            }
            let label = format!(
                "[doc:{}]{}",
                file_path.display(),
                caption.map(|c| format!(" {c}")).unwrap_or_default()
            );
            self.sent.lock().unwrap().push((chat_id, label));
            let mut id = self.next_id.lock().unwrap();
            let current = *id;
            *id += 1;
            Ok(current)
        }

        async fn send_photo(
            &self,
            chat_id: i64,
            file_path: &Path,
            caption: Option<&str>,
        ) -> TelegramResult<i32> {
            if self.should_fail {
                return Err(TelegramError::Send {
                    attempts: 1,
                    reason: "mock failure".to_string(),
                });
            }
            let label = format!(
                "[photo:{}]{}",
                file_path.display(),
                caption.map(|c| format!(" {c}")).unwrap_or_default()
            );
            self.sent.lock().unwrap().push((chat_id, label));
            let mut id = self.next_id.lock().unwrap();
            let current = *id;
            *id += 1;
            Ok(current)
        }
    }

    #[test]
    fn format_question_includes_hat_and_loop() {
        let msg = TelegramBot::format_question("Builder", 3, "main", "Which DB should I use?");
        assert!(msg.contains("<b>Builder</b>"));
        assert!(msg.contains("iteration 3"));
        assert!(msg.contains("<code>main</code>"));
        assert!(msg.contains("Which DB should I use?"));
    }

    #[test]
    fn format_question_escapes_html_in_content() {
        let msg = TelegramBot::format_question("Hat", 1, "loop-1", "Use <b>this</b> & that?");
        assert!(msg.contains("&lt;b&gt;this&lt;/b&gt;"));
        assert!(msg.contains("&amp; that?"));
    }

    #[test]
    fn format_greeting_includes_loop_id() {
        let msg = TelegramBot::format_greeting("feature-auth");
        assert!(msg.contains("<code>feature-auth</code>"));
        assert!(msg.contains("online"));
    }

    #[test]
    fn format_farewell_includes_loop_id() {
        let msg = TelegramBot::format_farewell("main");
        assert!(msg.contains("<code>main</code>"));
        assert!(msg.contains("shutting down"));
    }

    #[test]
    fn escape_html_handles_special_chars() {
        assert_eq!(
            super::escape_html("a < b & c > d"),
            "a &lt; b &amp; c &gt; d"
        );
        assert_eq!(super::escape_html("no specials"), "no specials");
        assert_eq!(super::escape_html(""), "");
    }

    #[tokio::test]
    async fn mock_bot_send_message_succeeds() {
        let bot = MockBot::new();
        let id = bot.send_message(123, "hello").await.unwrap();
        assert_eq!(id, 1);

        let sent = bot.sent_messages();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], (123, "hello".to_string()));
    }

    #[tokio::test]
    async fn mock_bot_send_message_increments_id() {
        let bot = MockBot::new();
        let id1 = bot.send_message(123, "first").await.unwrap();
        let id2 = bot.send_message(123, "second").await.unwrap();
        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[tokio::test]
    async fn mock_bot_failure_returns_send_error() {
        let bot = MockBot::failing();
        let result = bot.send_message(123, "hello").await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            TelegramError::Send { attempts: 1, .. }
        ));
    }
}
