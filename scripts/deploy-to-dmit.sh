#!/usr/bin/env bash
# 部署到 dmit 服务器（需在项目根目录执行，且 SSH Host dmit 已配置）
set -e
cd "$(dirname "$0")/.."
HOST="${1:-dmit}"
DEPLOY_DIR="/opt/pastebin"
TARBALL="pastebin-dist.tar.gz"

if [[ ! -f "$TARBALL" ]]; then
  echo "错误: 未找到 $TARBALL，请先运行 ./scripts/make-dist.sh"
  exit 1
fi

echo "上传 $TARBALL 到 $HOST:/opt/ ..."
scp -o ConnectTimeout=15 "$TARBALL" "$HOST:/opt/"

echo "在服务器上解压并创建 data 目录..."
ssh "$HOST" "mkdir -p $DEPLOY_DIR && cd $DEPLOY_DIR && tar -xzf /opt/$TARBALL && mkdir -p data && chown -R www-data:www-data $DEPLOY_DIR 2>/dev/null || true"

echo "部署完成. 若需用 systemd 托管，在服务器上执行："
echo "  sudo systemctl start pastebin   # 需先配置 /etc/systemd/system/pastebin.service"
echo "或直接运行："
echo "  ssh $HOST 'cd $DEPLOY_DIR && HOST=127.0.0.1 PORT=8080 DATABASE_URL=sqlite:///$DEPLOY_DIR/data/pastebin.db DATA_DIR=$DEPLOY_DIR/data TEMPLATES_DIR=$DEPLOY_DIR/templates nohup ./pastebin &'"
