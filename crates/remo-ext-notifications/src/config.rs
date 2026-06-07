//! Configuration types for the multi-channel notification extension.
//!
//! Defines structs for email (SMTP), DingTalk, WeCom (企业微信), Feishu (飞书),
//! Slack, and Telegram webhook/bot configurations, along with the
//! [`NotificationConfigKey`] that binds the `"notifications"` section in
//! `AgentSpec.sections`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use remo_runtime_contract::{PluginConfigKey, RedactedString};

// ---------------------------------------------------------------------------
// EmailConfig
// ---------------------------------------------------------------------------

/// SMTP email configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EmailConfig {
    /// SMTP server hostname (e.g. `smtp.gmail.com`).
    pub smtp_host: String,

    /// SMTP server port (default: 587).
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,

    /// SMTP authentication username.
    pub username: String,

    /// SMTP authentication password.
    pub password: RedactedString,

    /// From address for outgoing emails.
    pub from_address: String,
}

fn default_smtp_port() -> u16 {
    587
}

// ---------------------------------------------------------------------------
// WebhookConfig
// ---------------------------------------------------------------------------

/// Generic webhook configuration shared by DingTalk, WeCom, Feishu, and Slack.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebhookConfig {
    /// Webhook URL (e.g. `https://oapi.dingtalk.com/robot/send?access_token=...`).
    pub webhook_url: String,

    /// Optional signing secret for verified webhooks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub secret: Option<RedactedString>,
}

// ---------------------------------------------------------------------------
// TelegramConfig
// ---------------------------------------------------------------------------

/// Telegram Bot API configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TelegramConfig {
    /// Telegram Bot API token (e.g. `123456:ABC-DEF1234`).
    pub bot_token: RedactedString,

    /// Target chat ID to send messages to.
    pub chat_id: String,
}

// ---------------------------------------------------------------------------
// NotificationConfig
// ---------------------------------------------------------------------------

/// Combined notification configuration stored under `sections["notifications"]`.
///
/// Each channel is optional — only configured channels are available at runtime.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct NotificationConfig {
    /// SMTP email configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<EmailConfig>,

    /// DingTalk robot webhook configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dingtalk: Option<WebhookConfig>,

    /// WeCom (企业微信) robot webhook configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wecom: Option<WebhookConfig>,

    /// Feishu (飞书) robot webhook configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feishu: Option<WebhookConfig>,

    /// Slack Incoming Webhook configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slack: Option<WebhookConfig>,

    /// Telegram Bot API configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub telegram: Option<TelegramConfig>,

}

// ---------------------------------------------------------------------------
// NotificationConfigKey
// ---------------------------------------------------------------------------

/// [`PluginConfigKey`] binding for notification configuration in agent specs.
///
/// ```ignore
/// let config = spec.config::<NotificationConfigKey>();
/// ```
pub struct NotificationConfigKey;

impl PluginConfigKey for NotificationConfigKey {
    const KEY: &'static str = "notifications";
    type Config = NotificationConfig;
}
