use serde::{Deserialize, Serialize};

/// Sanitize HTML content to prevent XSS attacks
/// Allows safe HTML tags and removes potentially dangerous ones
fn sanitize_html(input: &str) -> String {
    ammonia::Builder::default()
        .link_rel(None)
        .clean(input)
        .to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub browser_id: String,
    pub message: String,
    pub message_type: MessageType,
    pub timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Offered,
    Requested,
}

#[derive(Deserialize)]
pub struct PostMessageRequest {
    pub browser_id: String,
    pub message: String,
    pub message_type: MessageType,
    pub phone: Option<String>,
    /// Honeypot field - should be empty for legitimate users
    #[serde(default)]
    pub website: Option<String>,
    pub location: Option<String>,
}

impl ChatMessage {
    pub fn new(browser_id: String, message: String, message_type: MessageType, phone: Option<String>, location: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            browser_id,
            // Sanitize message content to prevent XSS
            message: sanitize_html(&message),
            message_type,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            phone,
            location,
        }
    }

    /// Sanitize the message field (useful when loading from storage)
    pub fn sanitize_message(&mut self) {
        self.message = sanitize_html(&self.message);
    }
}

#[derive(Debug, Serialize)]
pub struct RateLimitError {
    pub error: String,
    pub message: String,
    pub retry_after: u64,
    pub retry_after_seconds: u64,
}

impl RateLimitError {
    pub fn new(retry_after: u64) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let seconds_remaining = if retry_after > now {
            retry_after - now
        } else {
            0
        };
        
        Self {
            error: "rate_limit_exceeded".to_string(),
            message: format!("Please wait {} seconds before posting again", seconds_remaining),
            retry_after,
            retry_after_seconds: seconds_remaining,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ContentFilterError {
    pub error: String,
    pub reason: String,
}

impl ContentFilterError {
    pub fn new(reason: String) -> Self {
        Self {
            error: "Content policy violation".to_string(),
            reason,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct ReportMessageRequest {
    pub message_id: String,
    pub reported_browser_id: String,
}

#[derive(Serialize)]
pub struct ReportResponse {
    pub success: bool,
    pub message: String,
    pub reports_on_ip: usize,
}
