//! Agnes AI Gateway 集成扩展。
//!
//! 提供免费 AI API 平台的 OpenAI 兼容协议接入，包括文本/图像/视频模型。

pub mod config;
pub mod plugin;

pub use config::{AgnesConfig, AgnesConfigKey, AgnesModelSpec, builtin_models, AGNES_MODELS};
pub use plugin::AgnesPlugin;
