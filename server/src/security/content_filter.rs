use regex::Regex;
use once_cell::sync::Lazy;

/// Content filter for detecting scams, spam, and policy violations
#[derive(Clone)]
pub struct ContentFilter {
    scam_url_regex: Regex,
    phone_regex: Regex,
    spam_phrases_regex: Regex,
}

/// Result of content filtering
#[derive(Debug, Clone)]
pub struct FilterResult {
    pub is_allowed: bool,
    pub reason: Option<String>,
    pub violation_type: Option<ViolationType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ViolationType {
    ScamUrl,
    EmbeddedPhone,
    SpamPhrase,
    Honeypot,
}

impl FilterResult {
    pub fn allowed() -> Self {
        Self {
            is_allowed: true,
            reason: None,
            violation_type: None,
        }
    }

    pub fn blocked(reason: String, violation_type: ViolationType) -> Self {
        Self {
            is_allowed: false,
            reason: Some(reason),
            violation_type: Some(violation_type),
        }
    }
}

// Compile regexes once at startup
static SCAM_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(t\.me|telegram\.me|telegram\.org/bot|bit\.ly|tinyurl\.com|goo\.gl|rebrand\.ly|ow\.ly)").unwrap()
});

static PHONE_REGEX: Lazy<Regex> = Lazy::new(|| {
    // Match various phone number patterns
    Regex::new(r"(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}|\+?\d{10,15}|\d{3}[-.\s]\d{3}[-.\s]\d{4}").unwrap()
});

static SPAM_PHRASES_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)(contact me on telegram|dm me|whatsapp only|text me at|call now|limited offer|act fast|click here|100% guaranteed|make money fast|free money|earn \$\d+|buy now|limited time)").unwrap()
});

impl ContentFilter {
    pub fn new() -> Self {
        Self {
            scam_url_regex: SCAM_URL_REGEX.clone(),
            phone_regex: PHONE_REGEX.clone(),
            spam_phrases_regex: SPAM_PHRASES_REGEX.clone(),
        }
    }

    /// Check if message content passes all filters
    /// 
    /// # Arguments
    /// * `message` - The message text to check
    /// 
    /// # Returns
    /// FilterResult indicating if the message is allowed and why if blocked
    pub fn check_message(&self, message: &str) -> FilterResult {
        // Check for scam URLs
        if self.scam_url_regex.is_match(message) {
            return FilterResult::blocked(
                "Message contains suspicious URL".to_string(),
                ViolationType::ScamUrl,
            );
        }

        // Check for embedded phone numbers
        if self.phone_regex.is_match(message) {
            return FilterResult::blocked(
                "Phone numbers should be in the dedicated phone field, not in the message".to_string(),
                ViolationType::EmbeddedPhone,
            );
        }

        // Check for spam phrases
        if self.spam_phrases_regex.is_match(message) {
            return FilterResult::blocked(
                "Message contains spam or suspicious phrases".to_string(),
                ViolationType::SpamPhrase,
            );
        }

        FilterResult::allowed()
    }

    /// Check if the honeypot field was filled (bot detection)
    /// 
    /// # Arguments
    /// * `honeypot_value` - The value of the honeypot field (should be empty for humans)
    /// 
    /// # Returns
    /// FilterResult indicating if this is likely a bot
    pub fn check_honeypot(&self, honeypot_value: Option<&str>) -> FilterResult {
        if let Some(value) = honeypot_value {
            if !value.is_empty() {
                return FilterResult::blocked(
                    "Bot detected via honeypot".to_string(),
                    ViolationType::Honeypot,
                );
            }
        }
        FilterResult::allowed()
    }

    /// Validate phone number format (if provided)
    /// 
    /// # Arguments
    /// * `phone` - Optional phone number string
    /// 
    /// # Returns
    /// True if phone is None or matches expected format
    pub fn validate_phone(&self, phone: Option<&str>) -> bool {
        if let Some(p) = phone {
            // Basic validation: should be 10-15 digits, can include +, -, (, ), spaces
            let cleaned = p.chars()
                .filter(|c| c.is_numeric())
                .collect::<String>();
            
            cleaned.len() >= 10 && cleaned.len() <= 15
        } else {
            true // None is valid
        }
    }

    /// Add custom suspicious patterns for runtime detection
    pub fn is_suspicious_pattern(&self, message: &str) -> bool {
        let lowercase = message.to_lowercase();
        
        // Check for excessive repetition
        if self.has_excessive_repetition(&lowercase) {
            return true;
        }

        // Check for excessive capitalization
        if self.has_excessive_caps(message) {
            return true;
        }

        false
    }

    /// Check if text has excessive character repetition (spam indicator)
    fn has_excessive_repetition(&self, text: &str) -> bool {
        let mut prev_char = '\0';
        let mut repeat_count = 0;
        
        for ch in text.chars() {
            if ch == prev_char && ch.is_alphanumeric() {
                repeat_count += 1;
                if repeat_count > 5 {
                    return true;
                }
            } else {
                repeat_count = 1;
                prev_char = ch;
            }
        }
        
        false
    }

    /// Check if text has excessive capitalization (spam indicator)
    fn has_excessive_caps(&self, text: &str) -> bool {
        if text.len() < 10 {
            return false;
        }
        
        let caps_count = text.chars().filter(|c| c.is_uppercase()).count();
        let letter_count = text.chars().filter(|c| c.is_alphabetic()).count();
        
        if letter_count == 0 {
            return false;
        }
        
        let caps_ratio = caps_count as f64 / letter_count as f64;
        caps_ratio > 0.7 // More than 70% caps
    }
}

impl Default for ContentFilter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scam_url_detection() {
        let filter = ContentFilter::new();
        
        let result = filter.check_message("Check out this t.me/scambot");
        assert!(!result.is_allowed);
        assert_eq!(result.violation_type, Some(ViolationType::ScamUrl));
        
        let result = filter.check_message("Normal message without URLs");
        assert!(result.is_allowed);
    }

    #[test]
    fn test_phone_number_detection() {
        let filter = ContentFilter::new();
        
        let result = filter.check_message("Call me at 555-123-4567");
        assert!(!result.is_allowed);
        assert_eq!(result.violation_type, Some(ViolationType::EmbeddedPhone));
    }

    #[test]
    fn test_spam_phrase_detection() {
        let filter = ContentFilter::new();
        
        let result = filter.check_message("Contact me on telegram for details");
        assert!(!result.is_allowed);
        assert_eq!(result.violation_type, Some(ViolationType::SpamPhrase));
    }

    #[test]
    fn test_honeypot() {
        let filter = ContentFilter::new();
        
        let result = filter.check_honeypot(Some("bot_filled_this"));
        assert!(!result.is_allowed);
        
        let result = filter.check_honeypot(Some(""));
        assert!(result.is_allowed);
        
        let result = filter.check_honeypot(None);
        assert!(result.is_allowed);
    }

    #[test]
    fn test_excessive_caps() {
        let filter = ContentFilter::new();
        
        assert!(filter.has_excessive_caps("HELLO THIS IS SPAM"));
        assert!(!filter.has_excessive_caps("This is normal text"));
    }
}
