//! Configuration for Agnes AI Gateway — a free AI API platform.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use remo_runtime_contract::PluginConfigKey;

/// Agnes AI 默认 API Base URL
pub const AGNES_DEFAULT_BASE_URL: &str = "https://agnes-ai.com/v1";

/// Agnes AI 支持的内置模型列表 (id, name, upstream)
pub const AGNES_MODELS: &[(&str, &str)] = &[
    ("agnes-1.5-flash", "Agnes 1.5 Flash"),
    ("agnes-2.0-flash", "Agnes 2.0 Flash"),
    ("agnes-image-2.0-flash", "Agnes Image 2.0 Flash"),
    ("agnes-image-2.1-flash", "Agnes Image 2.1 Flash"),
    ("agnes-video-v2.0", "Agnes Video V2.0"),
];

/// Agnes AI Gateway 配置
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct AgnesConfig {
    /// API Key (从 agnes-ai.com 获取)
    pub api_key: Option<String>,
    /// 使用的模型 ID
    pub model: String,
    /// 自定义 Base URL
    pub base_url: String,
    /// 温度参数 (0.0 - 1.0)
    pub temperature: f64,
    /// 最大输出 Token 数
    pub max_tokens: u32,
}

impl Default for AgnesConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            model: "agnes-1.5-flash".to_string(),
            base_url: AGNES_DEFAULT_BASE_URL.to_string(),
            temperature: 0.7,
            max_tokens: 4096,
        }
    }
}

/// 模型规格描述
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgnesModelSpec {
    pub id: String,
    pub name: String,
}

/// 返回内置模型列表
pub fn builtin_models() -> Vec<AgnesModelSpec> {
    AGNES_MODELS
        .iter()
        .map(|(id, name)| AgnesModelSpec {
            id: id.to_string(),
            name: name.to_string(),
        })
        .collect()
}

pub struct AgnesConfigKey;

impl PluginConfigKey for AgnesConfigKey {
    const KEY: &'static str = "agnes";
    type Config = AgnesConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = AgnesConfig::default();
        assert_eq!(cfg.model, "agnes-1.5-flash");
        assert!(cfg.api_key.is_none());
    }

    #[test]
    fn builtin_models_count() {
        assert_eq!(builtin_models().len(), 5);
    }
}
