# Project Memory

## fact

- Docker CI/CD踩坑记录：Windows NTFS大小写不敏感导致Git追踪问题(需git mv -f)。前端JSX在多次子任务编辑后标签嵌套损坏(需检查div平衡)。Dockerfile用3阶段(Node→Rust→Runtime)。github仓库suanx/remo，邮箱kobex@189.cn。
  <!-- tokeny-memory: {"id":"mem_92393a8a838c02f5","category":"fact","keywords":"","importance":7,"createdAt":1780827335132,"updatedAt":1780827335132,"accessCount":0} -->

## event

- 当前Docker构建阻塞：crates/remo/Cargo.toml的feature定义与dependencies不匹配(sandbox/agnes等feature引用了未在dependencies中声明的crate)。需要在crates/remo/Cargo.toml的[dependencies]中添加所有feature引用的ext crate依赖。
  <!-- tokeny-memory: {"id":"mem_8a84101e81cf493b","category":"event","keywords":"","importance":9,"createdAt":1780827335134,"updatedAt":1780827335134,"accessCount":0} -->
