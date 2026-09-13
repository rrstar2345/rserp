use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum SearchError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(String),

    #[error("HTML parsing failed: {0}")]
    ParseError(String),

    #[error("No results found")]
    EmptyResults,

    #[error("Captcha detected")]
    CaptchaDetected,

    #[error("Server returned an error: {0}")]
    ServerError(String),

    #[error("Timeout")]
    Timeout,

    #[error("Invalid query: {0}")]
    InvalidQuery(String),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

impl SearchError {
    #[allow(dead_code)]
    pub fn classification(&self) -> String {
        match self {
            SearchError::CaptchaDetected => "captcha".to_string(),
            SearchError::EmptyResults => "no_results".to_string(),
            SearchError::Timeout => "timeout".to_string(),
            _ => "error".to_string(),
        }
    }
}
