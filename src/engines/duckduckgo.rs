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
        tracing::debug!("DuckDuckGo: Checking for captcha");
        if detect_captcha(html, DDG_CAPTCHA_SELECTORS, DDG_CAPTCHA_MARKERS) {
            tracing::debug!("DuckDuckGo: Captcha detected");
            return Err(SearchError::CaptchaDetected);
        }

        // Check for empty results
        tracing::debug!("DuckDuckGo: Checking for empty results");
        if detect_empty_results(html, DDG_EMPTY_SELECTORS, DDG_EMPTY_MARKERS) {
            tracing::debug!("DuckDuckGo: Empty results detected via empty results check");
            return Err(SearchError::EmptyResults);
        }

        let document = Html::parse_document(html);
        
        // Debug: Inspect HTML structure (safe UTF-8 slicing)
        let html_start = if html.len() > 1000 {
            html.chars().take(500).collect::<String>()
        } else {
            html[..std::cmp::min(500, html.len())].to_string()
        };
        tracing::debug!("DuckDuckGo HTML first chars: {}", html_start);
        
        let mut results = Vec::new();

        // DuckDuckGo uses different structure than expected
        // Results are in: div.result.results_links.web-result
        let selectors = &[
            "div.result.web-result",
            "div.results_links.web-result",
            "div.result.results_links",
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

                    // Extract URL from h2 > a
                    let url = match extract_attr(&element, &["h2 a", "a.result__a"], "href") {
                        Some(url) => url,
                        None => continue,
                    };

                    // Skip javascript and anchor links
                    if url.starts_with("javascript:") || url.starts_with("#") || url.is_empty() {
                        continue;
                    }

                    // Extract title from h2 > a
                    let title = match extract_text_from_element(&element, &["h2 a", "a.result__a"]) {
                        Some(t) => t,
                        None => continue,
                    };

                    // Extract snippet from a.result__snippet
                    let snippet = extract_text_from_element(&element, &["a.result__snippet", "div.result__snippet"])
                        .unwrap_or_default();

                    // Extract display URL from a.result__url
                    let display_url = extract_text_from_element(&element, &["a.result__url", "span"])
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