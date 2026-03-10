# Pastebin

自用类 Ubuntu Pastebin：粘贴文本、图片/附件、语法高亮、全量 CRUD、列表，支持 PC/手机/iPad。

## 技术栈

- **后端**: Rust, Actix-web, SQLite (sqlx)
- **模板**: minijinja
- **前端**: 静态 HTML/CSS/JS，highlight.js（语法高亮）

## 运行

1. 安装 Rust：<https://rustup.rs/>，并执行 `rustup default stable`。
2. 克隆或进入项目目录后：

```bash
cargo run
```

3. 浏览器打开 <http://127.0.0.1:8080>。

## 配置文件

默认读取当前目录下的 `pastebin.toml`；路径可由环境变量 `CONFIG` 或 `PASTEBIN_CONFIG` 指定。配置项示例见项目根目录的 `pastebin.toml`（`database-url`、`host`、`port`、`data-dir`、`templates-dir`、`static-dir`）。文件不存在时使用内置默认值。

服务器部署详见 [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md)。

## 功能

- **新建**: 首页提交文本，可选标题/作者、语法、保留时间；可同时上传多张图片或附件。
- **列表**: `/list` 查看全部记录，支持查看、编辑、删除。
- **查看**: `/p/<id>` 查看单条，代码高亮，图片内联，附件可下载。
- **编辑**: `/p/<id>/edit` 修改内容与元数据（附件仅展示，不在此页增删）。
- **删除**: 列表或查看页可删除，会同时删除附件文件。

## 限制与安全

- 单条文本内容上限 512KB。
- 单文件上传上限 5MB，单 Paste 总上传 20MB。
- 允许类型：图片（image/*）、PDF、txt、zip、json、csv 等；禁止可执行与脚本类型。
- 上传文件以随机名存于 `data/<paste_id>/`，`.gitignore` 已忽略 `data/`。

## 后续扩展

- 用户登录：表已预留 `pastes.user_id`，可加用户表与会话后按 user_id 过滤。
- pastebinit 兼容 API、过期定时清理、列表分页等可按需添加。
