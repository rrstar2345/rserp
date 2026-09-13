mod core;
mod engines;
mod models;
mod server;

use clap::{Parser, Subcommand};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "rserp")]
#[command(about = "Open-source SERP API in Rust", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the API server
    #[command(alias = "serve")]
    Server {
        /// Port to listen on
        #[arg(short, long, default_value = "7000")]
        port: u16,
    },

    /// Perform a search from CLI
    Search {
        /// Search query
        query: String,

        /// Engines to search (comma-separated): duckduckgo,bing
        #[arg(short, long, default_value = "duckduckgo,bing")]
        engines: String,

        /// Number of results
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output format: json, pretty
        #[arg(short, long, default_value = "pretty")]
        format: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Server { port } => {
            server::start_server(port).await?;
        }
        Commands::Search {
            query,
            engines,
            limit,
            format,
        } => {
            perform_search(&query, &engines, limit, &format).await?;
        }
    }

    Ok(())
}

async fn perform_search(
    query: &str,
    engines: &str,
    limit: usize,
    format: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let engine_list: Vec<&str> = engines.split(',').map(|e| e.trim()).collect();

    let mut all_results = Vec::new();
    let mut responded = Vec::new();
    let mut failed = Vec::new();

    for engine_name in engine_list {
        if let Some(engine) = engines::Engine::from_string(engine_name) {
            match engine.search(query, limit).await {
                Ok(results) => {
                    responded.push(engine_name.to_string());
                    all_results.extend(results);
                }
                Err(e) => {
                    failed.push(format!("{}: {}", engine_name, e));
                    eprintln!("Error with {}: {}", engine_name, e);
                }
            }
        } else {
            failed.push(format!("Unknown engine: {}", engine_name));
            eprintln!("Unknown engine: {}", engine_name);
        }
    }

    // Sort by rank per engine and limit
    all_results.sort_by_key(|r| (r.engine.clone(), r.rank));
    all_results.truncate(limit);

    match format {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&all_results)?);
        }
        "pretty" | _ => {
            println!("\n=== Search Results ===");
            println!("Query: {}", query);
            println!("Engines: {}", responded.join(", "));
            if !failed.is_empty() {
                println!("Failed: {}", failed.join(", "));
            }
            println!("\n{} results found\n", all_results.len());

            for (i, result) in all_results.iter().enumerate() {
                println!("{}. [{}] {}", i + 1, result.engine, result.title);
                println!("   URL: {}", result.url);
                if !result.snippet.is_empty() {
                    println!("   Snippet: {}", result.snippet);
                }
                println!();
            }
        }
    }

    Ok(())
}
