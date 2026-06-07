//! Configuration for image and video generation extension.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use remo_runtime_contract::PluginConfigKey;

/// Images sizes available for generation
pub const IMAGE_SIZES: &[&str] = &[
    "1024x1024", "1792x1024", "1024x1792",
    "512x512", "768x768", "1024x768", "768x1024",
];

/// Media generation configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct MediaGenConfig {
    /// OpenAI API Key
    pub openai_api_key: Option<String>,
    /// Agnes AI API Key
    pub agnes_api_key: Option<String>,
    /// Agnes AI Base URL
    pub agnes_base_url: String,
    /// Custom OpenAI compatible Base URL
    pub custom_base_url: Option<String>,
    /// Custom OpenAI compatible API Key
    pub custom_api_key: Option<String>,
    /// Default image provider
    pub default_image_provider: String,
    /// Default image model
    pub default_image_model: String,
    /// Default image size
    pub default_image_size: String,
    /// Default video provider
    pub default_video_provider: String,
    /// Default video model
    pub default_video_model: String,
    /// Image quality
    pub image_quality: String,
    /// Number of images to generate
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
            let w = parts[0].parse::<u32>().unwrap_or(1024);
            let h = parts[1].parse::<u32>().unwrap_or(1024);
            (w, h)
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
        assert_eq!(cfg.default_image_size, "1024x1024");
    }

    #[test]
    fn parse_image_size_default() {
        let cfg = MediaGenConfig::default();
        let (w, h) = cfg.parse_image_size();
        assert_eq!(w, 1024);
        assert_eq!(h, 1024);
    }
}
