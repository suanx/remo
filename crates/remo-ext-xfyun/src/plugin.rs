use std::sync::Arc;
use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;
use crate::config::XfyunConfigKey;
use crate::tools::{GetEmbeddingTool, RerankDocumentsTool, XfyunImageGenTool};

pub const XFYUN_PLUGIN_NAME: &str = "xfyun";
pub const GET_EMBEDDING_TOOL_ID: &str = "xfyun:get_embedding";
pub const RERANK_DOCUMENTS_TOOL_ID: &str = "xfyun:rerank_documents";
pub const XFYUN_IMAGE_GEN_TOOL_ID: &str = "xfyun:generate_image";

/// 讯飞星辰 MaaS 平台插件。
///
/// 提供 OpenAI 兼容的对话推理、Embedding & Rerank、图片生成（TTI）服务。
pub struct XfyunPlugin;

impl Plugin for XfyunPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: XFYUN_PLUGIN_NAME }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_tool(GET_EMBEDDING_TOOL_ID, Arc::new(GetEmbeddingTool))?;
        registrar.register_tool(RERANK_DOCUMENTS_TOOL_ID, Arc::new(RerankDocumentsTool))?;
        registrar.register_tool(XFYUN_IMAGE_GEN_TOOL_ID, Arc::new(XfyunImageGenTool))?;
        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<XfyunConfigKey>()
                .with_display_name("讯飞星辰 MaaS")
                .with_description("讯飞星火大模型推理服务（OpenAI 兼容）+ Embedding & Rerank + TTI 图片生成")
                .with_category("xfyun")
                .with_editor("xfyun"),
        ]
    }

    fn on_activate(&self, agent_spec: &AgentSpec, _patch: &mut MutationBatch) -> Result<(), StateError> {
        let config = agent_spec.config::<XfyunConfigKey>()?;
        let base_url = config.effective_base_url();
        tracing::info!(
            region = %config.region,
            base_url = %base_url,
            model = %config.model,
            has_app_id = config.app_id.is_some(),
            "讯飞星辰 MaaS 插件已激活"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor() {
        let p = XfyunPlugin;
        assert_eq!(p.descriptor().name, "xfyun");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let p = XfyunPlugin;
        assert!(!p.config_schemas().is_empty());
    }
}
