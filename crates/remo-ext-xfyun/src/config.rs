//! Configuration for 讯飞星辰 MaaS 平台 (Xunfei Xinghuo MaaS).
//!
//! 星火大模型推理服务，使用 OpenAI 兼容 API 协议。

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use remo_runtime_contract::PluginConfigKey;

/// 讯飞星辰各区域 API Base URL
pub const XFYUN_BASE_URLS: &[(&str, &str)] = &[
    ("华北-北京", "https://maas-api.cn-huabei-1.xf-yun.com/v1"),
    ("华东-上海", "https://maas-api.cn-east-3.xf-yun.com/v1"),
    ("华南-广州", "https://maas-api.cn-south-1.xf-yun.com/v1"),
];

/// 默认区域
pub const DEFAULT_REGION: &str = "华北-北京";

/// 默认模型
pub const DEFAULT_MODEL: &str = "qwen3.5-2b";

/// 讯飞星辰 MaaS 平台配置
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct XfyunConfig {
    /// API Key (从讯飞开放平台获取)
    pub api_key: Option<String>,
    /// API Secret (部分接口需要)
    pub api_secret: Option<String>,
    /// 服务区域
    pub region: String,
    /// 自定义 Base URL (覆盖 region 设置)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_base_url: Option<String>,
    /// 使用的模型 ID
    pub model: String,
    /// 温度参数 (0.0 - 1.0)
    pub temperature: f64,
    /// 最大输出 Token 数
    pub max_tokens: u32,
    /// 是否启用流式输出
    pub stream: bool,
    /// Embedding 模型 ID（如 sde0a5839）
    pub embedding_model: String,
    /// Rerank 模型 ID（如 s125c8e0e）
    pub rerank_model: String,
    /// 应用 app_id（从开放平台控制台获取，图片生成必填）
    pub app_id: Option<String>,
    /// 图片生成 TTI 端点
    pub tti_endpoint: String,
    /// 图片生成模型 ID（domain 参数）
    pub tti_model: String,
    /// 图片默认宽度
    pub tti_width: u32,
    /// 图片默认高度
    pub tti_height: u32,
    /// 图片生成步数 (1-50)
    pub tti_steps: u32,
    /// 提示词相关度 (0-20)
    pub tti_guidance_scale: f64,
    /// 随机种子
    pub tti_seed: u32,
    /// 调度器
    pub tti_scheduler: String,
}
impl Default for XfyunConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_secret: None,
            region: DEFAULT_REGION.to_string(),
            custom_base_url: None,
            model: DEFAULT_MODEL.to_string(),
            temperature: 0.5,
            max_tokens: 4096,
            stream: false,
            embedding_model: "sde0a5839".to_string(),
            rerank_model: "s125c8e0e".to_string(),
            app_id: None,
            tti_endpoint: "https://maas-api.cn-huabei-1.xf-yun.com/v2.1/tti".to_string(),
            tti_model: "".to_string(),
            tti_width: 768,
            tti_height: 768,
            tti_steps: 20,
            tti_guidance_scale: 5.0,
            tti_seed: 42,
            tti_scheduler: "DPM++ 2M Karras".to_string(),
        }
    }
}

impl XfyunConfig {
    /// 获取有效的 Base URL
    pub fn effective_base_url(&self) -> String {
        if let Some(ref url) = self.custom_base_url {
            return url.trim_end_matches('/').to_string();
        }
        XFYUN_BASE_URLS
            .iter()
            .find(|(name, _)| name == &self.region)
            .map(|(_, url)| url.to_string())
            .unwrap_or_else(|| XFYUN_BASE_URLS[0].1.to_string())
    }
}

/// 模型规格描述
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct XfyunModelSpec {
    pub id: String,
    pub name: String,
}

pub struct XfyunConfigKey;

impl PluginConfigKey for XfyunConfigKey {
    const KEY: &'static str = "xfyun";
    type Config = XfyunConfig;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let cfg = XfyunConfig::default();
        assert_eq!(cfg.model, DEFAULT_MODEL);
        assert_eq!(cfg.region, "华北-北京");
        assert!(cfg.api_key.is_none());
        assert_eq!(cfg.embedding_model, "sde0a5839");
        assert_eq!(cfg.rerank_model, "s125c8e0e");
    }

    #[test]
    fn effective_base_url_default() {
        let cfg = XfyunConfig::default();
        assert!(cfg.effective_base_url().contains("xf-yun.com"));
    }

    #[test]
    fn custom_base_url_override() {
        let cfg = XfyunConfig {
            custom_base_url: Some("https://custom.example.com/v1".into()),
            ..Default::default()
        };
        assert_eq!(cfg.effective_base_url(), "https://custom.example.com/v1");
    }
}
