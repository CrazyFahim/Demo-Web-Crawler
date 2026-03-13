use anyhow::{Result};
use clap::Parser;
use html_parser::{Dom, Node};
use std::collections::{HashSet, VecDeque};
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Starting URL for the crawler
    #[arg(short, long)]
    starting_url: String,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let args = Args::parse();

    if let Err(e) = try_main(args).await {
        log::error!("Crawler cooked: {:?}", e);
    }
}

async fn try_main(args: Args) -> Result<()> {
    // 1. Initialize State
    let mut link_queue = VecDeque::new();
    let mut already_visited = HashSet::new();
    let max_links = 1000;

    // Parse the starting string into a proper Url object
    let start_url = Url::parse(&args.starting_url)?;
    link_queue.push_back(start_url);

    // 2. The Main Crawl Loop
    while let Some(current_url) = link_queue.pop_front() {
        // Exit if we've hit our limit
        if already_visited.len() >= max_links {
            break;
        }

        // Skip if already visited (Loop Prevention)
        if already_visited.contains(&current_url) {
            continue;
        }

        log::info!("Crawling: {}", current_url);
        already_visited.insert(current_url.clone());

        // 3. Fetch and Parse
        match crawl_url(current_url.clone()).await {
            Ok(new_links) => {
                for link_str in new_links {
                    // Use the 'url' crate to handle relative links automatically
                    if let Ok(next_url) = current_url.join(&link_str) {
                        if !already_visited.contains(&next_url) {
                            link_queue.push_back(next_url);
                        }
                    }
                }
            }
            Err(e) => log::warn!("Failed to crawl {}: {}", current_url, e),
        }
    }

    log::info!("Crawl finished. Visited {} unique links.", already_visited.len());
    Ok(())
}

async fn crawl_url(url: Url) -> Result<Vec<String>> {
    let html = reqwest::get(url.as_str()).await?.text().await?;
    let dom = Dom::parse(&html)?;
    let mut links = Vec::new();

    extract_links_recursive(&dom.children, &mut links);
    Ok(links)
}

fn extract_links_recursive(nodes: &[Node], links: &mut Vec<String>) {
    for node in nodes {
        if let Node::Element(elem) = node {
            if elem.name == "a" {
                // He simplified the href retrieval using .get()
                if let Some(Some(href)) = elem.attributes.get("href") {
                    links.push(href.clone());
                }
            }
            extract_links_recursive(&elem.children, links);
        }
    }
}