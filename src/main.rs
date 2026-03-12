use anyhow::Result;
use html_parser::{Dom, Node};
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();

    // The "try_main" pattern he used to avoid deep nesting
    if let Err(e) = try_main().await {
        log::error!("Crawler failed: {:?}", e);
        process::exit(-1);
    }
}

async fn try_main() -> Result<()> {
    let start_url = "https://google.com";
    log::info!("Starting crawl on: {}", start_url);

    // Initial crawl of the seed URL
    let found_links = crawl_url(start_url).await?;
    
    for link in found_links {
        log::info!("Discovered: {}", link);
    }

    Ok(())
}

async fn crawl_url(url: &str) -> Result<Vec<String>> {
    // 1. Fetch the HTML
    let html = reqwest::get(url)
        .await?
        .text()
        .await?;

    // 2. Parse into a DOM
    let dom = Dom::parse(&html)?;
    let mut links = Vec::new();

    // 3. Extract links from the DOM tree
    // He processed children recursively by checking each node type
    extract_links_recursive(&dom.children, url, &mut links);

    Ok(links)
}

fn extract_links_recursive(nodes: &[Node], root_url: &str, links: &mut Vec<String>) {
    for node in nodes {
        match node {
            Node::Element(elem) => {
                // Check if the element is an anchor <a>
                if elem.name == "a" {
                    // Look for the "href" attribute (The fix he made after the chat hint!)
                    if let Some(href) = elem.attributes.get("href") {
                        if let Some(link_val) = href {
                            let clean_url = sanitize_url(link_val, root_url);
                            links.push(clean_url);
                        }
                    }
                }
                // Recursively check children of this element
                extract_links_recursive(&elem.children, root_url, links);
            }
            _ => {} // Ignore Text and Comment nodes for link discovery
        }
    }
}

// The utility he wrote to handle relative links (e.g., "/services" -> "google.com/services")
fn sanitize_url(url: &str, root: &str) -> String {
    if url.starts_with("http") {
        url.to_string()
    } else {
        // Strip trailing slash from root and leading slash from relative path
        let root_clean = root.trim_end_matches('/');
        let url_clean = url.trim_start_matches('/');
        format!("{}/{}", root_clean, url_clean)
    }
}