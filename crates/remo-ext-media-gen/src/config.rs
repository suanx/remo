//! Configuration for image and video generation extension.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use remo_runtime_contract::PluginConfigKey;

pub const IMAGE_SIZES: &[&str] = &[
    "1024x1024", "1792x1024", "1024x1792",
    "512x512", "768x768", "1024x768", "768x1024",
];

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MediaGenConfig {
    pub openai_api_key: Option<String>,
    pub agnes_api_key: Option<String>,
    pub agnes_base_url: String,
    pub custom_base_url: Option<String>,
    pub custom_api_key: Option<String>,
    pub default_image_provider: String,
    pub default_image_model: String,
    pub default_image_size: String,
    pub default_video_provider: String,
    pub default_video_model: String,
    pub image_quality: String,
    pub n: u32,
}

impl Default for MediaGenConfig {
    fn default() -> Self {
        Self {
            openai_api_key: None,
            agnes_api_key: None,
            agnes_base_url: "https://agnes-ai.com/v1".to_string(),
            custom_base_url: None,
            custom_api_key: None,
            default_image_provider: "openai".to_string(),
            default_image_model: "dall-e-3".to_string(),
            default_image_size: "1024x1024".to_string(),
            default_video_provider: "agnes".to_string(),
            default_video_model: "agnes-video-v2.0".to_string(),
            image_quality: "standard".to_string(),
            n: 1,
        }
    }
}

impl MediaGenConfig {
    pub fn parse_image_size(&self) -> (u32, u32) {
        let parts: Vec<&str> = self.default_image_size.split('x').collect();
        if parts.len() == 2 {
            (parts[0].parse().unwrap_or(1024), parts[1].parse().unwrap_or(1024))
        } else {
            (1024, 1024)
        }
    }
}

pub struct MediaGenConfigKey;

impl PluginConfigKey for MediaGenConfigKey {
    const KEY: &'static str = "media_gen";
    type Config = MediaGenConfig;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn default_config() {
        let cfg = MediaGenConfig::default();
        assert_eq!(cfg.default_image_model, "dall-e-3");
    }
    #[test]
    fn parse_image_size() {
        let cfg = MediaGenConfig::default();
        assert_eq!(cfg.parse_image_size(), (1024, 1024));
    }
}
