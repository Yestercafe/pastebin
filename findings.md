# Findings & Decisions

## Requirements
- 粘贴文本 → 唯一链接 → 查看；过期时间、标题/作者、语法高亮
- 上传图片（内联）、附件（列表+下载）
- 列出全部记录；完整 CRUD（查看、编辑、删除）
- 前端兼容 PC、手机、iPad
- 单用户自用；预留 user_id 便于日后登录

## Technical Decisions
| Decision | Rationale |
|----------|-----------|
| Actix-web | 计划指定 |
| SQLite + sqlx | 计划指定，异步 + 编译期 SQL |
| minijinja | 模板语法熟悉，运行时加载 |
| 文件存 data/\<id\>/ | 按 paste 隔离，删除时整目录清理 |
