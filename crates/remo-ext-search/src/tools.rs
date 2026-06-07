//! Typed tools for web search and webpage fetching.
//!
//! Provides [`SearchWebTool`] for querying configured search providers and
//! [`FetchWebpageTool`] for retrieving and extracting webpage content.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::json;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult,
};

use crate::config::{SearchConfig, SearchConfigKey, SearchProvider};

// ---------------------------------------------------------------------------
// SearchWebTool
// ---------------------------------------------------------------------------

/// Arguments for performing a web search.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchWebArgs {
    /// The search query string.
    pub query: String,

    /// Maximum number of results to return (overrides config default).
    #[serde(default)]
    pub max_results: Option<usize>,

    /// Optional list of domains to restrict results to (overrides config).
    #[serde(default)]
    pub include_domains: Option<Vec<String>>,
}

/// Tool that performs a web search using the configured search provider.
///
/// Currently supports Tavily, SerpAPI, Bing, and Google Custom Search.
/// The provider and API key are resolved from the agent spec configuration.
pub struct SearchWebTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for SearchWebTool {
    type Args = SearchWebArgs;

    fn tool_id(&self) -> &str {
        "search:web"
    }

    fn name(&self) -> &str {
        "Search Web"
    }

    fn description(&self) -> &str {
        "Perform a web search using the configured search provider. Supports Tavily, SerpAPI, Bing, and Google Custom Search."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: SearchConfig = ctx
            .agent_spec
            .config::<SearchConfigKey>()
            .unwrap_or_default();

        let api_key = config.api_key.expose_secret();
        if api_key.is_empty() {
            return Err(ToolError::InvalidArguments(
                "Search API key is not configured. Set 'search.api_key' in the agent spec.".into(),
            ));
        }

        let max_results = args
            .max_results
            .unwrap_or(config.max_results);

        let include_domains = args.include_domains.or(config.include_domains);

        let results = match config.provider {
            SearchProvider::Tavily => {
                search_tavily(api_key, &args.query, max_results, include_domains.as_deref()).await
            }
            SearchProvider::SerpApi => {
                search_serpapi(api_key, &args.query, max_results, include_domains.as_deref()).await
            }
            SearchProvider::Bing => {
                search_bing(api_key, &args.query, max_results, include_domains.as_deref()).await
            }
            SearchProvider::Google => {
                search_google(api_key, &args.query, max_results, include_domains.as_deref()).await
            }
        }
        .map_err(|e| ToolError::ExecutionFailed(format!("Search request failed: {e}")))?;

        let data = json!({
            "query": args.query,
            "results": results,
            "count": results.len(),
            "provider": format!("{:?}", config.provider),
        });

        let result = ToolResult::success("search:web", data);
        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// FetchWebpageTool
// ---------------------------------------------------------------------------

/// Arguments for fetching a webpage.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FetchWebpageArgs {
    /// The URL of the webpage to fetch.
    pub url: String,
}

/// Tool that fetches a webpage and returns its content as Markdown text.
///
/// Uses `reqwest` to retrieve the raw HTML, then extracts the main text
/// content and converts it to Markdown format for LLM consumption.
pub struct FetchWebpageTool;

#[async_trait]
impl remo_runtime_contract::contract::tool::TypedTool for FetchWebpageTool {
    type Args = FetchWebpageArgs;

    fn tool_id(&self) -> &str {
        "search:fetch"
    }

    fn name(&self) -> &str {
        "Fetch Webpage"
    }

    fn description(&self) -> &str {
        "Fetch a webpage and return its content as Markdown-formatted text."
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let _ = ctx; // Context available for future use (e.g. cancellation)

        let content = fetch_webpage_content(&args.url).await
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to fetch URL: {e}")))?;

        let data = json!({
            "url": args.url,
            "content": content,
        });

        let result = ToolResult::success("search:fetch", data);
        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// Provider implementations
// ---------------------------------------------------------------------------

/// Perform a search via the Tavily Search API.
async fn search_tavily(
    api_key: &str,
    query: &str,
    max_results: usize,
    _include_domains: Option<&[String]>,
) -> Result<Vec<serde_json::Value>, String> {
    let url = "https://api.tavily.com/search";
    let mut body = json!({
        "api_key": api_key,
        "query": query,
        "max_results": max_results,
    });

    if let Some(domains) = _include_domains {
        body["include_domains"] = json!(domains);
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let results = data["results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    Ok(results)
}

/// Perform a search via SerpAPI.
async fn search_serpapi(
    api_key: &str,
    query: &str,
    max_results: usize,
    _include_domains: Option<&[String]>,
) -> Result<Vec<serde_json::Value>, String> {
    let mut params = vec![
        ("engine", "google"),
        ("api_key", api_key),
        ("q", query),
        ("num", &max_results.to_string()),
    ];

    let client = reqwest::Client::new();
    let mut req = client
        .get("https://serpapi.com/search")
        .query(&params);

    if let Some(domains) = _include_domains {
        // SerpAPI supports `site:` operator in the query itself
        let site_filter = domains
            .iter()
            .map(|d| format!("site:{}", d))
            .collect::<Vec<_>>()
            .join(" OR ");
        let filtered_query = format!("{} ({})", query, site_filter);
        req = client
            .get("https://serpapi.com/search")
            .query(&[
                ("engine", "google"),
                ("api_key", api_key),
                ("q", &filtered_query),
                ("num", &max_results.to_string()),
            ]);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let organic = data["organic_results"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let results: Vec<serde_json::Value> = organic
        .into_iter()
        .take(max_results)
        .map(|r| {
            json!({
                "title": r["title"],
                "url": r["link"],
                "snippet": r["snippet"],
            })
        })
        .collect();

    Ok(results)
}

/// Perform a search via Bing Web Search API (Azure).
async fn search_bing(
    api_key: &str,
    query: &str,
    max_results: usize,
    _include_domains: Option<&[String]>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = reqwest::Client::new();
    let mut req = client
        .get("https://api.bing.microsoft.com/v7.0/search")
        .header("Ocp-Apim-Subscription-Key", api_key)
        .query(&[("q", query), ("count", &max_results.to_string())]);

    if let Some(domains) = _include_domains {
        let site_filter = domains
            .iter()
            .map(|d| format!("site:{}", d))
            .collect::<Vec<_>>()
            .join(" OR ");
        let filtered_query = format!("{} ({})", query, site_filter);
        req = client
            .get("https://api.bing.microsoft.com/v7.0/search")
            .header("Ocp-Apim-Subscription-Key", api_key)
            .query(&[("q", &filtered_query), ("count", &max_results.to_string())]);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let web_pages = data["webPages"]["value"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let results: Vec<serde_json::Value> = web_pages
        .into_iter()
        .take(max_results)
        .map(|r| {
            json!({
                "title": r["name"],
                "url": r["url"],
                "snippet": r["snippet"],
            })
        })
        .collect();

    Ok(results)
}

/// Perform a search via Google Custom Search API.
async fn search_google(
    api_key: &str,
    query: &str,
    max_results: usize,
    _include_domains: Option<&[String]>,
) -> Result<Vec<serde_json::Value>, String> {
    // Google Custom Search requires both api_key and cx (search engine ID).
    // The api_key field is expected to be formatted as "cx:api_key" or just
    // the api_key with cx passed separately. We use the simple form where
    // api_key is the actual API key and cx must be configured separately.
    //
    // For now, extract the search engine ID from the first 12 characters
    // if the key contains a colon separator, otherwise require it in config.
    let (cx, actual_key) = if let Some(pos) = api_key.find(':') {
        (&api_key[..pos], &api_key[pos + 1..])
    } else {
        // Default CX - in production this should be configured explicitly
        return Err(
            "Google Custom Search requires both 'cx' (search engine ID) and 'api_key'. \
             Format the api_key as 'cx:api_key' in the search configuration."
                .into(),
        );
    };

    let client = reqwest::Client::new();
    let mut params = vec![
        ("key", actual_key),
        ("cx", cx),
        ("q", query),
        ("num", &max_results.to_string().as_str()),
    ];

    let mut req = client
        .get("https://www.googleapis.com/customsearch/v1")
        .query(&params);

    if let Some(domains) = _include_domains {
        let site_filter = domains
            .iter()
            .map(|d| format!("site:{}", d))
            .collect::<Vec<_>>()
            .join(" OR ");
        let filtered_query = format!("{} ({})", query, site_filter);
        params[2] = ("q", &filtered_query);
        req = client
            .get("https://www.googleapis.com/customsearch/v1")
            .query(&params);
    }

    let resp = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {e}"))?;

    let items = data["items"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    let results: Vec<serde_json::Value> = items
        .into_iter()
        .take(max_results)
        .map(|r| {
            json!({
                "title": r["title"],
                "url": r["link"],
                "snippet": r["snippet"],
            })
        })
        .collect();

    Ok(results)
}

/// Fetch a webpage and extract its text content as Markdown.
async fn fetch_webpage_content(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("RemoAI/1.0")
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {} returned by {}", status, url));
    }

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    // If it's not HTML, return the raw text (limited to reasonable size)
    if !content_type.contains("text/html") {
        let text = String::from_utf8_lossy(&bytes);
        let max_len = text.len().min(50_000);
        return Ok(text[..max_len].to_string());
    }

    // Simple HTML-to-text extraction
    let html = String::from_utf8_lossy(&bytes);
    let text = strip_html_tags(&html);
    let max_len = text.len().min(50_000);
    Ok(text[..max_len].to_string())
}

/// Minimal HTML tag stripper that produces readable Markdown-like text.
fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;

    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];

        if !in_tag {
            if ch == '<' {
                in_tag = true;
                // Check for script/style tags
                let remaining: String = chars.iter().skip(i + 1).take(50).collect();
                let lower = remaining.to_lowercase();
                if lower.starts_with("script") || lower.starts_with("/script") {
                    in_script = !in_script;
                } else if lower.starts_with("style") || lower.starts_with("/style") {
                    in_style = !in_style;
                }
                // Skip the tag
                let mut tag_depth = 1;
                while i < chars.len() && tag_depth > 0 {
                    i += 1;
                    if i < chars.len() {
                        if chars[i] == '>' {
                            tag_depth -= 1;
                        } else if chars[i] == '<' {
                            tag_depth += 1;
                        }
                    }
                }
                if i < chars.len() {
                    // Add a space after block-level tag endings for readability
                    result.push(' ');
                }
                in_tag = false;
                continue;
            }
            if !in_script && !in_style {
                result.push(ch);
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    // Clean up: collapse whitespace
    let cleaned: String = result
        .chars()
        .fold(String::new(), |mut acc, c| {
            if c.is_whitespace() {
                if !acc.ends_with(' ') {
                    acc.push(' ');
                }
            } else {
                acc.push(c);
            }
            acc
        })
        .trim()
        .to_string();

    cleaned
}
