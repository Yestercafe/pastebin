# 服务器部署

支持两种方式：**仅部署制品**（推荐，服务器无需代码与 Rust），或在服务器上构建。

---

## 1. 仅部署制品（服务器不放代码）

在本地或 CI 机器上构建，把**二进制 + 模板 + 静态资源**打包上传，服务器只运行、不装 Rust、不拉代码。

### 1.1 本地打包

在项目目录执行（或直接运行 `./scripts/make-dist.sh`）：

```bash
cargo build --release
mkdir -p dist
cp target/release/pastebin dist/
cp -r templates static dist/
# data 与数据库在服务器上新建，不打包
tar -czvf pastebin-dist.tar.gz -C dist .
```

得到 `pastebin-dist.tar.gz`，内含：`pastebin`、`templates/`、`static/`。

### 1.2 上传到服务器

```bash
scp pastebin-dist.tar.gz user@server:/opt/
ssh user@server "cd /opt && tar -xzvf pastebin-dist.tar.gz && mkdir -p data && chown www-data:www-data data"
```

部署目录建议固定为 `/opt/pastebin`，解压后结构示例：

```
/opt/pastebin/
├── pastebin          # 二进制
├── templates/
├── static/
└── data/             # 服务器上新建，放数据库与上传文件
```

### 1.3 服务器上运行

无需安装 Rust，直接运行（或交给 systemd，见下文）：

```bash
cd /opt/pastebin
export HOST=127.0.0.1
export PORT=8080
export DATABASE_URL=sqlite:///opt/pastebin/data/pastebin.db
export DATA_DIR=/opt/pastebin/data
export TEMPLATES_DIR=/opt/pastebin/templates
./pastebin
```

更新时：重新打包、上传、覆盖二进制和 templates/static，然后重启进程即可。

---

## 2. 在服务器上构建（可选）

若你希望直接在服务器上编译：

安装 Rust 后克隆并构建：

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup default stable

cd /opt && git clone <你的仓库> pastebin && cd pastebin
cargo build --release
```

二进制在 `target/release/pastebin`，运行方式同上；或使用下面 systemd 时把 `ExecStart` 改为 `target/release/pastebin`、`WorkingDirectory` 指向仓库根目录。

---

## 3. 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `HOST` | `127.0.0.1` | 监听地址，反代时用 127.0.0.1，直连用 0.0.0.0 |
| `PORT` | `8080` | 监听端口 |
| `DATABASE_URL` | `sqlite://pastebin.db` | SQLite 路径，建议用绝对路径如 `sqlite:///opt/pastebin/data/pastebin.db` |
| `TEMPLATES_DIR` | `templates` | 模板目录，相对或绝对 |
| `DATA_DIR` | `data` | 上传文件目录，相对或绝对，需可写 |

建议在部署目录下建 `data`、放数据库和上传文件，并保证进程用户有写权限。

---

## 4. systemd 服务

创建系统服务，开机自启（适用于「仅部署制品」的目录结构）：

```bash
sudo vim /etc/systemd/system/pastebin.service
```

```ini
[Unit]
Description=Pastebin
After=network.target

[Service]
Type=simple
User=www-data
Group=www-data
WorkingDirectory=/opt/pastebin
Environment="HOST=127.0.0.1"
Environment="PORT=8080"
Environment="DATABASE_URL=sqlite:///opt/pastebin/data/pastebin.db"
Environment="DATA_DIR=/opt/pastebin/data"
Environment="TEMPLATES_DIR=/opt/pastebin/templates"
ExecStart=/opt/pastebin/pastebin
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

注意：`ExecStart` 指向解压后的二进制 `/opt/pastebin/pastebin`。若你在服务器上从源码构建，改为 `ExecStart=/opt/pastebin/target/release/pastebin`。

创建数据目录并授权：

```bash
sudo mkdir -p /opt/pastebin/data
sudo chown www-data:www-data /opt/pastebin/data
sudo systemctl daemon-reload
sudo systemctl enable pastebin
sudo systemctl start pastebin
sudo systemctl status pastebin
```

---

## 5. Nginx 反代（可选）

让 Nginx 对外提供 HTTP/HTTPS，转发到本机 8080。

```nginx
server {
    listen 80;
    server_name paste.example.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        client_max_body_size 25M;
    }
}
```

上传限制需略大于单 Paste 总大小（如 20MB），故设 `client_max_body_size 25M`。若启用 HTTPS，用 certbot 或自签证书即可。

---

## 6. 部署检查

- 数据库与上传目录路径正确、进程用户可写。
- 若只通过 Nginx 访问，`HOST` 保持 `127.0.0.1` 即可。
- **仅部署制品**时更新：本地重新打包、上传覆盖二进制和 templates/static，再 `systemctl restart pastebin`。
