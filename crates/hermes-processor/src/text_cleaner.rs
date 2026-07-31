//! Text cleaning and preprocessing for Indonesian content

use regex::Regex;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{debug, warn};

/// Text cleaner for Indonesian news content
#[derive(Debug, Clone)]
pub struct TextCleaner {
    html_regex: Regex,
    url_regex: Regex,
    email_regex: Regex,
    phone_regex: Regex,
    extra_whitespace_regex: Regex,
    indonesian_stopwords: HashMap<String, bool>,
}

impl TextCleaner {
    /// Create new text cleaner with Indonesian language support
    pub fn new() -> Result<Self> {
        Ok(Self {
            html_regex: Regex::new(r"<[^>]*>")?,
            url_regex: Regex::new(r"https?://[^\s]+")?,
            email_regex: Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?,
            phone_regex: Regex::new(r"\+?[0-9]{1,4}?[-.\s]?\(?[0-9]{1,3}?\)?[-.\s]?[0-9]{1,4}[-.\s]?[0-9]{1,9}")?,
            extra_whitespace_regex: Regex::new(r"\s+")?,
            indonesian_stopwords: Self::build_indonesian_stopwords(),
        })
    }

    /// Clean text content for Indonesian market analysis
    pub fn clean_text(&self, content: &str) -> CleaningResult {
        let start_time = std::time::Instant::now();
        let original_length = content.len();
        
        debug!("🧹 Starting text cleaning for {} chars", original_length);
        
        let mut cleaned = content.to_string();
        let mut operations_applied = Vec::new();
        
        // Step 1: Remove HTML tags
        if self.html_regex.is_match(&cleaned) {
            cleaned = self.html_regex.replace_all(&cleaned, " ").to_string();
            operations_applied.push("html_removal".to_string());
        }
        
        // Step 2: Remove URLs (preserve readability)
        if self.url_regex.is_match(&cleaned) {
            cleaned = self.url_regex.replace_all(&cleaned, "[URL]").to_string();
            operations_applied.push("url_replacement".to_string());
        }
        
        // Step 3: Remove emails 
        if self.email_regex.is_match(&cleaned) {
            cleaned = self.email_regex.replace_all(&cleaned, "[EMAIL]").to_string();
            operations_applied.push("email_replacement".to_string());
        }
        
        // Step 4: Remove phone numbers
        if self.phone_regex.is_match(&cleaned) {
            cleaned = self.phone_regex.replace_all(&cleaned, "[PHONE]").to_string();
            operations_applied.push("phone_replacement".to_string());
        }
        
        // Step 5: Normalize Indonesian text
        cleaned = self.normalize_indonesian_text(&cleaned);
        operations_applied.push("indonesian_normalization".to_string());
        
        // Step 6: Fix extra whitespace
        cleaned = self.extra_whitespace_regex.replace_all(&cleaned, " ").to_string();
        cleaned = cleaned.trim().to_string();
        operations_applied.push("whitespace_normalization".to_string());
        
        let processing_duration = start_time.elapsed();
        let final_length = cleaned.len();
        
        debug!(
            original_length = original_length,
            final_length = final_length,
            reduction_percent = %format!("{:.1}%", 
                (1.0 - final_length as f64 / original_length as f64) * 100.0),
            operations = ?operations_applied,
            duration_ms = processing_duration.as_millis(),
            "✅ Text cleaning completed"
        );
        
        CleaningResult {
            original_text: content.to_string(),
            cleaned_text: cleaned,
            original_length,
            final_length,
            operations_applied,
            processing_duration_ms: processing_duration.as_millis() as u64,
        }
    }
    
    /// Normalize Indonesian text (currency, numbers, common abbreviations)
    fn normalize_indonesian_text(&self, text: &str) -> String {
        let mut normalized = text.to_string();
        
        // Indonesian currency normalization
        normalized = normalized.replace("Rp ", "IDR ");
        normalized = normalized.replace("rupiah", "IDR");
        
        // Indonesian stock exchange terms
        normalized = normalized.replace("Bursa Efek Indonesia", "BEI");
        normalized = normalized.replace("Indeks Harga Saham Gabungan", "IHSG");
        
        // Common Indonesian abbreviations
        normalized = normalized.replace("Bapak", "Bpk");
        normalized = normalized.replace("Ibu", "Ibu");
        normalized = normalized.replace("Presiden", "Pres");
        
        // Indonesian government institutions
        normalized = normalized.replace("Kementerian Keuangan", "Kemenkeu");
        normalized = normalized.replace("Bank Indonesia", "BI");
        normalized = normalized.replace("Otoritas Jasa Keuangan", "OJK");
        
        // Financial terms normalization
        normalized = normalized.replace("triliun", "T");
        normalized = normalized.replace("miliar", "B");
        normalized = normalized.replace("juta", "M");
        
        normalized
    }
    
    /// Extract key financial entities from Indonesian text
    pub fn extract_financial_entities(&self, text: &str) -> FinancialEntities {
        let mut entities = FinancialEntities::new();
        
        // Extract Indonesian stock symbols (Christian's portfolio focus)
        let stock_patterns = [
            ("BMRI", "Bank Mandiri"),
            ("BBRI", "Bank BRI"),
            ("INCO", "Vale Indonesia"),
            ("ANTM", "Aneka Tambang"),
            ("PTBA", "Bukit Asam"),
            ("TAPG", "Triputra Agro"),
            ("TLKM", "Telkom Indonesia"),
            ("ASII", "Astra International"),
            ("KLBF", "Kalbe Farma"),
            ("TSPC", "Tempo Scan Pacific"),
        ];
        
        for (symbol, name) in &stock_patterns {
            if text.contains(symbol) || text.contains(name) {
                entities.stock_symbols.push(symbol.to_string());
            }
        }
        
        // Extract currency amounts
        let currency_regex = Regex::new(r"(?i)(?:rp|idr)\s*([0-9,.]+)\s*(?:triliun|miliar|juta|ribu)?").unwrap();
        for cap in currency_regex.captures_iter(text) {
            if let Some(amount) = cap.get(1) {
                entities.currency_amounts.push(amount.as_str().to_string());
            }
        }
        
        // Extract Indonesian institutions
        let institutions = [
            "Bank Indonesia", "BI", "OJK", "Kemenkeu", "BEI", "IHSG",
            "Kementerian ESDM", "Pertamina", "PLN", "Garuda Indonesia"
        ];
        
        for institution in &institutions {
            if text.contains(institution) {
                entities.institutions.push(institution.to_string());
            }
        }
        
        entities
    }
    
    /// Build Indonesian stopwords dictionary
    fn build_indonesian_stopwords() -> HashMap<String, bool> {
        let stopwords = [
            // Common Indonesian stopwords
            "dan", "atau", "yang", "di", "ke", "dari", "untuk", "dengan", "pada",
            "dalam", "oleh", "adalah", "akan", "telah", "sudah", "belum", "tidak",
            "bukan", "juga", "hanya", "dapat", "bisa", "menjadi", "seperti", "antara",
            // Articles and pronouns
            "ini", "itu", "saya", "kami", "kita", "mereka", "dia", "ia", "nya",
            // Common verbs
            "ada", "kata", "mari", "jadi", "lalu", "maka", "agar", "bila", "jika",
            // Time markers
            "saat", "ketika", "selama", "setelah", "sebelum", "kini", "kali", "hari",
        ];
        
        stopwords.iter().map(|&word| (word.to_string(), true)).collect()
    }
    
    /// Check if word is Indonesian stopword
    pub fn is_stopword(&self, word: &str) -> bool {
        self.indonesian_stopwords.contains_key(&word.to_lowercase())
    }
}

/// Result of text cleaning operation
#[derive(Debug, Clone)]
pub struct CleaningResult {
    pub original_text: String,
    pub cleaned_text: String,
    pub original_length: usize,
    pub final_length: usize,
    pub operations_applied: Vec<String>,
    pub processing_duration_ms: u64,
}

impl CleaningResult {
    /// Calculate text reduction percentage
    pub fn reduction_percentage(&self) -> f64 {
        if self.original_length == 0 {
            return 0.0;
        }
        (1.0 - self.final_length as f64 / self.original_length as f64) * 100.0
    }
    
    /// Check if cleaning was effective
    pub fn is_effective(&self) -> bool {
        self.final_length > 0 && self.reduction_percentage() < 90.0
    }
}

/// Financial entities extracted from Indonesian text
#[derive(Debug, Clone)]
pub struct FinancialEntities {
    pub stock_symbols: Vec<String>,
    pub currency_amounts: Vec<String>,
    pub institutions: Vec<String>,
}

impl FinancialEntities {
    fn new() -> Self {
        Self {
            stock_symbols: Vec::new(),
            currency_amounts: Vec::new(),
            institutions: Vec::new(),
        }
    }
    
    /// Check if entities contain Christian's portfolio stocks
    pub fn contains_portfolio_stocks(&self) -> bool {
        let portfolio_stocks = ["BMRI", "BBRI", "INCO", "ANTM", "PTBA", "TAPG", "TLKM", "ASII", "KLBF", "TSPC"];
        self.stock_symbols.iter().any(|symbol| portfolio_stocks.contains(&symbol.as_str()))
    }
    
    /// Count total entities found
    pub fn total_entities(&self) -> usize {
        self.stock_symbols.len() + self.currency_amounts.len() + self.institutions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_cleaner_creation() {
        let cleaner = TextCleaner::new().unwrap();
        assert!(cleaner.indonesian_stopwords.len() > 0);
        assert!(cleaner.is_stopword("dan"));
        assert!(cleaner.is_stopword("atau"));
        assert!(!cleaner.is_stopword("investasi"));
    }

    #[test]
    fn test_html_removal() {
        let cleaner = TextCleaner::new().unwrap();
        let html_content = "<p>BMRI mencatatkan <strong>kinerja positif</strong> di Q4 2023.</p>";
        let result = cleaner.clean_text(html_content);
        
        assert!(!result.cleaned_text.contains("<p>"));
        assert!(!result.cleaned_text.contains("<strong>"));
        assert!(result.cleaned_text.contains("BMRI"));
        assert!(result.operations_applied.contains(&"html_removal".to_string()));
    }

    #[test]
    fn test_indonesian_normalization() {
        let cleaner = TextCleaner::new().unwrap();
        let content = "Investasi Rp 10 triliun di Bursa Efek Indonesia oleh Bank Indonesia";
        let result = cleaner.clean_text(content);
        
        assert!(result.cleaned_text.contains("IDR"));
        assert!(result.cleaned_text.contains("BEI"));
        assert!(result.cleaned_text.contains("BI"));
    }

    #[test]
    fn test_financial_entities_extraction() {
        let cleaner = TextCleaner::new().unwrap();
        let content = "BMRI dan BBRI mencatatkan laba Rp 15 triliun, kata Bank Indonesia";
        let entities = cleaner.extract_financial_entities(content);
        
        assert_eq!(entities.stock_symbols.len(), 2);
        assert!(entities.stock_symbols.contains(&"BMRI".to_string()));
        assert!(entities.stock_symbols.contains(&"BBRI".to_string()));
        assert!(entities.contains_portfolio_stocks());
        assert!(entities.institutions.len() > 0);
    }

    #[test]
    fn test_url_and_email_replacement() {
        let cleaner = TextCleaner::new().unwrap();
        let content = "Kunjungi https://example.com atau email ke info@company.co.id untuk info INCO";
        let result = cleaner.clean_text(content);
        
        assert!(result.cleaned_text.contains("[URL]"));
        assert!(result.cleaned_text.contains("[EMAIL]"));
        assert!(result.cleaned_text.contains("INCO"));
        assert!(result.operations_applied.contains(&"url_replacement".to_string()));
        assert!(result.operations_applied.contains(&"email_replacement".to_string()));
    }

    #[test]
    fn test_cleaning_result_metrics() {
        let cleaner = TextCleaner::new().unwrap();
        let content = "<h1>ANTM</h1><p>Performance report</p>   extra   spaces   ";
        let result = cleaner.clean_text(content);
        
        assert!(result.reduction_percentage() > 0.0);
        assert!(result.is_effective());
        assert!(result.final_length < result.original_length);
        assert!(result.processing_duration_ms < 1000); // Should be very fast
    }

    #[test]
    fn test_financial_entities_portfolio_detection() {
        let entities = FinancialEntities {
            stock_symbols: vec!["BMRI".to_string(), "UNKNOWN".to_string()],
            currency_amounts: vec!["10000".to_string()],
            institutions: vec!["BI".to_string()],
        };
        
        assert!(entities.contains_portfolio_stocks());
        assert_eq!(entities.total_entities(), 3);
        
        let non_portfolio = FinancialEntities {
            stock_symbols: vec!["UNKNOWN".to_string()],
            currency_amounts: vec![],
            institutions: vec![],
        };
        
        assert!(!non_portfolio.contains_portfolio_stocks());
    }
}