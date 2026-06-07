//! Tool implementations for sending notifications through various channels.
//!
//! Provides three [`TypedTool`] implementations:
//! - [`SendEmailTool`] — sends emails via SMTP (simplified, logs via `tracing::info`).
//! - [`SendDingTalkTool`] — sends DingTalk robot messages via webhook.
//! - [`SendWeComTool`] — sends WeCom (企业微信) robot messages via webhook.

use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use remo_runtime_contract::contract::tool::{
    ToolCallContext, ToolError, ToolOutput, ToolResult, TypedTool,
};
use remo_runtime_contract::PluginConfigKey;

use crate::config::{NotificationConfig, NotificationConfigKey};

// ---------------------------------------------------------------------------
// SendEmailTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendEmailTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendEmailArgs {
    /// Recipient email address.
    pub to: String,

    /// Email subject line.
    pub subject: String,

    /// Email body content.
    pub body: String,

    /// Whether the body is HTML (default: false, plain text).
    #[serde(default)]
    pub is_html: Option<bool>,
}

/// Tool that sends an email via the configured SMTP server.
///
/// Reads SMTP credentials from the agent's `notifications` config section.
/// Currently logs the email details via `tracing::info` as a simplified
/// implementation — actual SMTP delivery requires the `lettre` crate.
pub struct SendEmailTool;

#[async_trait]
impl TypedTool for SendEmailTool {
    type Args = SendEmailArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_email"
    }

    fn name(&self) -> &str {
        "Send Email"
    }

    fn description(&self) -> &str {
        "Send an email notification via the configured SMTP server."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        let email_cfg = config
            .email
            .as_ref()
            .ok_or_else(|| ToolError::ExecutionFailed("Email not configured in agent spec under sections.notifications.email".into()))?;

        let content_type = if args.is_html.unwrap_or(false) {
            "text/html"
        } else {
            "text/plain"
        };

        tracing::info!(
            target: "remo::notifications::email",
            smtp_host = %email_cfg.smtp_host,
            smtp_port = %email_cfg.smtp_port,
            from = %email_cfg.from_address,
            to = %args.to,
            subject = %args.subject,
            content_type = %content_type,
            "Simulated email send"
        );

        let result = ToolResult::success(
            "notifications:send_email",
            json!({
                "status": "sent",
                "to": args.to,
                "subject": args.subject,
                "content_type": content_type,
                "message": "Email logged successfully (SMTP delivery not yet implemented — see tracing output)",
            }),
        );

        Ok(result.into())
    }
}

// ---------------------------------------------------------------------------
// SendDingTalkTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendDingTalkTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendDingTalkArgs {
    /// Message content to send.
    pub message: String,

    /// Message type: `"text"` (default) or `"markdown"`.
    #[serde(default)]
    pub msg_type: Option<String>,

    /// Optional webhook URL override. Falls back to the configured webhook
    /// in `sections.notifications.dingtalk.webhook_url`.
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Tool that sends a message to a DingTalk robot webhook.
///
/// Uses the configured DingTalk webhook URL from the agent's `notifications`
/// config section, with an optional per-call override.
pub struct SendDingTalkTool;

#[async_trait]
impl TypedTool for SendDingTalkTool {
    type Args = SendDingTalkArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_dingtalk"
    }

    fn name(&self) -> &str {
        "Send DingTalk Message"
    }

    fn description(&self) -> &str {
        "Send a notification message to a DingTalk group robot."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        // Resolve webhook URL: args override -> config
        let webhook_url = match args.webhook_url {
            Some(ref url) if !url.is_empty() => url.clone(),
            _ => config
                .dingtalk
                .as_ref()
                .map(|w| w.webhook_url.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "DingTalk webhook URL not configured. Provide it in the agent spec under \
                         sections.notifications.dingtalk.webhook_url, or pass webhook_url in the arguments"
                            .into(),
                    )
                })?,
        };

        let msg_type = args.msg_type.as_deref().unwrap_or("text");

        let payload = match msg_type {
            "markdown" => json!({
                "msgtype": "markdown",
                "markdown": {
                    "title": &args.message.lines().next().unwrap_or("Notification"),
                    "text": &args.message,
                }
            }),
            _ => json!({
                "msgtype": "text",
                "text": {
                    "content": &args.message,
                }
            }),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".into());

        if status.is_success() {
            tracing::info!(
                target: "remo::notifications::dingtalk",
                webhook_url = %webhook_url,
                msg_type = %msg_type,
                "DingTalk message sent"
            );
            let result = ToolResult::success(
                "notifications:send_dingtalk",
                json!({
                    "status": "sent",
                    "msg_type": msg_type,
                    "response": body,
                }),
            );
            Ok(result.into())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "DingTalk webhook returned HTTP {status}: {body}"
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// SendWeComTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendWeComTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendWeComArgs {
    /// Message content to send.
    pub message: String,

    /// Message type: `"text"` (default) or `"markdown"`.
    #[serde(default)]
    pub msg_type: Option<String>,

    /// Optional webhook URL override. Falls back to the configured webhook
    /// in `sections.notifications.wecom.webhook_url`.
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Tool that sends a message to a WeCom (企业微信) robot webhook.
///
/// Uses the configured WeCom webhook URL from the agent's `notifications`
/// config section, with an optional per-call override.
pub struct SendWeComTool;

#[async_trait]
impl TypedTool for SendWeComTool {
    type Args = SendWeComArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_wecom"
    }

    fn name(&self) -> &str {
        "Send WeCom Message"
    }

    fn description(&self) -> &str {
        "Send a notification message to a WeCom (企业微信) group robot."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        // Resolve webhook URL: args override -> config
        let webhook_url = match args.webhook_url {
            Some(ref url) if !url.is_empty() => url.clone(),
            _ => config
                .wecom
                .as_ref()
                .map(|w| w.webhook_url.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "WeCom webhook URL not configured. Provide it in the agent spec under \
                         sections.notifications.wecom.webhook_url, or pass webhook_url in the arguments"
                            .into(),
                    )
                })?,
        };

        let msg_type = args.msg_type.as_deref().unwrap_or("text");

        let payload = match msg_type {
            "markdown" => json!({
                "msgtype": "markdown",
                "markdown": {
                    "content": &args.message,
                }
            }),
            _ => json!({
                "msgtype": "text",
                "text": {
                    "content": &args.message,
                }
            }),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".into());

        if status.is_success() {
            tracing::info!(
                target: "remo::notifications::wecom",
                webhook_url = %webhook_url,
                msg_type = %msg_type,
                "WeCom message sent"
            );
            let result = ToolResult::success(
                "notifications:send_wecom",
                json!({
                    "status": "sent",
                    "msg_type": msg_type,
                    "response": body,
                }),
            );
            Ok(result.into())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "WeCom webhook returned HTTP {status}: {body}"
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// SendFeishuTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendFeishuTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendFeishuArgs {
    /// Message content to send.
    pub message: String,

    /// Message type: `"text"` (default) or `"interactive"` (markdown card).
    #[serde(default)]
    pub msg_type: Option<String>,

    /// Card title when msg_type is `"interactive"`.
    #[serde(default)]
    pub title: Option<String>,

    /// Optional webhook URL override. Falls back to the configured webhook
    /// in `sections.notifications.feishu.webhook_url`.
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Tool that sends a message to a Feishu (飞书) group robot webhook.
///
/// Supports two message types:
/// - `"text"` — plain text message.
/// - `"interactive"` — rich markdown card with a colored header.
///
/// Uses the configured Feishu webhook URL from the agent's `notifications`
/// config section, with an optional per-call override.
pub struct SendFeishuTool;

#[async_trait]
impl TypedTool for SendFeishuTool {
    type Args = SendFeishuArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_feishu"
    }

    fn name(&self) -> &str {
        "Send Feishu Message"
    }

    fn description(&self) -> &str {
        "Send a notification message to a Feishu (飞书) group robot."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        // Resolve webhook URL: args override -> config
        let webhook_url = match args.webhook_url {
            Some(ref url) if !url.is_empty() => url.clone(),
            _ => config
                .feishu
                .as_ref()
                .map(|w| w.webhook_url.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "Feishu webhook URL not configured. Provide it in the agent spec under \
                         sections.notifications.feishu.webhook_url, or pass webhook_url in the arguments"
                            .into(),
                    )
                })?,
        };

        let msg_type = args.msg_type.as_deref().unwrap_or("text");

        let payload = match msg_type {
            "interactive" => {
                let title = args.title.as_deref().unwrap_or("Notification");
                json!({
                    "msg_type": "interactive",
                    "card": {
                        "header": {
                            "title": {
                                "tag": "plain_text",
                                "content": title,
                            },
                            "template": "blue",
                        },
                        "elements": [
                            {
                                "tag": "markdown",
                                "content": &args.message,
                            }
                        ],
                    }
                })
            }
            _ => json!({
                "msg_type": "text",
                "content": {
                    "text": &args.message,
                }
            }),
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".into());

        if status.is_success() {
            tracing::info!(
                target: "remo::notifications::feishu",
                webhook_url = %webhook_url,
                msg_type = %msg_type,
                "Feishu message sent"
            );
            let result = ToolResult::success(
                "notifications:send_feishu",
                json!({
                    "status": "sent",
                    "msg_type": msg_type,
                    "response": body,
                }),
            );
            Ok(result.into())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "Feishu webhook returned HTTP {status}: {body}"
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// SendSlackTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendSlackTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendSlackArgs {
    /// Message content to send.
    pub message: String,

    /// If `true`, wraps the message in a rich block with `mrkdwn` formatting.
    /// Defaults to `false` (simple text message).
    #[serde(default)]
    pub use_blocks: Option<bool>,

    /// Optional webhook URL override. Falls back to the configured webhook
    /// in `sections.notifications.slack.webhook_url`.
    #[serde(default)]
    pub webhook_url: Option<String>,
}

/// Tool that sends a message to a Slack channel via Incoming Webhook.
///
/// Supports two payload formats:
/// - Simple: `{ "text": "..." }`
/// - Rich blocks: `{ "blocks": [{ "type": "section", "text": { "type": "mrkdwn", "text": "..." } }] }`
///
/// Uses the configured Slack webhook URL from the agent's `notifications`
/// config section, with an optional per-call override.
pub struct SendSlackTool;

#[async_trait]
impl TypedTool for SendSlackTool {
    type Args = SendSlackArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_slack"
    }

    fn name(&self) -> &str {
        "Send Slack Message"
    }

    fn description(&self) -> &str {
        "Send a notification message to a Slack channel via Incoming Webhook."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        // Resolve webhook URL: args override -> config
        let webhook_url = match args.webhook_url {
            Some(ref url) if !url.is_empty() => url.clone(),
            _ => config
                .slack
                .as_ref()
                .map(|w| w.webhook_url.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "Slack webhook URL not configured. Provide it in the agent spec under \
                         sections.notifications.slack.webhook_url, or pass webhook_url in the arguments"
                            .into(),
                    )
                })?,
        };

        let use_blocks = args.use_blocks.unwrap_or(false);

        let payload = if use_blocks {
            json!({
                "blocks": [
                    {
                        "type": "section",
                        "text": {
                            "type": "mrkdwn",
                            "text": &args.message,
                        }
                    }
                ]
            })
        } else {
            json!({
                "text": &args.message,
            })
        };

        let client = reqwest::Client::new();
        let resp = client
            .post(&webhook_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".into());

        if status.is_success() {
            tracing::info!(
                target: "remo::notifications::slack",
                webhook_url = %webhook_url,
                use_blocks = %use_blocks,
                "Slack message sent"
            );
            let result = ToolResult::success(
                "notifications:send_slack",
                json!({
                    "status": "sent",
                    "use_blocks": use_blocks,
                    "response": body,
                }),
            );
            Ok(result.into())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "Slack webhook returned HTTP {status}: {body}"
            )))
        }
    }
}

// ---------------------------------------------------------------------------
// SendTelegramTool
// ---------------------------------------------------------------------------

/// Arguments for [`SendTelegramTool`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendTelegramArgs {
    /// Message content to send.
    pub message: String,

    /// Optional chat ID override. Falls back to the configured chat_id
    /// in `sections.notifications.telegram.chat_id`.
    #[serde(default)]
    pub chat_id: Option<String>,

    /// Parse mode: `"Markdown"` (default) or `"HTML"`.
    #[serde(default)]
    pub parse_mode: Option<String>,

    /// Optional bot token override. Falls back to the configured bot_token
    /// in `sections.notifications.telegram.bot_token`.
    #[serde(default)]
    pub bot_token: Option<String>,
}

/// Tool that sends a message via the Telegram Bot API.
///
/// Calls `POST https://api.telegram.org/bot{token}/sendMessage` with the
/// provided text, chat ID, and parse mode.
///
/// Uses the configured Telegram bot token and chat ID from the agent's
/// `notifications` config section, with optional per-call overrides.
pub struct SendTelegramTool;

#[async_trait]
impl TypedTool for SendTelegramTool {
    type Args = SendTelegramArgs;

    fn tool_id(&self) -> &str {
        "notifications:send_telegram"
    }

    fn name(&self) -> &str {
        "Send Telegram Message"
    }

    fn description(&self) -> &str {
        "Send a notification message via Telegram Bot API."
    }

    fn category(&self) -> Option<&str> {
        Some("notifications")
    }

    async fn execute(
        &self,
        args: Self::Args,
        ctx: &ToolCallContext,
    ) -> Result<ToolOutput, ToolError> {
        let config: NotificationConfig = ctx
            .agent_spec
            .config::<NotificationConfigKey>()
            .map_err(|e| ToolError::Internal(format!("Failed to read config: {e}")))?;

        // Resolve bot token: args override -> config
        let bot_token = match args.bot_token {
            Some(ref token) if !token.is_empty() => token.clone(),
            _ => config
                .telegram
                .as_ref()
                .map(|t| t.bot_token.0.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "Telegram bot token not configured. Provide it in the agent spec under \
                         sections.notifications.telegram.bot_token, or pass bot_token in the arguments"
                            .into(),
                    )
                })?,
        };

        // Resolve chat ID: args override -> config
        let chat_id = match args.chat_id {
            Some(ref id) if !id.is_empty() => id.clone(),
            _ => config
                .telegram
                .as_ref()
                .map(|t| t.chat_id.clone())
                .ok_or_else(|| {
                    ToolError::ExecutionFailed(
                        "Telegram chat_id not configured. Provide it in the agent spec under \
                         sections.notifications.telegram.chat_id, or pass chat_id in the arguments"
                            .into(),
                    )
                })?,
        };

        let parse_mode = args.parse_mode.as_deref().unwrap_or("Markdown");

        let payload = json!({
            "chat_id": &chat_id,
            "text": &args.message,
            "parse_mode": parse_mode,
        });

        let api_url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");

        let client = reqwest::Client::new();
        let resp = client
            .post(&api_url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| ToolError::ExecutionFailed(format!("HTTP request failed: {e}")))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .unwrap_or_else(|_| "<unreadable>".into());

        if status.is_success() {
            tracing::info!(
                target: "remo::notifications::telegram",
                chat_id = %chat_id,
                parse_mode = %parse_mode,
                "Telegram message sent"
            );
            let result = ToolResult::success(
                "notifications:send_telegram",
                json!({
                    "status": "sent",
                    "chat_id": &chat_id,
                    "parse_mode": parse_mode,
                    "response": body,
                }),
            );
            Ok(result.into())
        } else {
            Err(ToolError::ExecutionFailed(format!(
                "Telegram API returned HTTP {status}: {body}"
            )))
        }
    }
}
