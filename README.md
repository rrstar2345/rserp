# RSERP

Open-source SERP API built in Rust.

## Current Features

-  DuckDuckGo search engine
-  Bing search engine
-  HTTP API server with `/search` endpoint
-  CLI for direct searches
-  JSON responses compatible with OpenSERP format
-  Error handling (captcha detection, no results, timeouts)

## Setup

### Prerequisites
- Rust 1.96+ ([Install Rust](https://rustup.rs/))

### Installation

```bash
git clone https://github.com/rrstar2345/rserp.git
cd rserp
cargo build --release
```

### Quick Start

#### API Server

Start the server on port 7000:
```bash
cargo run --release -- server --port 7000
```

Or:
```bash
./target/release/rserp server --port 7000
```

Make a search request:
```bash
# Single engine search
curl "http://127.0.0.1:7000/duckduckgo/search?text=rust%20programming&limit=5"

# Multi-engine search
curl "http://127.0.0.1:7000/search?text=rust%20programming&engines=duckduckgo,bing&limit=10"

# Health check
curl "http://127.0.0.1:7000/health"
```

#### CLI Usage

```bash
# Search with default engines (DuckDuckGo + Bing)
cargo run -- search "rust programming"

# Specify engines
cargo run -- search "rust programming" --engines duckduckgo --limit 5

# JSON output
cargo run -- search "rust programming" --format json
```

## API Endpoints

### Search
```
GET /search?text=<query>&engines=<engine1,engine2>&limit=<num>&lang=<lang>&region=<region>
```

**Parameters:**
- `text` (required): Search query
- `engines` (optional, default: `duckduckgo,bing`): Comma-separated engine list
- `limit` (optional, default: 10): Number of results
- `lang` (optional): Language code (future use)
- `region` (optional): Region code (future use)

**Response:**
```json
{
  "query": {
    "text": "rust programming",
    "engines_requested": ["duckduckgo", "bing"]
  },
  "meta": {
    "request_id": "uuid",
    "requested_at": "2024-01-15T10:30:00Z",
    "took_ms": 1500,
    "engines_responded": ["duckduckgo"],
    "engines_failed": ["bing"],
    "version": "2.1"
  },
  "results": [
    {
      "id": "uuid",
      "rank": 1,
      "type": "organic",
      "title": "Rust Programming Language",
      "url": "https://www.rust-lang.org/",
      "display_url": "rust-lang.org",
      "snippet": "Empowering everyone to build reliable and efficient software.",
      "domain": "rust-lang.org",
      "engine": "duckduckgo"
    }
  ],
  "serp_features": [],
  "pagination": {
    "page": 1,
    "has_more": true,
    "next_start": 10
  }
}
```

### Individual Engine Endpoints
```
GET /duckduckgo/search?text=<query>&limit=<num>
GET /bing/search?text=<query>&limit=<num>
```

### Health Check
```
GET /health
```

## Project Structure

```
rserp/
└──  src/
      └──  main.rs              # CLI entry point
      └──  server.rs            # Axum HTTP server
      └──  models.rs            # Request/response types
      └──  engines/
            └──  mod.rs         # Engine router
            └──  duckduckgo.rs  # DuckDuckGo implementation
            └──  bing.rs        # Bing implementation
      └── core/
          └──  mod.rs           # Core exports
          └──  http.rs          # HTTP client
          └──  parser.rs        # HTML parsing utilities
          └──  errors.rs        # Error types
└──  Cargo.toml
└── README.md
```

## Architecture

### Engine Design
Each search engine follows a simple interface:

1. **HTTP Fetch**: Download the search results page using the HTTP client
2. **HTML Parse**: Use `scraper` crate to parse HTML selectors
3. **Normalize**: Convert engine-specific formats to OpenSERP Result schema
4. **Return**: Yield Vec<Result> or SearchError

### Adding a New Engine

1. Create `src/engines/newengine.rs`:
```rust
pub struct NewEngineEngine;

impl NewEngineEngine {
    pub fn name() -> &'static str {
        "newengine"
    }

    pub async fn search(query: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        let http_client = crate::core::HttpClient::new()?;
        let url = format!("https://example.com/search?q={}", urlencoding::encode(query));
        let html = http_client.fetch(&url).await?;
        Self::parse(&html, limit)
    }

    fn parse(html: &str, limit: usize) -> Result<Vec<SearchResult>, SearchError> {
        // Parse HTML and extract results
        // Return Vec<SearchResult>
    }
}
```

2. Add to `src/engines/mod.rs`:
```rust
pub mod newengine;

pub enum Engine {
    DuckDuckGo,
    Bing,
    NewEngine,  // Add here
}
```

3. Update match statements in `Engine::from_string()` and `Engine::search()`.

## Roadmap

- [ ] Google search engine
- [ ] Yandex search engine
- [ ] Baidu search engine
- [ ] Ecosia search engine
- [ ] Image search support
- [ ] Response caching
- [ ] Proxy rotation
- [ ] Browser-rendered results (Chromium via headless)
- [ ] URL extraction/content scraping
- [ ] Docker deployment
- [ ] Configuration file support
- [ ] Rate limiting & resilience

## Differences from Go Version

1. **Async/Await**: Uses `tokio` for async runtime instead of Go's goroutines
2. **Web Framework**: Uses `axum` instead of Go's `chi` router
3. **HTML Parsing**: Uses `scraper` (CSS selector-based) instead of `goquery` (jQuery-like)
4. **Simpler Now**: Initial port focuses on core search functionality
5. **No Browser Rendering Yet**: Go version uses headless Chrome; Rust version currently uses HTTP only

## Performance

Initial benchmarks (rough):
- Single search: ~1-2 seconds
- Multi-engine search: ~2-3 seconds (concurrent)
- API response time: <100ms (after initial fetch)

## Known Limitations

1. HTML selectors may need tuning as search engines change their layouts
2. No browser automation yet (headless Chrome support coming)
3. Limited proxy/rotation support
4. No cache layer yet
5. Captcha handling is basic detection only

## Debugging

Enable detailed logging:
```bash
RUST_LOG=debug cargo run -- search "query"
```

## Testing

Run tests:
```bash
cargo test
```

## Contributing

Contributions welcome! Areas to help:
- Add more search engines
- Improve HTML selectors
- Add tests
- Performance optimizations
- Browser automation integration

