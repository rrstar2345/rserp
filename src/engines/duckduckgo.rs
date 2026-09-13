use crate::core::{detect_captcha, detect_empty_results, extract_attr, extract_text_from_element, SearchError};
use crate::models::{Result as SearchResult, ResultType};
use scraper::{Html, Selector};
use uuid::Uuid;

const DUCKDUCKGO_URL: &str = "https://html.duckduckgo.com/html/";

// DuckDuckGo-specific selectors for captcha detection
const DDG_CAPTCHA_SELECTORS: &[&str] = &[
    "form[action*='anomaly']",
    "input[name='challenge']",
    "div[id*='anomaly']",
    "div[class*='captcha']",
];

// DuckDuckGo-specific text markers for captcha detection
const DDG_CAPTCHA_MARKERS: &[&str] = &[
    "bots user",
    "bots use duckduckgo too",
    "human verification",
    "unusual traffic",
    "anomaly",
];

// DuckDuckGo no-results selectors
const DDG_EMPTY_SELECTORS: &[&str] = &[
    "div[class*='no-results']",
    "[data-testid='no-results']",
    "div[data-result='no-results']",
];

// DuckDuckGo no-results markers - made more specific to avoid false positives
const DDG_EMPTY_MARKERS: &[&str] = &[
    "no results for",
    "no results were found",
    "couldn't find any",
];

pub struct DuckDuckGoEngine;

impl DuckDuckGoEngine {
    pub fn name() -> &'static str {
        "duckduckgo"
    }

    pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let http_client = crate::core::HttpClient::new()?;
        let encoded_query = urlencoding::encode(query);
        let url = format!("{}?q={}&p=1", DUCKDUCKGO_URL, encoded_query);

        let html = http_client.fetch(&url).await?;
        Self::parse(&html, limit)
    }

    fn parse(html: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        tracing::debug!("DuckDuckGo parse: HTML length = {}", html.len());
        
        // Check for captcha first using both selectors and text markers
        if detect_captcha(html, DDG_CAPTCHA_SELECTORS, DDG_CAPTCHA_MARKERS) {
            tracing::debug!("DuckDuckGo: Captcha detected");
            return Err(SearchError::CaptchaDetected);
        }

        // Check for empty results
        if detect_empty_results(html, DDG_EMPTY_SELECTORS, DDG_EMPTY_MARKERS) {
            tracing::debug!("DuckDuckGo: Empty results detected");
            return Err(SearchError::EmptyResults);
        }

        let document = Html::parse_document(html);
        let mut results = Vec::new();

        // DuckDuckGo uses data-testid attribute for results
        // Try multiple selector patterns to handle different DDG layouts
        let selectors = &[
            "article[data-testid='result']",
            "article[data-testid='ad']",
            "div[data-testid='result']",
            "div[data-testid='ad']",
            "li[data-layout='organic']",
        ];

        let mut rank = 1u32;

        for selector_str in selectors {
            tracing::debug!("DuckDuckGo: Trying selector: {}", selector_str);
            if let Ok(result_selector) = Selector::parse(selector_str) {
                let count = document.select(&result_selector).count();
                tracing::debug!("DuckDuckGo: Selector {} matched {} elements", selector_str, count);
                
                for element in document.select(&result_selector) {
                    if results.len() >= limit {
                        break;
                    }

                    // Extract URL from various possible selectors
                    let url = match extract_attr(&element, &["a[data-testid='result-title-a']", "h2 a", "a[href]"], "href") {
                        Some(url) => url,
                        None => continue,
                    };

                    // Skip javascript and anchor links
                    if url.starts_with("javascript:") || url.starts_with("#") || url.is_empty() {
                        continue;
                    }

                    // Extract title from various possible selectors
                    let title = match extract_text_from_element(&element, &["h2", "span[data-testid*='result-title']", "a[data-testid='result-title-a']"]) {
                        Some(t) => t,
                        None => continue,
                    };

                    // Extract snippet
                    let snippet = extract_text_from_element(&element, &["div[data-result='snippet']", ".result__snippet", "p"])
                        .unwrap_or_default();

                    // Extract display URL
                    let display_url = extract_text_from_element(&element, &["p", "span"])
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

        tracing::debug!("DuckDuckGo: Found {} results", results.len());
        
        if results.is_empty() {
            tracing::debug!("DuckDuckGo: No results matched any selectors");
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