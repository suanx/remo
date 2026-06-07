//! OpenCode Zen model discovery — auto-fetch free models from the Zen API.

use crate::config::{DiscoveredModel, OpenCodeConfig, builtin_free_models};
use serde::Deserialize;

/// Response from OpenCode Zen `/v1/models` endpoint.
#[derive(Debug, Deserialize)]
struct ZenModelsResponse {
    data: Vec<ZenModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ZenModelEntry {
    id: String,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    pricing: Option<ZenPricing>,
}

#[derive(Debug, Deserialize)]
struct ZenPricing {
    #[serde(default)]
    input: Option<f64>,
    #[serde(default)]
    output: Option<f64>,
    #[serde(default)]
    cached_input: Option<f64>,
}

/// Discover models from OpenCode Zen API.
/// Falls back to built-in free models if the API is unreachable.
pub async fn discover_models(config: &OpenCodeConfig) -> Vec<DiscoveredModel> {
    // If auto-discovery is disabled, only return built-in free models
    if !config.auto_discover_free_models {
        return builtin_free_models();
    }
    
    // Try to fetch from Zen API
    let models_url = format!("{}/models", config.zen_api_url.trim_end_matches('/'));
    
    let client = reqwest::Client::new();
    let mut req = client.get(&models_url).timeout(std::time::Duration::from_secs(10));
    
    // Add API key if available (may provide more model visibility)
    if let Some(ref key) = config.zen_api_key {
        req = req.header("Authorization", format!("Bearer {}", key));
    }
    
    let zen_models: Vec<DiscoveredModel> = match req.send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<ZenModelsResponse>().await {
                Ok(zen_resp) => {
                    tracing::info!(count = zen_resp.data.len(), "discovered models from OpenCode Zen API");
                    zen_resp.data.into_iter().map(|entry| {
                        let is_free = entry.pricing.as_ref()
                            .map(|p| {
                                p.input.map(|v| v == 0.0).unwrap_or(false)
                                    && p.output.map(|v| v == 0.0).unwrap_or(false)
                            })
                            .unwrap_or(false);
                        
                        let upstream = entry.id.clone();
                        // Simplify ID: remove "opencode/" prefix if present
                        let id = entry.id.trim_start_matches("opencode/").to_string();
                        let name = entry.name.unwrap_or_else(|| id.clone());
                        
                        DiscoveredModel {
                            id,
                            name,
                            is_free,
                            provider: "opencode".to_string(),
                            upstream_model: upstream,
                        }
                    }).collect()
                }
                Err(e) => {
                    tracing::warn!(error = %e, "failed to parse Zen models response, falling back to built-in");
                    vec![]
                }
            }
        }
        Ok(resp) => {
            tracing::warn!(status = %resp.status(), "Zen models API returned non-success, falling back to built-in");
            vec![]
        }
        Err(e) => {
            tracing::warn!(error = %e, "failed to reach OpenCode Zen API, falling back to built-in free models");
            vec![]
        }
    };
    
    // Merge: prefer API results, fall back to built-in for any missing free models
    if zen_models.is_empty() {
        builtin_free_models()
    } else {
        // Keep API results filtered to free models
        let mut result: Vec<DiscoveredModel> = zen_models.into_iter().filter(|m| m.is_free).collect();
        
        // If no free models from API, use built-in
        if result.is_empty() {
            builtin_free_models()
        } else {
            result
        }
    }
}

/// Build model specs from discovered models for registration into the runtime.
pub fn discovered_models_to_specs(models: &[DiscoveredModel]) -> Vec<DiscoveredModel> {
    models.to_vec()
}
