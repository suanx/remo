# ============================================================================
# Remo AI Agent Server — Dockerfile
# 多阶段构建：前端构建 → Rust 编译 → 运行阶段
# ============================================================================

# ── 前端构建阶段 ──────────────────────────────────────────────────────────
FROM node:20-slim AS frontend-builder

WORKDIR /app/frontend

# 复制依赖清单并安装（利用 Docker 缓存层）
COPY packages/awaken-app/package.json ./
RUN npm install

# 复制前端源码并构建
COPY packages/awaken-app/ .
RUN npm run build
# 构建产物输出到 /app/frontend/dist/

# ── Rust 构建阶段 ─────────────────────────────────────────────────────────
FROM rust:1.86-slim-bookworm AS builder

# 编译期系统依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 复制工作区清单（Docker 缓存层：清单不变则重装依赖跳过）
COPY Cargo.toml Cargo.lock ./
COPY crates/remo/Cargo.toml crates/remo/
COPY crates/remo-agent/Cargo.toml crates/remo-agent/
COPY crates/remo-contract/Cargo.toml crates/remo-contract/
COPY crates/remo-runtime-contract/Cargo.toml crates/remo-runtime-contract/
COPY crates/remo-server-contract/Cargo.toml crates/remo-server-contract/
COPY crates/remo-protocol-a2a/Cargo.toml crates/remo-protocol-a2a/
COPY crates/remo-stores/Cargo.toml crates/remo-stores/
COPY crates/remo-runtime/Cargo.toml crates/remo-runtime/
COPY crates/remo-server/Cargo.toml crates/remo-server/
COPY crates/remo-ext-permission/Cargo.toml crates/remo-ext-permission/
COPY crates/remo-ext-observability/Cargo.toml crates/remo-ext-observability/
COPY crates/remo-ext-mcp/Cargo.toml crates/remo-ext-mcp/
COPY crates/remo-ext-skills/Cargo.toml crates/remo-ext-skills/
COPY crates/remo-ext-reminder/Cargo.toml crates/remo-ext-reminder/
COPY crates/remo-ext-generative-ui/Cargo.toml crates/remo-ext-generative-ui/
COPY crates/remo-ext-deferred-tools/Cargo.toml crates/remo-ext-deferred-tools/
COPY crates/remo-ext-sandbox/Cargo.toml crates/remo-ext-sandbox/
COPY crates/remo-ext-rag/Cargo.toml crates/remo-ext-rag/
COPY crates/remo-ext-workflow/Cargo.toml crates/remo-ext-workflow/
COPY crates/remo-ext-memory/Cargo.toml crates/remo-ext-memory/
COPY crates/remo-ext-multimodal/Cargo.toml crates/remo-ext-multimodal/
COPY crates/remo-ext-playground/Cargo.toml crates/remo-ext-playground/
COPY crates/remo-ext-search/Cargo.toml crates/remo-ext-search/
COPY crates/remo-ext-evaluator/Cargo.toml crates/remo-ext-evaluator/
COPY crates/remo-ext-notifications/Cargo.toml crates/remo-ext-notifications/
COPY crates/remo-ext-voice/Cargo.toml crates/remo-ext-voice/
COPY crates/remo-ext-opencode/Cargo.toml crates/remo-ext-opencode/
COPY crates/remo-ext-xfyun/Cargo.toml crates/remo-ext-xfyun/
COPY crates/remo-ext-agnes/Cargo.toml crates/remo-ext-agnes/
COPY crates/remo-ext-media-gen/Cargo.toml crates/remo-ext-media-gen/
COPY crates/remo-tool-pattern/Cargo.toml crates/remo-tool-pattern/
COPY crates/remo-doctest/Cargo.toml crates/remo-doctest/
COPY crates/remo-eval/Cargo.toml crates/remo-eval/
# 创建伪 src 入口，使 cargo 能解析工作区并编译依赖
RUN mkdir -p crates/remo-server/src && \
    echo "fn main() {}" > crates/remo-server/src/main.rs && \
    mkdir -p \
        crates/remo/src \
        crates/remo-agent/src \
        crates/remo-contract/src \
        crates/remo-runtime-contract/src \
        crates/remo-server-contract/src \
        crates/remo-protocol-a2a/src \
        crates/remo-stores/src \
        crates/remo-runtime/src \
        crates/remo-ext-permission/src \
        crates/remo-ext-observability/src \
        crates/remo-ext-mcp/src \
        crates/remo-ext-skills/src \
        crates/remo-ext-reminder/src \
        crates/remo-ext-generative-ui/src \
        crates/remo-ext-deferred-tools/src \
        crates/remo-ext-sandbox/src \
        crates/remo-ext-rag/src \
        crates/remo-ext-workflow/src \
        crates/remo-ext-memory/src \
        crates/remo-ext-multimodal/src \
        crates/remo-ext-playground/src \
        crates/remo-ext-search/src \
        crates/remo-ext-evaluator/src \
        crates/remo-ext-notifications/src \
        crates/remo-ext-voice/src \
        crates/remo-ext-opencode/src \
        crates/remo-ext-xfyun/src \
        crates/remo-ext-agnes/src \
        crates/remo-ext-media-gen/src \
        crates/remo-tool-pattern/src \
        crates/remo-doctest/src \
        crates/remo-eval/src && \
    touch crates/remo/src/lib.rs \
          crates/remo-agent/src/lib.rs \
          crates/remo-contract/src/lib.rs \
          crates/remo-runtime-contract/src/lib.rs \
          crates/remo-server-contract/src/lib.rs \
          crates/remo-protocol-a2a/src/lib.rs \
          crates/remo-stores/src/lib.rs \
          crates/remo-runtime/src/lib.rs \
          crates/remo-ext-permission/src/lib.rs \
          crates/remo-ext-observability/src/lib.rs \
          crates/remo-ext-mcp/src/lib.rs \
          crates/remo-ext-skills/src/lib.rs \
          crates/remo-ext-reminder/src/lib.rs \
          crates/remo-ext-generative-ui/src/lib.rs \
          crates/remo-ext-deferred-tools/src/lib.rs \
          crates/remo-ext-sandbox/src/lib.rs \
          crates/remo-ext-rag/src/lib.rs \
          crates/remo-ext-workflow/src/lib.rs \
          crates/remo-ext-memory/src/lib.rs \
          crates/remo-ext-multimodal/src/lib.rs \
          crates/remo-ext-playground/src/lib.rs \
          crates/remo-ext-search/src/lib.rs \
          crates/remo-ext-evaluator/src/lib.rs \
          crates/remo-ext-notifications/src/lib.rs \
          crates/remo-ext-voice/src/lib.rs \
          crates/remo-ext-opencode/src/lib.rs \
          crates/remo-ext-xfyun/src/lib.rs \
          crates/remo-ext-agnes/src/lib.rs \
          crates/remo-ext-media-gen/src/lib.rs \
          crates/remo-tool-pattern/src/lib.rs \
          crates/remo-doctest/src/lib.rs \
          crates/remo-eval/src/lib.rs
# 编译所有依赖（生成可缓存中间产物）
RUN cargo fetch --locked && \
    cargo build --package=remo-server --release --locked

# 复制完整源码（覆盖伪 src）
COPY . .

# 触达真实源码后重新编译（仅项目代码改变，依赖已缓存）
RUN cargo build --package=remo-server --release --locked

# ── 运行阶段 ──────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

# 运行时系统依赖
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    wget \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN groupadd --gid 10001 remo && \
    useradd --uid 10001 --gid remo --shell /sbin/nologin --create-home remo

WORKDIR /app
COPY --from=builder /app/target/release/remo-server /app/remo-server
COPY --from=frontend-builder /app/frontend/dist /app/static

ENV REMO_STATIC_DIR=/app/static

USER remo

EXPOSE 3000

# 健康检查：请求 /health 端点（axum 内置 200 OK）
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://127.0.0.1:3000/health || exit 1

ENTRYPOINT ["/app/remo-server"]
