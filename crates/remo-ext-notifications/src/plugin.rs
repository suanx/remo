//! Notification plugin: registers config schema and notification tools.

use std::sync::Arc;

use remo_runtime::plugins::{ConfigSchema, Plugin, PluginDescriptor, PluginRegistrar};
use remo_runtime::state::MutationBatch;
use remo_runtime_contract::StateError;
use remo_runtime_contract::registry_spec::AgentSpec;

use crate::config::NotificationConfigKey;
use crate::tools::{
    SendDingTalkTool, SendEmailTool, SendFeishuTool, SendSlackTool, SendTelegramTool, SendWeComTool,
};

/// Stable plugin name for the notification extension.
pub const NOTIFICATIONS_PLUGIN_NAME: &str = "notifications";

/// Tool ID for the send-email tool.
pub const SEND_EMAIL_TOOL_ID: &str = "notifications:send_email";

/// Tool ID for the send-dingtalk tool.
pub const SEND_DINGTALK_TOOL_ID: &str = "notifications:send_dingtalk";

/// Tool ID for the send-wecom tool.
pub const SEND_WECOM_TOOL_ID: &str = "notifications:send_wecom";

/// Tool ID for the send-feishu tool.
pub const SEND_FEISHU_TOOL_ID: &str = "notifications:send_feishu";

/// Tool ID for the send-slack tool.
pub const SEND_SLACK_TOOL_ID: &str = "notifications:send_slack";

/// Tool ID for the send-telegram tool.
pub const SEND_TELEGRAM_TOOL_ID: &str = "notifications:send_telegram";

/// Multi-channel notification plugin.
///
/// Provides tools for sending notifications through email (SMTP), DingTalk
/// webhook, WeCom (企业微信) webhook, Feishu (飞书) webhook, Slack webhook,
/// and Telegram Bot API.
///
/// # Registered Components
///
/// - **Tool**: [`SendEmailTool`] — sends emails via configured SMTP.
/// - **Tool**: [`SendDingTalkTool`] — sends messages to DingTalk robot.
/// - **Tool**: [`SendWeComTool`] — sends messages to WeCom robot.
/// - **Tool**: [`SendFeishuTool`] — sends messages to Feishu robot.
/// - **Tool**: [`SendSlackTool`] — sends messages to Slack webhook.
/// - **Tool**: [`SendTelegramTool`] — sends messages via Telegram Bot API.
///
/// # Configuration
///
/// All channel credentials live under `sections.notifications` in the agent
/// spec (see [`NotificationConfig`](crate::config::NotificationConfig)).
pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn descriptor(&self) -> PluginDescriptor {
        PluginDescriptor {
            name: NOTIFICATIONS_PLUGIN_NAME,
        }
    }

    fn register(&self, registrar: &mut PluginRegistrar) -> Result<(), StateError> {
        registrar.register_tool(
            SEND_EMAIL_TOOL_ID,
            Arc::new(SendEmailTool),
        )?;

        registrar.register_tool(
            SEND_DINGTALK_TOOL_ID,
            Arc::new(SendDingTalkTool),
        )?;

        registrar.register_tool(
            SEND_WECOM_TOOL_ID,
            Arc::new(SendWeComTool),
        )?;

        registrar.register_tool(
            SEND_FEISHU_TOOL_ID,
            Arc::new(SendFeishuTool),
        )?;

        registrar.register_tool(
            SEND_SLACK_TOOL_ID,
            Arc::new(SendSlackTool),
        )?;

        registrar.register_tool(
            SEND_TELEGRAM_TOOL_ID,
            Arc::new(SendTelegramTool),
        )?;

        Ok(())
    }

    fn config_schemas(&self) -> Vec<ConfigSchema> {
        vec![
            ConfigSchema::for_key::<NotificationConfigKey>()
                .with_display_name("Notifications")
                .with_description(
                    "Multi-channel notification configuration for email (SMTP), \
                     DingTalk, WeCom (企业微信), Feishu (飞书), Slack, and Telegram.",
                )
                .with_category("notifications")
                .with_editor("notifications"),
        ]
    }

    fn on_activate(
        &self,
        _agent_spec: &AgentSpec,
        _patch: &mut MutationBatch,
    ) -> Result<(), StateError> {
        // Validate the config at activation time — if the section exists but
        // fails to deserialize, the agent will refuse to activate.
        let _config = _agent_spec.config::<NotificationConfigKey>()?;
        Ok(())
    }
}
