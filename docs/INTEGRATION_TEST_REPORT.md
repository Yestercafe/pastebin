# Pastebin 集成测试报告

**测试方式**: 自动化脚本 `scripts/integration_test.sh`（curl 请求）  
**被测服务**: http://127.0.0.1:8080（需先执行 `cargo run`）

---

## 1. 测试范围与需求对照

| 需求 | 测试用例 |
|------|----------|
| 核心：粘贴 → 链接 → 查看 | POST /paste → 302/303 → GET /p/{id}，查看页含内容/标题/作者 |
| 过期、标题、作者、语法高亮 | 表单提交 title/author/language/expires，查看页含 code/highlight |
| 列出全部记录 | GET /list，列表含该 Paste、含查看/编辑/删除 |
| 编辑 | GET /p/{id}/edit 预填，POST 更新，再查看内容已变更 |
| 删除 | POST /p/{id}/delete → 302/303 → /list，再 GET /p/{id} 得 404 |
| 静态资源、多端 | 首页 viewport、导航；CSS/JS 可访问 |
| 不存在 | GET /p/nonexistent → 404 |
| 图片/附件 | POST /paste 支持 multipart 表单 |

---

## 2. 测试用例与结果

运行 `bash scripts/integration_test.sh` 后，各用例通过即表示功能正确。

| 用例 | 结果 |
|------|------|
| GET / 返回 200 | 通过 |
| 首页含提交表单 | 通过 |
| 首页含 viewport | 通过 |
| 首页含导航（所有记录） | 通过 |
| GET /list 返回 200 | 通过 |
| 列表页含导航 | 通过 |
| POST /paste 返回 302/303 | 通过 |
| 重定向到 /p/ID | 通过 |
| GET /p/ID 返回 200 | 通过 |
| 查看页含内容 | 通过 |
| 查看页含标题 | 通过 |
| 查看页含作者 | 通过 |
| 查看页含 highlight 或 code | 通过 |
| 查看页含编辑链接 | 通过 |
| GET /p/ID/edit 返回 200 | 通过 |
| 编辑页含 content 预填 | 通过 |
| 编辑页表单 action 指向 /p/ID/edit | 通过 |
| POST /p/ID/edit 返回 302/303/200 | 通过 |
| 更新后查看页返回 200 | 通过 |
| 更新后内容已变更 | 通过 |
| 列表页含该 Paste ID | 通过 |
| 列表页含查看/编辑/删除 | 通过 |
| POST /p/ID/delete 返回 302/303 | 通过 |
| 删除后重定向到 /list | 通过 |
| 删除后 GET /p/ID 返回 404 | 通过 |
| CSS 可访问 | 通过 |
| JS 可访问 | 通过 |
| 不存在的 ID 返回 404 | 通过 |
| 带表单的 POST /paste 成功 | 通过 |
| 新 Paste 可查看 | 通过 |

---

## 3. 如何运行

1. 启动服务：
   ```bash
   cd /home/ivan/repos/pastebin
   cargo run
   ```
2. 新开终端执行测试：
   ```bash
   cd /home/ivan/repos/pastebin
   bash scripts/integration_test.sh
   ```
3. 可选：保存输出
   ```bash
   bash scripts/integration_test.sh 2>&1 | tee docs/test_run_$(date +%Y%m%d_%H%M).log
   ```

---

## 4. 附录

- 测试脚本: `scripts/integration_test.sh`
- 最近一次控制台输出: `scripts/test_run.log`
