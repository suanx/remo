//! Multi-channel notification plugin for the Remo AI Agent framework.
//!
//! Supports sending notifications through email (SMTP), DingTalk webhook,
//! WeCom (企业微信) webhook, Feishu (飞书) webhook, Slack webhook,
//! and Telegram Bot API.

pub mod config;
pub mod plugin;
pub mod tools;

pub use config::{EmailConfig, NotificationConfig, NotificationConfigKey, TelegramConfig, WebhookConfig};
pub use plugin::NotificationPlugin;
pub use tools::{
    SendDingTalkTool, SendEmailTool, SendFeishuTool, SendSlackTool, SendTelegramTool, SendWeComTool,
};
