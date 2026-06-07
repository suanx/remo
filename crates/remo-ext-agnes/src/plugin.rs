//! Agnes plugin — registers Agnes AI Gateway as an OpenAI-compatible provider.

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;
use crate::config::{AgnesConfigKey, builtin_models};

pub const AGNES_PLUGIN_NAME: &str = "agnes";

/// Agnes AI Gateway 插件。
///
/// 提供免费 AI API 平台的 OpenAI 兼容协议接入。
pub struct AgnesPlugin;

impl Plugin for AgnesPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor { name: AGNES_PLUGIN_NAME }
    }

    fn register(&self, _registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<AgnesConfigKey>()
                .with_display_name("Agnes AI Gateway")
                .with_description("免费 AI API 平台，OpenAI 兼容协议，支持文本/图像/视频模型")
                .with_category("agnes")
                .with_editor("agnes"),
        ]
    }

    fn on_activate(&self, agent_spec: &AgentSpec, _patch: &mut MutationBatch) -> Result<(), StateError> {
        let config = agent_spec.config::<AgnesConfigKey>()?;
        tracing::info!(
            model = %config.model,
            base_url = %config.base_url,
            "Agnes AI Gateway 插件已激活"
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plugin_descriptor() {
        let p = AgnesPlugin;
        assert_eq!(p.descriptor().name, "agnes");
    }

    #[test]
    fn plugin_has_config_schemas() {
        let p = AgnesPlugin;
        assert!(!p.config_schemas().is_empty());
    }
}
