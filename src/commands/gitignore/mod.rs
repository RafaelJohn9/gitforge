use crate::commands::base::Runnable;

use crate::utils::cache::{Cache, CacheManager};
use crate::utils::progress;
use crate::utils::remote::Fetcher;

use clap::Subcommand;

pub mod add;
pub mod list;
pub mod preview;

// Global constants - these can stay in the main module file
const GITHUB_API_BASE: &str = "https://api.github.com/repos/github/gitignore";
const GITHUB_RAW_BASE: &str = "https://raw.githubusercontent.com/github/gitignore/main";
const OUTPUT_BASE_PATH: &str = ".";
const OUTPUT: &str = "gitignore_templates";
const GITIGNORE_CACHE_NAME: &str = "gitignore_templates";
const CACHE_MAX_AGE_SECONDS: u64 = 60 * 60 * 24 * 30; // 30 days

#[derive(Subcommand)]
pub enum Command {
    Add(add::AddArgs),
    List(list::ListArgs),
    Preview(preview::PreviewArgs),
}

impl Command {
    pub fn execute(&self) -> anyhow::Result<()> {
        match self {
            Command::Add(args) => args.run(),
            Command::List(args) => args.run(),
            Command::Preview(args) => args.run(),
        }
    }
}

fn find_template_in_cache<'a>(
    template_name: &str,
    cache: &'a Cache<String>,
) -> Result<&'a str, anyhow::Error> {
    let normalized_template = template_name.to_lowercase();

    // Common key variants to try during lookup
    let possible_keys = vec![
        normalized_template.clone(),           // exact match
        normalized_template.replace('/', "-"), // global/windows -> global-windows
    ];

    // Try direct matches first
    for key in &possible_keys {
        if let Some(entry) = cache.entries.get(key) {
            return Ok(&entry.data);
        }
    }

    // Fallback: if the template is "global/windows", try "windows"
    if let Some(last_part) = normalized_template.split('/').last() {
        if let Some(entry) = cache.entries.get(last_part) {
            return Ok(&entry.data);
        }
    }

    // Fallback: full scan with normalization
    for (cache_key, entry) in &cache.entries {
        let key_lower = cache_key.to_lowercase();

        if key_lower == normalized_template {
            return Ok(&entry.data);
        }

        // Try with dash replacement
        let dash_normalized = normalized_template.replace('/', "-");
        if key_lower == dash_normalized {
            return Ok(&entry.data);
        }

        if key_lower.ends_with(&normalized_template) {
            return Ok(&entry.data);
        }

        if normalized_template.ends_with(&key_lower) {
            return Ok(&entry.data);
        }
    }

    Err(anyhow::anyhow!(
        "Template '{}' not found in cache. Try `gitforge gitignore list` to view available templates.",
        template_name
    ))
}

/// Ensures the gitignore cache exists and is up-to-date
fn ensure_gitignore_cache(
    cache_manager: &mut CacheManager,
    update_cache: bool,
) -> Result<Cache<String>, anyhow::Error> {
    // Only print if we are updating the cache
    let should_update =
        cache_manager.should_update_cache::<String>(GITIGNORE_CACHE_NAME, CACHE_MAX_AGE_SECONDS)?;

    if !should_update && !update_cache {
        let cache = cache_manager.load_cache(GITIGNORE_CACHE_NAME)?;
        // Only print if running in verbose/debug mode (not implemented here)
        // e.g., println!("Loaded gitignore template cache ({} templates)", cache.entries.len());
        return Ok(cache);
    }

    let pb = progress::spinner("Updating gitignore template cache...");

    let fetcher = Fetcher::new();
    let folders = vec![
        ("", ""), // root
        ("Global", "Global/"),
        ("community", "community/"),
    ];

    let mut cache = Cache::new();

    for (folder, prefix) in folders {
        let url = if folder.is_empty() {
            format!("{}/contents", GITHUB_API_BASE)
        } else {
            format!("{}/contents/{}", GITHUB_API_BASE, folder)
        };

        let entries = fetcher.fetch_json(&url)?;
        if let Some(array) = entries.as_array() {
            for entry in array {
                if let Some(name) = entry.get("name").and_then(|n| n.as_str()) {
                    if name.ends_with(".gitignore") {
                        let template_name = &name[..name.len() - ".gitignore".len()];

                        // Create the full path for fetching
                        let full_path = if prefix.is_empty() {
                            name.to_string()
                        } else {
                            format!("{}{}", prefix, name)
                        };

                        // Store with a single, consistent key format
                        let cache_key = if folder.is_empty() {
                            template_name.to_lowercase()
                        } else {
                            format!("{}-{}", folder.to_lowercase(), template_name.to_lowercase())
                        };

                        cache.insert(cache_key, full_path);
                    }
                }
            }
        }
    }
    pb.finish_and_clear();
    println!(
        "Gitignore template cache updated ({} templates available).",
        cache.entries.len()
    );

    cache_manager.save_cache(GITIGNORE_CACHE_NAME, &cache)?;
    Ok(cache)
}
