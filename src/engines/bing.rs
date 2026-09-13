use crate::core::{detect_captcha, detect_empty_results, extract_attr, extract_text_from_element, SearchError};
use crate::models::{Result as SearchResult, ResultType};
use scraper::{Html, Selector};
use uuid::Uuid;

const BING_URL: &str = "https://www.bing.com/search";

// Bing-specific selectors for captcha detection
const BING_CAPTCHA_SELECTORS: &[&str] = &[
    "div[class*='captcha']",
    "div[class*='challenge']",
    "form[action*='challenge']",
    "div[id*='captcha']",
];

// Bing-specific text markers for captcha detection
const BING_CAPTCHA_MARKERS: &[&str] = &[
    "please solve the captcha",
    "prove you're not a bot",
    "verify you're human",
    "unusual activity",
    "temporarily blocked",
];

// Bing no-results selectors
const BING_EMPTY_SELECTORS: &[&str] = &[
    "div[class*='b_no']",
    "span[class*='no-results']",
];

// Bing no-results markers - more specific to avoid false positives
const BING_EMPTY_MARKERS: &[&str] = &[
    "no results for",
    "no webpages found for",
    "no results were found",
];

pub struct BingEngine;

impl BingEngine {
    pub fn name() -> &'static str {
        "bing"
    }

    pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let http_client = crate::core::HttpClient::new()?;
        let encoded_query = urlencoding::encode(query);
        let url = format!("{}?q={}", BING_URL, encoded_query);

        let html = http_client.fetch(&url).await?;
        Self::parse(&html, limit)
    }

    fn parse(html: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        tracing::debug!("Bing parse: HTML length = {}", html.len());
        
        // Check for captcha using both selectors and text markers
        tracing::debug!("Bing: Checking for captcha");
        if detect_captcha(html, BING_CAPTCHA_SELECTORS, BING_CAPTCHA_MARKERS) {
            tracing::debug!("Bing: Captcha detected");
            return Err(SearchError::CaptchaDetected);
        }

        // Check for empty results
        tracing::debug!("Bing: Checking for empty results");
        if detect_empty_results(html, BING_EMPTY_SELECTORS, BING_EMPTY_MARKERS) {
            tracing::debug!("Bing: Empty results detected via empty results check");
            return Err(SearchError::EmptyResults);
        }

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // Bing result selectors - try multiple patterns
        let selectors = &[
            "li.b_algo",
            "li[data-layout='organic']",
            "div.b_algo",
        ];

        let mut rank = 1u32;

        for selector_str in selectors {
            tracing::debug!("Bing: Trying selector: {}", selector_str);
            if let Ok(result_selector) = Selector::parse(selector_str) {
                let count = document.select(&result_selector).count();
                tracing::debug!("Bing: Selector {} matched {} elements", selector_str, count);
                
                for element in document.select(&result_selector) {
                    if results.len() >= limit {
                        break;
                    }

                    // Extract URL from various possible selectors in Bing's structure
                    let url = match extract_attr(&element, &["h2 a", "h3 a", "a[href^='http']"], "href") {
                        Some(url) => url,
                        None => continue,
                    };

                    // Skip javascript and invalid links
                    if url.starts_with("javascript:") || url.is_empty() {
                        continue;
                    }

                    // Extract title
                    let title = match extract_text_from_element(&element, &["h2 a", "h2", "h3 a", "h3"]) {
                        Some(t) => t,
                        None => continue,
                    };

                    // Extract snippet - Bing puts it in different places
                    let snippet = extract_text_from_element(&element, &[".b_caption p", ".b_snippet", "p"])
                        .unwrap_or_default();

                    // Extract display URL
                    let display_url = extract_text_from_element(&element, &["cite"])
                        .unwrap_or_else(|| url.clone());

                    let domain = extract_domain(&url);

                    let result = SearchResult {
                        id: Uuid::new_v4().to_string(),
                        rank,
                        result_type: ResultType::Organic,
                        title,
                        url,
                        display_url,
                        snippet,
                        domain,
                        favicon: None,
                        engine: Self::name().to_string(),
                        domain_info: None,
                    };

                    results.push(result);
                    rank += 1;
                }

                if results.len() >= limit {
                    break;
                }
            }
        }

        if results.is_empty() {
            return Err(SearchError::EmptyResults);
        }

        Ok(results)
    }
}

fn extract_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(domain) = parsed.domain() {
            return domain.to_string();
        }
    }
    String::new()
}