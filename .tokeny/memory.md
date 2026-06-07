# Project Memory

## fact

- 扩展分布(21个ext)：search/evaluator/notifications/voice/opencode/xfyun/agnes/media-gen为本次新增。通知支持6通道(Email/钉钉/企微/飞书/Slack/Telegram)。xfyun支持chat/embedding/rerank/TTI图片生成。OpenCode Zen提供4个免费模型。media-gen支持DALL-E3和Agnes图片视频生成。
  <!-- tokeny-memory: {"id":"mem_707ffc103586d0d6","category":"fact","keywords":"","importance":8,"createdAt":1780823758732,"updatedAt":1780823758732,"accessCount":0} -->
- Docker CI/CD踩坑记录：Windows开发需注意NTFS大小写不敏感导致的Git追踪问题(git mv -f强制更正)。曾遇到8个构建失败：Cargo.toml小写/npm缺lockfile/src目录不存在/tsc类型检查/package.json语法错误/App.tsx大小写/workspace.dependencies缺失/JSX标签不匹配。Dockerfile用3阶段(Node→Rust→Runtime)。docker-compose支持dev/prod双模式。
  <!-- tokeny-memory: {"id":"mem_855eed253f23e653","category":"fact","keywords":"","importance":7,"createdAt":1780823758732,"updatedAt":1780823758732,"accessCount":0} -->
