use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Cell, Color, Table};
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use url::Url;

/// A concurrent website link checker and uptime monitor
#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Target URL to crawl and check links (e.g., https://books.toscrape.com)
    #[arg(short, long)]
    url: Option<String>,

    /// Path to a text file of endpoint URLs to monitor (one per line)
    #[arg(short, long)]
    file: Option<String>,

    /// Maximum number of concurrent requests
    #[arg(short, long, default_value_t = 10)]
    concurrency: usize,

    /// Optional output file path for report export (.json or .csv)
    #[arg(short, long)]
    output: Option<String>,
}

#[derive(Debug, Serialize)]
struct LinkResult {
    url: String,
    status: String,
    duration_ms: u128,
    is_ok: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    if args.url.is_none() && args.file.is_none() {
        eprintln!("❌ Please provide either --url <URL> or --file <PATH>");
        std::process::exit(1);
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;

    let mut targets = HashSet::new();

    // Mode 1: Crawl a single URL and extract its links
    if let Some(target_url) = &args.url {
        let base_url = Url::parse(target_url)?;
        println!("🔍 Crawling: {}", base_url);

        let response = client.get(base_url.clone()).send().await?;
        let body = response.text().await?;
        let extracted = extract_links(&base_url, &body);
        println!("🔗 Extracted {} unique links.", extracted.len());
        targets.extend(extracted);
    }

    // Mode 2: Load endpoints from a config file
    if let Some(file_path) = &args.file {
        let content = fs::read_to_string(file_path)?;
        let file_urls = parse_endpoints_file(&content);
        println!("📄 Loaded {} endpoints from '{}'.", file_urls.len(), file_path);
        targets.extend(file_urls);
    }

    if targets.is_empty() {
        println!("⚠️ No valid URLs found to check.");
        return Ok(());
    }

    println!(
        "⚡ Checking {} total URLs concurrently (max {} at a time)...\n",
        targets.len(),
        args.concurrency
    );

    let start_total = Instant::now();
    let mut results = check_links_concurrently(client, targets, args.concurrency).await;
    let total_elapsed = start_total.elapsed();

    // Sort results: broken links first, then alphabetically by URL
    results.sort_by(|a, b| a.is_ok.cmp(&b.is_ok).then_with(|| a.url.cmp(&b.url)));

    print_table(&results);

    let healthy = results.iter().filter(|r| r.is_ok).count();
    let broken = results.len() - healthy;

    println!(
        "\n📊 Summary: {} total | {} healthy | {} broken | Finished in {:.2?}",
        results.len(),
        healthy,
        broken,
        total_elapsed
    );

    // Export report if --output flag was provided
    if let Some(output_path) = &args.output {
        export_report(&results, output_path)?;
    }

    Ok(())
}

/// Reads lines from a config file, ignoring blank lines and '#' comments
fn parse_endpoints_file(content: &str) -> HashSet<String> {
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| Url::parse(line).ok().map(|u| u.to_string()))
        .collect()
}

/// Checks all links concurrently using Tokio tasks bounded by a Semaphore
async fn check_links_concurrently(
    client: reqwest::Client,
    links: HashSet<String>,
    max_concurrency: usize,
) -> Vec<LinkResult> {
    let semaphore = Arc::new(Semaphore::new(max_concurrency));
    let mut handles = Vec::new();

    for link in links {
        let client_clone = client.clone();
        let sem_clone = Arc::clone(&semaphore);

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            let start = Instant::now();

            match client_clone.get(&link).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    LinkResult {
                        url: link,
                        status: status.as_u16().to_string(),
                        duration_ms: start.elapsed().as_millis(),
                        is_ok: status.is_success(),
                    }
                }
                Err(e) => {
                    let reason = if e.is_timeout() {
                        "TIMEOUT".to_string()
                    } else {
                        "ERROR".to_string()
                    };
                    LinkResult {
                        url: link,
                        status: reason,
                        duration_ms: start.elapsed().as_millis(),
                        is_ok: false,
                    }
                }
            }
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
}

/// Parses HTML and returns a deduplicated set of absolute HTTP/HTTPS URLs
fn extract_links(base_url: &Url, html_content: &str) -> HashSet<String> {
    let mut links = HashSet::new();
    let document = Html::parse_document(html_content);
    let selector = Selector::parse("a[href]").unwrap();

    for element in document.select(&selector) {
        if let Some(href) = element.value().attr("href") {
            if let Ok(mut resolved_url) = base_url.join(href) {
                if resolved_url.scheme() == "http" || resolved_url.scheme() == "https" {
                    resolved_url.set_fragment(None);
                    links.insert(resolved_url.to_string());
                }
            }
        }
    }

    links
}

/// Renders a formatted, color-coded table in the terminal
fn print_table(results: &[LinkResult]) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_header(vec!["Health", "Status", "Time (ms)", "URL"]);

    for res in results {
        let health_cell = if res.is_ok {
            Cell::new("OK").fg(Color::Green)
        } else {
            Cell::new("FAIL").fg(Color::Red)
        };

        table.add_row(vec![
            health_cell,
            Cell::new(&res.status),
            Cell::new(res.duration_ms.to_string()),
            Cell::new(&res.url),
        ]);
    }

    println!("{table}");
}

/// Exports the health report to JSON or CSV based on file extension
fn export_report(
    results: &[LinkResult],
    path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if path.ends_with(".json") {
        let json = serde_json::to_string_pretty(results)?;
        fs::write(path, json)?;
        println!("💾 JSON report saved to '{}'", path);
    } else if path.ends_with(".csv") {
        let mut wtr = csv::Writer::from_path(path)?;
        for res in results {
            wtr.serialize(res)?;
        }
        wtr.flush()?;
        println!("💾 CSV report saved to '{}'", path);
    } else {
        eprintln!("⚠️ Unsupported file format for '{}'. Use .json or .csv", path);
    }

    Ok(())
}