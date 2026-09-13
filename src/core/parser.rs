use scraper::{Html, Selector};

/// Detects if HTML contains captcha challenge using selectors and text markers
pub fn detect_captcha(html: &str, captcha_selectors: &[&str], captcha_markers: &[&str]) -> bool {
    // First check selectors (cheaper than full HTML walk)
    if has_selector_match(html, captcha_selectors) {
        return true;
    }

    // Then check text markers (case-insensitive)
    let html_lower = html.to_lowercase();
    for marker in captcha_markers {
        if html_lower.contains(&marker.to_lowercase()) {
            return true;
        }
    }

    false
}

/// Detects if HTML indicates no results using selectors and text markers
pub fn detect_empty_results(html: &str, empty_selectors: &[&str], empty_markers: &[&str]) -> bool {
    tracing::debug!("detect_empty_results: Checking {} selectors and {} markers", empty_selectors.len(), empty_markers.len());
    
    // First check selectors (more reliable than text markers)
    tracing::debug!("detect_empty_results: Checking selectors");
    if has_selector_match(html, empty_selectors) {
        tracing::debug!("Empty results detected via selector match");
        return true;
    }
    tracing::debug!("detect_empty_results: No selector match found");

    // Only check text markers within content areas to avoid false positives
    // from navigation, headers, or other UI elements
    let document = Html::parse_document(html);
    
    // Look for text markers only in main content areas
    let content_selectors = &[
        "main",
        "article",
        "div[role='main']",
        ".content",
        ".search-results",
        "#content",
    ];
    
    tracing::debug!("detect_empty_results: Checking text markers in content areas");
    for content_selector_str in content_selectors {
        if let Ok(content_selector) = Selector::parse(content_selector_str) {
            if let Some(content) = document.select(&content_selector).next() {
                let content_text = content.inner_html().to_lowercase();
                tracing::debug!("detect_empty_results: Found content area: {}", content_selector_str);
                for marker in empty_markers {
                    if content_text.contains(&marker.to_lowercase()) {
                        tracing::debug!("Empty results detected via marker: {}", marker);
                        return true;
                    }
                }
            }
        }
    }

    tracing::debug!("detect_empty_results: No empty results markers detected");
    false
}

#[allow(dead_code)]
pub fn extract_text(html: &str, selectors: &[&str]) -> Option<String> {
    let document = Html::parse_document(html);

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                let text = element.inner_html();
                let normalized = normalize_whitespace(&text);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    None
}

pub fn extract_text_from_element(
    element: &scraper::element_ref::ElementRef,
    selectors: &[&str],
) -> Option<String> {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = element.select(&selector).next() {
                let text = el.inner_html();
                let normalized = normalize_whitespace(&text);
                if !normalized.is_empty() {
                    return Some(normalized);
                }
            }
        }
    }

    None
}

pub fn extract_attr(
    element: &scraper::element_ref::ElementRef,
    selectors: &[&str],
    attr: &str,
) -> Option<String> {
    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if let Some(el) = element.select(&selector).next() {
                if let Some(value) = el.value().attr(attr) {
                    let trimmed = value.trim();
                    if !trimmed.is_empty() {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }
    }

    None
}

pub fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}

#[allow(dead_code)]
pub fn extract_domain(url: &str) -> String {
    if let Ok(parsed) = url::Url::parse(url) {
        if let Some(domain) = parsed.domain() {
            return domain.to_string();
        }
    }
    String::new()
}

pub fn has_selector_match(html: &str, selectors: &[&str]) -> bool {
    let document = Html::parse_document(html);

    for selector_str in selectors {
        if let Ok(selector) = Selector::parse(selector_str) {
            if document.select(&selector).next().is_some() {
                return true;
            }
        }
    }

    false
}