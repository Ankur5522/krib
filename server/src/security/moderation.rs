use regex::Regex;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashSet;

/// Moderation result from various checks
#[derive(Debug, Clone)]
pub struct ModerationResult {
    pub is_allowed: bool,
    pub reason: Option<String>,
    #[allow(dead_code)]
    pub violation_type: Option<ModerationViolationType>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ModerationViolationType {
    Profanity,
    OffTopic,
    Spam,
    HateContent,
    HarassmentContent,
    SexualContent,
    OpenAiViolation,
}

impl ModerationResult {
    pub fn allowed() -> Self {
        Self {
            is_allowed: true,
            reason: None,
            violation_type: None,
        }
    }

    pub fn blocked(reason: String, violation_type: ModerationViolationType) -> Self {
        Self {
            is_allowed: false,
            reason: Some(reason),
            violation_type: Some(violation_type),
        }
    }
}

/// OpenAI Moderation API Response
#[derive(Debug, Deserialize)]
pub struct OpenAiModerationResponse {
    pub results: Vec<OpenAiModerationResult>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiModerationResult {
    pub categories: OpenAiCategories,
    #[allow(dead_code)]
    pub category_scores: OpenAiCategoryScores,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiCategories {
    pub hate: bool,
    pub harassment: bool,
    pub sexual: bool,
    pub violence: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub self_harm: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub sexual_minors: bool,
    #[serde(default)]
    #[allow(dead_code)]
    pub illegal: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct OpenAiCategoryScores {
    pub hate: f64,
    pub harassment: f64,
    pub sexual: f64,
    pub violence: f64,
    #[serde(default)]
    #[allow(dead_code)]
    pub self_harm: f64,
    #[serde(default)]
    #[allow(dead_code)]
    pub sexual_minors: f64,
    #[serde(default)]
    #[allow(dead_code)]
    pub illegal: f64,
}

// Compile regexes at startup for English profanity patterns
static ENGLISH_PROFANITY: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(damn|hell|crap|ass|bitch|bastard|piss|fuck|shit|asshole|dick|cock|pussy|whore|slut|cunt)\b").unwrap()
});

// Compile regexes at startup
static URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"https?://[^\s]+|www\.[^\s]+").unwrap());

// Known scam domains
static SCAM_DOMAINS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        "t.me",
        "telegram.me",
        "telegram.org",
        "bit.ly",
        "tinyurl.com",
        "goo.gl",
        "rebrand.ly",
        "ow.ly",
        "lnk.co",
        "short.link",
        "bitly.com",
        "adf.ly",
        "j.mp",
        "clickbank.net",
    ]
    .into_iter()
    .collect()
});

/// Content moderation service with profanity filter, context check, and OpenAI integration
#[derive(Clone)]
pub struct ModerationService {
    openai_api_key: Option<String>,
    http_client: Option<reqwest::Client>,
}

impl ModerationService {
    pub fn new(openai_api_key: Option<String>) -> Self {
        let http_client = openai_api_key.as_ref().map(|_| reqwest::Client::new());

        Self {
            openai_api_key,
            http_client,
        }
    }

    /// Run all moderation checks asynchronously
    /// Returns ModerationResult with the first violation found
    pub async fn moderate_message(&self, content: &str) -> ModerationResult {
        // 1. Check for profanity/vulgar language
        let profanity_result = self.check_profanity(content).await;
        if !profanity_result.is_allowed {
            return profanity_result;
        }

        // 2. Check for relevance to rentals (context check)
        let relevance_result = self.check_rental_relevance(content);
        if !relevance_result.is_allowed {
            return relevance_result;
        }

        // 3. Check for spam (URLs and patterns)
        let spam_result = self.check_spam(content);
        if !spam_result.is_allowed {
            return spam_result;
        }

        // 4. OpenAI Moderation API check (if configured)
        if let Some(result) = self.check_openai_moderation(content).await {
            if !result.is_allowed {
                return result;
            }
        }

        ModerationResult::allowed()
    }

    /// Check for profanity and vulgar language
    /// Handles English profanity patterns and Hinglish text
    async fn check_profanity(&self, content: &str) -> ModerationResult {
        // Check for English profanity patterns
        if ENGLISH_PROFANITY.is_match(content) {
            return ModerationResult::blocked(
                "Profanity or offensive language detected".to_string(),
                ModerationViolationType::Profanity,
            );
        }

        // Basic Hinglish profanity patterns (common offensive words transliterated)
        let hinglish_offensive = vec![
            r"(?i)\b(bc|bhosdike|lodu|chutiya|gaandu|gandu|harami|besharam)\b",
            r"(?i)\b(madarchod|mdarc|behenchod|bevkuf|chakka)\b",
            r"(?i)\b(randi|teri|terepa|saali|ullu|chakli)\b",
        ];

        for pattern in hinglish_offensive {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(content) {
                    return ModerationResult::blocked(
                        "Offensive or vulgar language detected".to_string(),
                        ModerationViolationType::Profanity,
                    );
                }
            }
        }

        ModerationResult::allowed()
    }

    /// Check if message is relevant to rental/property context
    /// Uses keyword density to determine relevance
    fn check_rental_relevance(&self, content: &str) -> ModerationResult {
        // Rental-related keywords
        let rental_keywords = vec![
            "room", "rooms", "flat", "flat", "apartment", "bhk", "bh", "studio", "rent",
            "rented", "rental", "lease", "property", "location", "area", "locality",
            "available", "looking", "wanted", "accommodation", "lodging", "tenant",
            "landlord", "owner", "deposit", "advance", "monthly", "furnished",
            "unfurnished", "sharing", "pg", "hostel", "shared", "attached", "bathroom",
            "kitchen", "parking", "vegetarian", "non-veg", "pets", "furnishing",
        ];

        let content_lower = content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();

        if words.is_empty() {
            return ModerationResult::allowed(); // Empty messages are OK (will be caught elsewhere)
        }

        // Count keyword matches
        let keyword_count = words
            .iter()
            .filter(|word| {
                rental_keywords
                    .iter()
                    .any(|keyword| word.contains(keyword))
            })
            .count();

        // Calculate keyword density
        let keyword_density = keyword_count as f64 / words.len() as f64;

        // Messages with less than 10% keyword density are considered off-topic
        // unless they're very short (3 words or less, which might be sparse but legitimate)
        if keyword_density < 0.1 && words.len() > 3 {
            return ModerationResult::blocked(
                "Message appears off-topic for rental platform".to_string(),
                ModerationViolationType::OffTopic,
            );
        }

        ModerationResult::allowed()
    }

    /// Check for spam patterns - multiple URLs and known scam domains
    fn check_spam(&self, content: &str) -> ModerationResult {
        // Count external URLs
        let url_matches: Vec<&str> = URL_REGEX.find_iter(content).map(|m| m.as_str()).collect();

        // Check if more than 2 URLs
        if url_matches.len() > 2 {
            return ModerationResult::blocked(
                format!(
                    "Message contains too many URLs ({} found, max 2 allowed)",
                    url_matches.len()
                ),
                ModerationViolationType::Spam,
            );
        }

        // Check for known scam domains
        for url in &url_matches {
            for scam_domain in SCAM_DOMAINS.iter() {
                if url.to_lowercase().contains(scam_domain) {
                    return ModerationResult::blocked(
                        format!("Message contains link to known scam domain: {}", scam_domain),
                        ModerationViolationType::Spam,
                    );
                }
            }
        }

        ModerationResult::allowed()
    }

    /// Check message against OpenAI's moderation API
    /// Returns None if API check is disabled or fails, Some(result) otherwise
    async fn check_openai_moderation(&self, content: &str) -> Option<ModerationResult> {
        // Skip if API key is not configured
        let api_key = self.openai_api_key.as_ref()?;
        let client = self.http_client.as_ref()?;

        // Prepare request to OpenAI Moderation API
        let request_body = serde_json::json!({
            "input": content,
            "model": "text-moderation-latest"
        });

        match client
            .post("https://api.openai.com/v1/moderations")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_body)
            .send()
            .await
        {
            Ok(response) => match response.json::<OpenAiModerationResponse>().await {
                Ok(moderation_response) => {
                    if let Some(result) = moderation_response.results.first() {
                        // Check categories
                        if result.categories.hate {
                            return Some(ModerationResult::blocked(
                                "Content violates hate speech policy".to_string(),
                                ModerationViolationType::HateContent,
                            ));
                        }

                        if result.categories.harassment {
                            return Some(ModerationResult::blocked(
                                "Content violates harassment policy".to_string(),
                                ModerationViolationType::HarassmentContent,
                            ));
                        }

                        if result.categories.sexual {
                            return Some(ModerationResult::blocked(
                                "Content violates sexual content policy".to_string(),
                                ModerationViolationType::SexualContent,
                            ));
                        }

                        // Also check violence
                        if result.categories.violence {
                            return Some(ModerationResult::blocked(
                                "Content violates violence policy".to_string(),
                                ModerationViolationType::OpenAiViolation,
                            ));
                        }
                    }
                    None
                }
                Err(e) => {
                    eprintln!("Failed to parse OpenAI moderation response: {}", e);
                    None
                }
            },
            Err(e) => {
                eprintln!("OpenAI moderation API request failed: {}", e);
                None
            }
        }
    }

    /// Helper function for external rental relevance check
    /// Can be extended with more sophisticated NLP or ML models
    #[allow(dead_code)]
    pub fn is_relevant_to_rentals(content: &str) -> bool {
        let rental_keywords = vec![
            "room", "flat", "apartment", "bhk", "rent", "rental", "property", "location",
            "available", "looking", "accommodation", "tenant", "landlord", "deposit",
        ];

        let content_lower = content.to_lowercase();
        let words: Vec<&str> = content_lower.split_whitespace().collect();

        if words.is_empty() {
            return true; // Consider empty as potentially relevant
        }

        let keyword_count = words
            .iter()
            .filter(|word| rental_keywords.iter().any(|kw| word.contains(kw)))
            .count();

        let keyword_density = keyword_count as f64 / words.len() as f64;

        keyword_density >= 0.1 || words.len() <= 3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_relevant_to_rentals() {
        assert!(ModerationService::is_relevant_to_rentals(
            "Looking for a 2 BHK flat near metro"
        ));
        assert!(ModerationService::is_relevant_to_rentals(
            "Room available for rent in Delhi"
        ));
        assert!(!ModerationService::is_relevant_to_rentals(
            "Buy this amazing product now for cheap"
        ));
        assert!(!ModerationService::is_relevant_to_rentals(
            "Check out this movie I watched yesterday"
        ));
    }

    #[tokio::test]
    async fn test_profanity_check() {
        let service = ModerationService::new(None);
        let result = service.check_profanity("This is a normal message").await;
        assert!(result.is_allowed);

        // Test with potential profanity (rustrict might catch it)
        let result = service
            .check_profanity("This message contains damn profanity")
            .await;
        // Result depends on rustrict's dictionary
    }

    #[test]
    fn test_spam_multiple_urls() {
        let service = ModerationService::new(None);
        let content_with_urls = "Check https://example.com and http://test.com and https://another.com";
        let result = service.check_spam(content_with_urls);
        assert!(!result.is_allowed);
        assert_eq!(
            result.violation_type,
            Some(ModerationViolationType::Spam)
        );
    }

    #[test]
    fn test_spam_scam_domains() {
        let service = ModerationService::new(None);
        let content = "Contact me on https://t.me/username";
        let result = service.check_spam(content);
        assert!(!result.is_allowed);
    }

    #[test]
    fn test_valid_single_url() {
        let service = ModerationService::new(None);
        let content = "Check my portfolio at https://example.com";
        let result = service.check_spam(content);
        assert!(result.is_allowed);
    }
}
