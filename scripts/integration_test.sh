#!/usr/bin/env bash
# Pastebin 集成测试脚本
# 依赖: curl, 需先启动服务 (cargo run)
BASE="${BASE_URL:-http://127.0.0.1:8080}"
PASS=0
FAIL=0
REPORT=""
CURL_OPTS="-s -m 5"

assert() {
  local name="$1"
  local cond="$2"
  if eval "$cond"; then
    ((PASS++)) || true
    REPORT+="| ✅ $name | 通过 |"$'\n'
    return 0
  else
    ((FAIL++)) || true
    REPORT+="| ❌ $name | 失败 |"$'\n'
    return 1
  fi
}

echo "=== Pastebin 集成测试 ==="
echo "BASE_URL=$BASE"
echo ""

# 1. GET / 首页
echo "[1] GET / 首页..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_index.html -w "%{http_code}" "$BASE/" || echo "000")
assert "GET / 返回 200" "[[ '$RES' == '200' ]]"
assert "首页含提交表单" "grep -q 'name=\"content\"' /tmp/pastebin_index.html"
assert "首页含 viewport" "grep -q 'viewport' /tmp/pastebin_index.html"
assert "首页含导航(所有记录)" "grep -q 'href=\"/list\"' /tmp/pastebin_index.html"

# 2. GET /list 列表
echo "[2] GET /list 列表..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_list.html -w "%{http_code}" "$BASE/list" || echo "000")
assert "GET /list 返回 200" "[[ '$RES' == '200' ]]"
assert "列表页含导航" "grep -q 'href=\"/\"' /tmp/pastebin_list.html"

# 3. POST /paste 创建
echo "[3] POST /paste 创建..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_redirect.txt -w "%{http_code}" -D /tmp/pastebin_headers.txt \
  -X POST "$BASE/paste" \
  -F "content=Hello World Test $(date +%s)" \
  -F "title=TestTitle" \
  -F "author=TestAuthor" \
  -F "language=python" \
  -F "expires=1d" || echo "000")
assert "POST /paste 返回 302/303" "[[ '$RES' == '302' || '$RES' == '303' ]]"
LOC=$(grep -i "^Location:" /tmp/pastebin_headers.txt 2>/dev/null | tr -d '\r' | cut -d' ' -f2)
assert "重定向到 /p/ID" "[[ -n '$LOC' && '$LOC' =~ ^/p/[a-zA-Z0-9]+$ ]]"
PASTE_ID="${LOC#/p/}"
echo "  创建 Paste ID: $PASTE_ID"

# 4. GET /p/{id} 查看
echo "[4] GET /p/$PASTE_ID 查看..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_view.html -w "%{http_code}" "$BASE$LOC" || echo "000")
assert "GET /p/ID 返回 200" "[[ '$RES' == '200' ]]"
assert "查看页含内容" "grep -q 'Hello World' /tmp/pastebin_view.html"
assert "查看页含标题" "grep -q 'TestTitle' /tmp/pastebin_view.html"
assert "查看页含作者" "grep -q 'TestAuthor' /tmp/pastebin_view.html"
assert "查看页含 highlight 或 code" "grep -qE 'highlight|language-|<code|pre' /tmp/pastebin_view.html"
assert "查看页含编辑链接" "grep -q '/edit' /tmp/pastebin_view.html"

# 5. GET /p/{id}/edit 编辑表单
echo "[5] GET /p/$PASTE_ID/edit 编辑表单..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_edit.html -w "%{http_code}" "$BASE/p/$PASTE_ID/edit" || echo "000")
assert "GET /p/ID/edit 返回 200" "[[ '$RES' == '200' ]]"
assert "编辑页含 content 预填" "grep -q 'Hello World Test' /tmp/pastebin_edit.html"
assert "编辑页表单 action 指向 /p/ID/edit" "grep -q 'action=.*edit' /tmp/pastebin_edit.html"

# 6. POST /p/{id}/edit 更新
echo "[6] POST /p/$PASTE_ID/edit 更新..."
RES=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" -L \
  -X POST "$BASE/p/$PASTE_ID/edit" \
  -F "content=Updated content $(date +%s)" \
  -F "title=UpdatedTitle" \
  -F "author=UpdatedAuthor" \
  -F "language=text" \
  -F "expires=1w" || echo "000")
assert "POST /p/ID/edit 返回 302/303/200" "[[ '$RES' == '302' || '$RES' == '303' || '$RES' == '200' ]]"
RES2=$(curl $CURL_OPTS -o /tmp/pastebin_view2.html -w "%{http_code}" "$BASE/p/$PASTE_ID" || echo "000")
assert "更新后查看页返回 200" "[[ '$RES2' == '200' ]]"
assert "更新后内容已变更" "grep -q 'Updated' /tmp/pastebin_view2.html"

# 7. GET /list 应包含该条
echo "[7] GET /list 应包含该条..."
curl $CURL_OPTS -o /tmp/pastebin_list2.html "$BASE/list" || true
assert "列表页含该 Paste ID" "grep -q \"$PASTE_ID\" /tmp/pastebin_list2.html"
assert "列表页含查看/编辑/删除" "grep -q '删除' /tmp/pastebin_list2.html"

# 8. POST /p/{id}/delete 删除
echo "[8] POST /p/$PASTE_ID/delete 删除..."
RES=$(curl $CURL_OPTS -o /tmp/pastebin_after_del.txt -w "%{http_code}" -D /tmp/pastebin_del_headers.txt \
  -X POST "$BASE/p/$PASTE_ID/delete" || echo "000")
assert "POST /p/ID/delete 返回 302/303" "[[ '$RES' == '302' || '$RES' == '303' ]]"
DEL_LOC=$(grep -i "^Location:" /tmp/pastebin_del_headers.txt | tr -d '\r' | cut -d' ' -f2)
assert "删除后重定向到 /list" "[[ '$DEL_LOC' == '/list' ]]"

# 9. 删除后 GET /p/{id} 应 404
echo "[9] 删除后 GET /p/$PASTE_ID 应 404..."
RES=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" "$BASE/p/$PASTE_ID" || echo "000")
assert "删除后 GET /p/ID 返回 404" "[[ '$RES' == '404' ]]"

# 10. 静态资源与多端
echo "[10] 静态资源..."
RES_CSS=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" "$BASE/static/css/style.css" || echo "000")
RES_JS=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" "$BASE/static/js/app.js" || echo "000")
assert "CSS 可访问" "[[ '$RES_CSS' == '200' ]]"
assert "JS 可访问" "[[ '$RES_JS' == '200' ]]"

# 11. 过期 / 不存在
echo "[11] GET /p/nonexistent 应 404..."
RES=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" "$BASE/p/nonexistent123" || echo "000")
assert "不存在的 ID 返回 404" "[[ '$RES' == '404' ]]"

# 12. 创建带附件的 Paste（可选：无附件也通过）
echo "[12] POST /paste 带文本（multipart 同表单）..."
RES2=$(curl $CURL_OPTS -o /dev/null -w "%{http_code}" -D /tmp/pastebin_headers2.txt \
  -X POST "$BASE/paste" \
  -F "content=Attachment test $(date +%s)" \
  -F "title=WithFile" \
  -F "expires=1d")
LOC2=$(grep -i "^Location:" /tmp/pastebin_headers2.txt 2>/dev/null | tr -d '\r' | cut -d' ' -f2)
assert "带表单的 POST /paste 成功" "[[ '$RES2' == '302' || '$RES2' == '303' ]]"
if [[ -n "$LOC2" ]]; then
  RES_V=$(curl $CURL_OPTS -o /tmp/pastebin_view3.html -w "%{http_code}" "$BASE$LOC2" || echo "000")
  assert "新 Paste 可查看" "[[ '$RES_V' == '200' ]]"
fi

echo ""
echo "=== 结果 ==="
echo "通过: $PASS, 失败: $FAIL"
echo ""
echo "| 用例 | 结果 |"
echo "|------|------|"
echo "$REPORT"
# 若无法连接则提示
if [[ $PASS -eq 0 && $FAIL -gt 0 ]]; then
  echo ""
  echo "提示: 若大量失败请先启动服务: cargo run"
fi
[[ $FAIL -eq 0 ]] && exit 0 || exit 1
