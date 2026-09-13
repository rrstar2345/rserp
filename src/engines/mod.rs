pub mod bing;
pub mod duckduckgo;

use crate::core::SearchError;
use crate::models::Result as SearchResult;

#[derive(Debug, Clone, Copy)]
pub enum Engine {
    DuckDuckGo,
    Bing,
}

impl Engine {
    pub fn name(&self) -> &'static str {
        match self {
            Engine::DuckDuckGo => duckduckgo::DuckDuckGoEngine::name(),
            Engine::Bing => bing::BingEngine::name(),
        }
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        match self {
            Engine::DuckDuckGo => duckduckgo::DuckDuckGoEngine::search(query, limit).await,
            Engine::Bing => bing::BingEngine::search(query, limit).await,
        }
    }

    pub fn from_string(engine: &str) -> Option<Self> {
        match engine.to_lowercase().as_str() {
            "duckduckgo" | "ddg" => Some(Engine::DuckDuckGo),
            "bing" => Some(Engine::Bing),
            _ => None,
        }
    }
}

impl std::fmt::Display for Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
