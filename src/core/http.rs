use crate::core::errors::SearchError;
use reqwest::Client;
use std::time::Duration;

// Chrome-like user agents for different platforms
const USER_AGENTS: &[&str] = &[
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36",
];

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self, SearchError> {
        let user_agent = USER_AGENTS[std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| (d.as_secs() as usize) % USER_AGENTS.len())
            .unwrap_or(0)];

        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(Duration::from_secs(20))
            .pool_max_idle_per_host(4)
            .build()
            .map_err(|e| SearchError::RequestFailed(e.to_string()))?;

        Ok(Self { client })
    }

    pub async fn fetch(&self, url: &str) -> Result<String, SearchError> {
        tracing::debug!("HTTP fetch: {}", url);
        self.fetch_with_headers(url, &[]).await
    }

    pub async fn fetch_with_headers(
        &self,
        url: &str,
        extra_headers: &[(&str, &str)],
    ) -> Result<String, SearchError> {
        let mut request = self.client.get(url);

        // Add Chrome-like headers
        request = request
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
            .header("Accept-Language", "en-US,en;q=0.9")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Sec-Fetch-Dest", "document")
            .header("Sec-Fetch-Mode", "navigate")
            .header("Sec-Fetch-Site", "none")
            .header("Sec-Fetch-User", "?1")
            .header("Cache-Control", "max-age=0");

        // Add extra headers if provided
        for (key, value) in extra_headers {
            request = request.header(*key, *value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| SearchError::RequestFailed(e.to_string()))?;

        let status = response.status();
        tracing::debug!("HTTP response status: {} for URL: {}", status, url);
        
        // Log response headers
        tracing::debug!("Response headers:");
        for (name, value) in response.headers() {
            if let Ok(val_str) = value.to_str() {
                tracing::debug!("  {}: {}", name, val_str);
            }
        }

        // Detect captcha/rate limiting
        if status.as_u16() == 429 || status.as_u16() == 403 {
            tracing::debug!("Rate limiting/captcha detected: HTTP {}", status);
            return Err(SearchError::CaptchaDetected);
        }

        if !status.is_success() {
            tracing::debug!("HTTP error response: {}", status);
            return Err(SearchError::ServerError(format!(
                "HTTP {} - {}",
                status,
                status.canonical_reason().unwrap_or("Unknown")
            )));
        }

        // reqwest automatically handles decompression (gzip, deflate, brotli)
        let text = response
            .text()
            .await
            .map_err(|e| SearchError::RequestFailed(e.to_string()))?;
        
        tracing::debug!("Successfully decoded response, length: {}", text.len());
        
        // Save to file for debugging (only first time or on error)
        if text.len() < 20000 {
            let _ = std::fs::write("/tmp/ddg_response.html", &text);
            tracing::debug!("Saved response to /tmp/ddg_response.html");
        }
        
        Ok(text)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create HTTP client")
    }
}