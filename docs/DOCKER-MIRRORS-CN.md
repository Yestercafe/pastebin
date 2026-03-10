# Docker 国内镜像源配置

拉取 `ubuntu:20.04`、`debian:bullseye` 等镜像时若出现 **short read**、**content size of zero**、**unexpected EOF**，多为网络或镜像源问题。可配置国内镜像加速后重试。

## 推荐镜像源（仅供参考，以实际可用为准）

| 镜像源 | 地址 | 说明 |
|--------|------|------|
| 轩辕镜像 | `https://docker.xuanyuan.me` | 社区维护，免费 |
| DaoCloud | `https://docker.m.daocloud.io` | 长期运营 |
| 1ms.run | `https://docker.1ms.run` | 社区源 |

**说明**：国内镜像站时有变动或限速，若某个不可用可换其他或搜索「Docker 镜像加速 2025」获取最新列表。

## Linux 配置方法

```bash
sudo mkdir -p /etc/docker
sudo tee /etc/docker/daemon.json <<'EOF'
{
  "registry-mirrors": [
    "https://docker.xuanyuan.me",
    "https://docker.m.daocloud.io",
    "https://docker.1ms.run"
  ]
}
EOF
sudo systemctl daemon-reload
sudo systemctl restart docker
```

验证：`docker info | grep -A 5 "Registry Mirrors"` 应能看到上述地址。

## macOS / Windows

- **macOS**：Docker Desktop → Settings → Docker Engine → 在 JSON 中为 `registry-mirrors` 添加上述地址 → Apply & Restart。
- **Windows**：Docker Desktop → Settings → Docker Engine → 同上。

## 配置后重试构建

```bash
./scripts/make-dist-ubuntu2004.sh
```

若仍失败，可先单独拉取测试：`docker pull debian:bullseye`。
