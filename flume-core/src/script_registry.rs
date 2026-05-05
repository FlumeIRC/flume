//! Script registry client — fetch, search, install, and update scripts
//! from the FlumeIRC community scripts repository.
//!
//! Registry index: https://scripts.flumeirc.io/index.json

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

const REGISTRY_URL: &str = "https://scripts.flumeirc.io/index.json";

/// The registry index — list of available community scripts.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryIndex {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub scripts: Vec<RegistryEntry>,
}

/// A single script entry in the registry.
#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub flume_min: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    pub raw_url: Option<String>,
}

/// Tracking info for an installed registry script.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InstalledScript {
    pub version: String,
    #[serde(default = "default_source")]
    pub source: String,
    #[serde(default)]
    pub installed_at: String,
}

fn default_source() -> String {
    "registry".to_string()
}

/// Fetch the registry index from scripts.flumeirc.io.
/// Uses a local cache at ~/.cache/flume/registry.json (24h TTL).
pub fn fetch_index() -> Result<RegistryIndex, String> {
    // Check cache first
    let cache_path = cache_dir().join("registry.json");
    if let Ok(meta) = std::fs::metadata(&cache_path) {
        if let Ok(modified) = meta.modified() {
            let age = modified.elapsed().unwrap_or_default();
            if age.as_secs() < 86400 {
                // Cache is fresh (< 24h)
                if let Ok(content) = std::fs::read_to_string(&cache_path) {
                    if let Ok(index) = serde_json::from_str::<RegistryIndex>(&content) {
                        return Ok(index);
                    }
                }
            }
        }
    }

    // Fetch from network
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let resp = client
        .get(REGISTRY_URL)
        .send()
        .map_err(|e| format!("Failed to fetch registry: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Registry returned HTTP {}", resp.status()));
    }

    let content = resp
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    let index: RegistryIndex = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse registry index: {}", e))?;

    // Save to cache
    let _ = std::fs::create_dir_all(cache_dir());
    let _ = std::fs::write(&cache_path, &content);

    Ok(index)
}

/// Force-refresh the registry cache.
pub fn refresh_cache() -> Result<RegistryIndex, String> {
    let cache_path = cache_dir().join("registry.json");
    let _ = std::fs::remove_file(&cache_path);
    fetch_index()
}

/// Search the registry by query string. Matches name, description, tags, and author.
pub fn search<'a>(index: &'a RegistryIndex, query: &str) -> Vec<&'a RegistryEntry> {
    let q = query.to_lowercase();
    index
        .scripts
        .iter()
        .filter(|s| {
            s.name.to_lowercase().contains(&q)
                || s.description.to_lowercase().contains(&q)
                || s.author.to_lowercase().contains(&q)
                || s.category.to_lowercase().contains(&q)
                || s.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect()
}

/// Find a script by exact name.
pub fn find<'a>(index: &'a RegistryIndex, name: &str) -> Option<&'a RegistryEntry> {
    let name_lower = name.to_lowercase();
    index
        .scripts
        .iter()
        .find(|s| s.name.to_lowercase() == name_lower)
}

/// Download a script's source code from its raw_url.
/// Returns (filename, content).
pub fn download_script(entry: &RegistryEntry) -> Result<(String, String), String> {
    let url = entry
        .raw_url
        .as_ref()
        .ok_or_else(|| format!("No download URL for script '{}'", entry.name))?;

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP error: {}", e))?;

    let resp = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to download: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("HTTP {}: {}", resp.status(), url));
    }

    let content = resp
        .text()
        .map_err(|e| format!("Failed to read: {}", e))?;

    let ext = match entry.language.as_str() {
        "python" => "py",
        _ => "lua",
    };
    let filename = format!("{}.{}", entry.name, ext);

    Ok((filename, content))
}

/// Check if a script's flume_min version is compatible with the running Flume.
pub fn is_compatible(entry: &RegistryEntry) -> bool {
    let current = env!("CARGO_PKG_VERSION");
    if entry.flume_min.is_empty() {
        return true;
    }
    version_gte(current, &entry.flume_min)
}

/// Simple semver comparison: is `a` >= `b`?
fn version_gte(a: &str, b: &str) -> bool {
    let parse = |s: &str| -> (u32, u32, u32) {
        let parts: Vec<u32> = s.split('.').filter_map(|p| p.parse().ok()).collect();
        (
            parts.first().copied().unwrap_or(0),
            parts.get(1).copied().unwrap_or(0),
            parts.get(2).copied().unwrap_or(0),
        )
    };
    parse(a) >= parse(b)
}

// ── Installed script tracking ──

fn installed_path() -> PathBuf {
    crate::config::data_dir()
        .join("scripts")
        .join("installed.toml")
}

fn cache_dir() -> PathBuf {
    let home = directories::UserDirs::new()
        .map(|d| d.home_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".cache").join("flume")
}

/// Load the installed scripts registry.
pub fn load_installed() -> HashMap<String, InstalledScript> {
    let path = installed_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return HashMap::new(),
    };
    toml::from_str(&content).unwrap_or_default()
}

/// Save the installed scripts registry.
pub fn save_installed(installed: &HashMap<String, InstalledScript>) {
    let path = installed_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(content) = toml::to_string_pretty(installed) {
        let _ = std::fs::write(&path, content);
    }
}

/// Mark a script as installed from the registry.
pub fn mark_installed(name: &str, version: &str) {
    let mut installed = load_installed();
    installed.insert(
        name.to_string(),
        InstalledScript {
            version: version.to_string(),
            source: "registry".to_string(),
            installed_at: chrono::Utc::now().to_rfc3339(),
        },
    );
    save_installed(&installed);
}

/// Remove a script from the installed registry.
pub fn unmark_installed(name: &str) {
    let mut installed = load_installed();
    installed.remove(name);
    save_installed(&installed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compare() {
        assert!(version_gte("1.2.5", "1.2.5"));
        assert!(version_gte("1.2.6", "1.2.5"));
        assert!(version_gte("1.3.0", "1.2.5"));
        assert!(version_gte("2.0.0", "1.2.5"));
        assert!(!version_gte("1.2.4", "1.2.5"));
        assert!(!version_gte("1.1.9", "1.2.5"));
    }

    #[test]
    fn search_matches() {
        let index = RegistryIndex {
            version: "1".to_string(),
            scripts: vec![
                RegistryEntry {
                    name: "weather".to_string(),
                    description: "Show weather for a city".to_string(),
                    author: "FlumeIRC".to_string(),
                    version: "1.0.0".to_string(),
                    language: "python".to_string(),
                    category: "utility".to_string(),
                    tags: vec!["weather".to_string(), "command".to_string()],
                    flume_min: "1.2.5".to_string(),
                    dependencies: vec![],
                    raw_url: None,
                },
                RegistryEntry {
                    name: "8ball".to_string(),
                    description: "Magic 8-ball oracle".to_string(),
                    author: "FlumeIRC".to_string(),
                    version: "1.0.0".to_string(),
                    language: "python".to_string(),
                    category: "fun".to_string(),
                    tags: vec!["fun".to_string()],
                    flume_min: "1.2.5".to_string(),
                    dependencies: vec![],
                    raw_url: None,
                },
            ],
        };

        assert_eq!(search(&index, "weather").len(), 1);
        assert_eq!(search(&index, "fun").len(), 1);
        assert_eq!(search(&index, "FlumeIRC").len(), 2);
        assert_eq!(search(&index, "nonexistent").len(), 0);
    }

    #[test]
    fn find_by_name() {
        let index = RegistryIndex {
            version: "1".to_string(),
            scripts: vec![RegistryEntry {
                name: "weather".to_string(),
                description: String::new(),
                author: String::new(),
                version: "1.0.0".to_string(),
                language: "python".to_string(),
                category: String::new(),
                tags: vec![],
                flume_min: String::new(),
                dependencies: vec![],
                raw_url: None,
            }],
        };

        assert!(find(&index, "weather").is_some());
        assert!(find(&index, "Weather").is_some()); // case insensitive
        assert!(find(&index, "missing").is_none());
    }
}
