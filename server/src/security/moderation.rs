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

// Leet speak and character substitution map for bypass detection
static LEET_SPEAK_MAP: &[(&str, &str)] = &[
    ("@", "a"),
    ("4", "a"),
    ("1", "i"),
    ("!", "i"),
    ("3", "e"),
    ("0", "o"),
    ("5", "s"),
    ("$", "s"),
    ("7", "t"),
    ("+", "t"),
    ("8", "b"),
    ("9", "g"),
];

// Extended profanity list with semantic variations and common typos
static PROFANITY_WORDS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    vec![
        // Original offensive words
        "damn", "hell", "crap", "ass", "bitch", "bastard", "piss", "fuck", "shit",
        "asshole", "dick", "cock", "pussy", "whore", "slut", "cunt",
        // Semantic variations and euphemisms
        "fk", "f*k", "f***", "fu*k", "fck", "fcuk",
        "sh*t", "s*it", "sh1t", "shyt", "sheit",
        "b*tch", "bit*h", "b!tch", "biatch", "btch",
        "a**", "a$s", "azz", "arse",
        "h*ll", "hel", "h3ll",
        "d@mn", "damn", "dammit", "damnit",
        "c*ck", "c0ck", "c**k", "cawk",
        "pu$$y", "p*ssy", "puss1", "kitty", // some are context-dependent
        "wh0re", "wh*re", "hoar",
        "sl*t", "slyt", "sloot",
        "c*nt", "cunt", "cnt", // might catch false positives
        // Indian Hinglish variations with typos
        "bc", "b.c", "b c", "bhd",
        "mf", "m.f", "m f", "mofo",
        // Extended Hinglish (case-insensitive handled by regex)
        "lodu", "lod", "loda", "lodu",
        "chutiya", "chut", "chutya", "chutiye",
        "gaandu", "gandu", "gaand",
        "harami", "haram", "haramkhor",
        "madarchod", "madarc", "maadarc",
        "behenchod", "bewakoof", "bevkoof",
        "randi", "rand", "randiya",
        "ullu", "ull",
        "saali", "sali",
        "teri", "tere",
    ]
    .into_iter()
    .collect()
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
    /// Handles English profanity patterns, Hinglish text, leet speak, and common typos
    async fn check_profanity(&self, content: &str) -> ModerationResult {
        // Normalize the text for checking (handle leet speak, special characters, etc.)
        let normalized = self.normalize_text_for_profanity_check(content);
        let normalized_lower = normalized.to_lowercase();

        // Check direct regex match first (existing ENGLISH_PROFANITY regex)
        if ENGLISH_PROFANITY.is_match(content) {
            return ModerationResult::blocked(
                "Profanity or offensive language detected".to_string(),
                ModerationViolationType::Profanity,
            );
        }

        // Check normalized text against profanity word list
        let words: Vec<&str> = normalized_lower.split_whitespace().collect();
        for word in &words {
            // Remove punctuation from word for checking
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());
            
            if PROFANITY_WORDS.contains(clean_word) {
                return ModerationResult::blocked(
                    "Profanity or offensive language detected".to_string(),
                    ModerationViolationType::Profanity,
                );
            }

            // Check for partial matches with fuzzy detection
            if self.fuzzy_profanity_check(clean_word) {
                return ModerationResult::blocked(
                    "Offensive or vulgar language detected".to_string(),
                    ModerationViolationType::Profanity,
                );
            }
        }

        // Check for character-spaced profanity (e.g., "b i t c h", "f*** you")
        let despaced = content.to_lowercase().split_whitespace().collect::<Vec<_>>().join("");
        for word in PROFANITY_WORDS.iter() {
            if despaced.contains(word) && word.len() > 2 {
                return ModerationResult::blocked(
                    "Offensive or vulgar language detected".to_string(),
                    ModerationViolationType::Profanity,
                );
            }
        }

        // Hinglish pattern checks (unchanged for robustness)
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

    /// Normalize text by removing leet speak and special character substitutions
    fn normalize_text_for_profanity_check(&self, text: &str) -> String {
        let mut normalized = text.to_string();
        
        // Replace leet speak characters
        for (leet_char, normal_char) in LEET_SPEAK_MAP {
            normalized = normalized.replace(leet_char, normal_char);
        }
        
        // Remove extra special characters that might be used to bypass filters
        normalized = normalized
            .replace("*", "")
            .replace("!", "")
            .replace("$", "")
            .replace("@", "a")
            .replace("#", "")
            .replace("~", "")
            .replace("^", "");
        
        normalized
    }

    /// Fuzzy check for profanity - detects common misspellings and variations
    /// Returns true if word is likely a variation of a profane word
    fn fuzzy_profanity_check(&self, word: &str) -> bool {
        if word.len() < 3 {
            return false;
        }

        // Check for repeated character patterns (e.g., "fuckkkk" or "biiitch")
        let has_excessive_repeats = word
            .chars()
            .zip(word.chars().skip(1))
            .zip(word.chars().skip(2))
            .any(|((a, b), c)| a == b && b == c);

        if has_excessive_repeats && self.contains_profane_root(word) {
            return true;
        }

        // Only do Levenshtein check for words that are within a reasonable range
        // of known profane words, and only if word is at least 4 chars
        if word.len() >= 4 {
            for profane_word in PROFANITY_WORDS.iter() {
                // Only compare against profane words with similar length
                if profane_word.len() > 2 && (word.len() as i32 - profane_word.len() as i32).abs() <= 2 {
                    if self.levenshtein_distance(word, profane_word) <= 1 {
                        // Double-check it's actually a profanity variant
                        if self.is_profanity_variant(word, profane_word) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }

    /// Check if a word is a variant of a profane word (not just coincidentally similar)
    fn is_profanity_variant(&self, word: &str, profane_word: &str) -> bool {
        // Avoid false positives by checking if the word contains the core of the profane word
        if profane_word.len() > 3 {
            // For longer words, require the core to be present
            let core = &profane_word[0..std::cmp::min(4, profane_word.len())];
            return word.contains(core);
        }
        true
    }

    /// Check if word contains the root of a profane word
    fn contains_profane_root(&self, word: &str) -> bool {
        let common_roots = vec![
            "fuck", "shit", "damn", "bitch", "cock", "ass", "cunt",
            "chut", "gand", "maadar", "lod", "rand",
        ];

        for root in common_roots {
            if word.contains(root) {
                return true;
            }
        }

        false
    }

    /// Calculate Levenshtein distance between two strings
    /// Useful for detecting common typos in offensive words
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        
        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i - 1][j] + 1,     // deletion
                        matrix[i][j - 1] + 1,     // insertion
                    ),
                    matrix[i - 1][j - 1] + cost, // substitution
                );
            }
        }

        matrix[len1][len2]
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

    #[tokio::test]
    async fn test_leet_speak_profanity() {
        let service = ModerationService::new(None);
        
        // Test leet speak variations
        let test_cases = vec![
            "f*ck you",     // asterisk
            "sh1t man",     // number substitution
            "f*** off",     // asterisk censoring
            "b!tch please", // exclamation mark
            "a$$hole",      // dollar signs
            "d@mn it",      // at symbol
        ];
        
        for case in test_cases {
            let result = service.check_profanity(case).await;
            assert!(!result.is_allowed, "Failed to detect: {}", case);
        }
    }

    #[tokio::test]
    async fn test_spaced_profanity() {
        let service = ModerationService::new(None);
        
        // Test spaced out profanity
        let test_cases = vec![
            "b i t c h",
            "f u c k",
            "s h i t",
            "a s s h o l e",
        ];
        
        for case in test_cases {
            let result = service.check_profanity(case).await;
            assert!(!result.is_allowed, "Failed to detect spaced: {}", case);
        }
    }

    #[tokio::test]
    async fn test_hinglish_profanity() {
        let service = ModerationService::new(None);
        
        // Test Hinglish variations
        let test_cases = vec![
            "bc tu kaun hai",
            "lodu sale",
            "chutiya insaan",
            "gaandu harami",
            "madarchod",
            "randi ka bacha",
            "ullu banaya",
        ];
        
        for case in test_cases {
            let result = service.check_profanity(case).await;
            assert!(!result.is_allowed, "Failed to detect Hinglish: {}", case);
        }
    }

    #[tokio::test]
    async fn test_typo_variations() {
        let service = ModerationService::new(None);
        
        // Test obvious shorthand variations
        let result = service.check_profanity("fk you").await;
        assert!(!result.is_allowed, "Should detect 'fk' as profanity variant");
    }

    #[tokio::test]
    async fn test_repeated_character_profanity() {
        let service = ModerationService::new(None);
        
        // Test repeated characters with profane roots
        let result = service.check_profanity("fuckkkk").await;
        assert!(!result.is_allowed, "Should detect repeated profanity");
    }

    #[tokio::test]
    async fn test_valid_messages_not_flagged() {
        let service = ModerationService::new(None);
        
        // Test legitimate rental-related messages
        let result = service.check_profanity("Looking for a 2 BHK flat in Mumbai").await;
        assert!(result.is_allowed, "Should not flag legitimate flat rental query");
        
        let result = service.check_profanity("What's the rent for this property?").await;
        assert!(result.is_allowed, "Should not flag rent inquiry");
    }

    #[test]
    fn test_levenshtein_distance() {
        let service = ModerationService::new(None);
        
        // Test distance calculation
        assert_eq!(service.levenshtein_distance("cat", "cat"), 0);
        assert_eq!(service.levenshtein_distance("cat", "car"), 1);
        assert_eq!(service.levenshtein_distance("fuck", "fuk"), 1);
        assert_eq!(service.levenshtein_distance("shit", "sheit"), 1);
    }}