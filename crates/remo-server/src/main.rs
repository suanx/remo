//! Remo AI Agent 服务器 — Docker 入口点
//!
//! 通过环境变量配置，启动多协议 HTTP 服务器。
//! 默认使用内存存储；设置 `REMO_DATA_DIR` 后自动切换为 SQLite 持久化。

use std::path::PathBuf;
use std::sync::Arc;

use remo_runtime::AgentRuntimeBuilder;
use remo_server::app::{ServerConfig, ServerState, serve};
use remo_server::mailbox::{Mailbox, MailboxConfig};
use remo_stores::{InMemoryStore, MemoryCommitCoordinator};

// ---------------------------------------------------------------------------
// 环境变量配置
// ---------------------------------------------------------------------------

const ENV_REMO_ADDRESS: &str = "REMO_ADDRESS";
const ENV_REMO_DATA_DIR: &str = "REMO_DATA_DIR";
const ENV_REMO_LOG: &str = "REMO_LOG";
const ENV_REMO_ADMIN_TOKEN: &str = "REMO_ADMIN_API_BEARER_TOKEN";
const ENV_REMO_STATIC_DIR: &str = "REMO_STATIC_DIR";
const ENV_REMO_UPLOAD_DIR: &str = "REMO_UPLOAD_DIR";

/// 默认监听地址
const DEFAULT_ADDRESS: &str = "0.0.0.0:3000";

/// 默认日志级别
const DEFAULT_LOG: &str = "remo_server=info,remo_runtime=warn,tower_http=warn";

fn env_or_default(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

// ---------------------------------------------------------------------------
// 主入口
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // 1. 初始化日志
    init_tracing();

    // 2. 构建存储后端
    let store = Arc::new(InMemoryStore::new());
    let coordinator = MemoryCommitCoordinator::wrap(store.clone());

    // 3. 构建运行时
    let runtime = Arc::new(
        AgentRuntimeBuilder::new()
            .with_commit_coordinator(coordinator)
            .build()
            .expect("构建 AgentRuntime 失败"),
    );

    // 4. 服务器配置
    let config = ServerConfig {
        address: env_or_default(ENV_REMO_ADDRESS, DEFAULT_ADDRESS),
        static_dir: std::env::var(ENV_REMO_STATIC_DIR).ok(),
        ..ServerConfig::default()
    };

    // 5. 构建 ServerState（SQLite 或内存）
    let state = if let Some(data_dir) = data_dir_from_env() {
        tracing::info!(data_dir = %data_dir.display(), "使用 SQLite 持久化存储");
        let mailbox_path = data_dir.join("mailbox.db");
        let mailbox_store = Arc::new(
            remo_stores::SqliteMailboxStore::open(&mailbox_path)
                .expect("打开 SQLite mailbox 失败"),
        );
        let mailbox = Arc::new(Mailbox::new(
            runtime.clone(), mailbox_store, store.clone(),
            "remo-server".into(), MailboxConfig::default(),
        ));
        ServerState::new(runtime.clone(), mailbox, store, runtime.resolver_arc(), config)
    } else {
        tracing::info!("使用内存存储（设置 REMO_DATA_DIR 切换为 SQLite 持久化）");
        ServerState::new_with_local_mailbox(runtime.clone(), store, runtime.resolver_arc(), config)
    };

    tracing::info!(address = %state.server_config.address, "Remo AI Agent 服务器启动中");

    // 6. 启动服务
    serve(state).await
}

fn data_dir_from_env() -> Option<PathBuf> {
    let path_str = std::env::var(ENV_REMO_DATA_DIR).ok()?;
    if path_str.is_empty() { return None; }
    let path = PathBuf::from(path_str);
    std::fs::create_dir_all(&path).ok()?;
    Some(path)
}

fn init_tracing() {
    let log_level = env_or_default(ENV_REMO_LOG, DEFAULT_LOG);
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};
    let filter = EnvFilter::try_new(&log_level).unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG));
    tracing_subscriber::registry().with(filter).with(fmt::layer().with_target(true)).init();
    tracing::debug!(log_level = %log_level, "tracing 已初始化");
}
