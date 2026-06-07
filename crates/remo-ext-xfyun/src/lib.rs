//! 讯飞星辰 MaaS 平台集成扩展。
//!
//! 提供星火大模型推理服务、Embedding & Rerank、TTI 图片生成的 OpenAI 兼容 API 接入。

pub mod config;
pub mod plugin;
pub mod tools;

pub use config::{XfyunConfig, XfyunConfigKey, XfyunModelSpec, XFYUN_BASE_URLS};
pub use plugin::XfyunPlugin;
pub use tools::{GetEmbeddingTool, RerankDocumentsTool, XfyunImageGenTool};
