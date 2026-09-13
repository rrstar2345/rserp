use chrono::Utc;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Query {
    pub text: String,
    pub lang: Option<String>,
    pub region: Option<String>,
    pub limit: usize,
}

impl Default for Query {
    fn default() -> Self {
        Self {
            text: String::new(),
            lang: None,
            region: None,
            limit: 10,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryEcho {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    pub engines_requested: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub request_id: String,
    pub requested_at: String,
    pub took_ms: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub engines_responded: Vec<String>,
    pub engines_failed: Vec<String>,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub has_more: bool,
    pub next_start: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ResultType {
    Organic,
    Ad,
    FeaturedSnippet,
    AnswerBox,
    RelatedSearches,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tld: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sld: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Result {
    pub id: String,
    pub rank: u32,
    #[serde(rename = "type")]
    pub result_type: ResultType,
    pub title: String,
    pub url: String,
    pub display_url: String,
    pub snippet: String,
    pub domain: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
    pub engine: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_info: Option<DomainInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerpFeature {
    pub id: String,
    pub engine: String,
    #[serde(rename = "type")]
    pub feature_type: ResultType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub extracted_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Envelope {
    pub query: QueryEcho,
    pub meta: ResponseMeta,
    pub results: Vec<Result>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub serp_features: Vec<SerpFeature>,
    pub pagination: Pagination,
}

impl Envelope {
    pub fn new(query: &Query, request_id: String, engines: Vec<String>) -> Self {
        let now = Utc::now();
        Self {
            query: QueryEcho {
                text: query.text.clone(),
                lang: query.lang.clone(),
                region: query.region.clone(),
                engines_requested: engines,
            },
            meta: ResponseMeta {
                request_id,
                requested_at: now.to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
                took_ms: 0,
                engines_responded: Vec::new(),
                engines_failed: Vec::new(),
                version: "2.1".to_string(),
            },
            results: Vec::new(),
            serp_features: Vec::new(),
            pagination: Pagination {
                page: 1,
                has_more: false,
                next_start: query.limit as u32,
            },
        }
    }

    pub fn finalize(&mut self, started_at: std::time::Instant) {
        self.meta.took_ms = started_at.elapsed().as_millis() as u64;
        let limit = self.query.engines_requested.len() as u32;
        self.pagination.has_more = self.results.len() as u32 >= limit;
    }
}
