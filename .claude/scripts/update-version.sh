#!/bin/bash
# 自动更新 README.md 的脚本
# 功能：1. 更新版本号  2. 更新变更日志  3. 更新开发进度

set -e

cd "$(dirname "$0")/../.."

README="README.md"
CHANGES_MADE=0

# macOS/Linux 兼容的 sed -i
inplace_sed() {
    if [[ "$OSTYPE" == "darwin"* ]]; then
        sed -i '' "$@"
    else
        sed -i "$@"
    fi
}

# 缓存 git 数据，避免重复调用
LATEST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
RECENT_COMMITS=$(git log -10 --pretty=format:"%s" 2>/dev/null || echo "")

# ========== 辅助函数：更新开发进度项 ==========
update_progress_item() {
    local pattern="$1"
    local item="$2"
    local name="$3"

    if echo "$RECENT_COMMITS" | grep -iqE "$pattern" && grep -q "\- \[ \] $item" "$README"; then
        inplace_sed "s/- \[ \] $item/- [x] $item/g" "$README"
        echo "✅ 开发进度已更新: $name"
        CHANGES_MADE=1
    fi
}

# ========== 1. 更新版本号 ==========
update_version() {
    if [ -z "$LATEST_TAG" ]; then
        echo "⚠️  未找到 git tag，跳过版本更新"
        return 0
    fi

    local CURRENT_VERSION=$(grep -o 'Download-v[0-9]*\.[0-9]*\.[0-9]*' "$README" | head -1 | sed 's/Download-//')

    if [ -z "$CURRENT_VERSION" ]; then
        echo "⚠️  README.md 中未找到版本号，跳过"
        return 0
    fi

    if [ "$LATEST_TAG" = "$CURRENT_VERSION" ]; then
        echo "✅ 版本号已是最新: $LATEST_TAG"
        return 0
    fi

    inplace_sed "s/Download-$CURRENT_VERSION/Download-$LATEST_TAG/g" "$README"
    echo "✅ 版本号已更新: $CURRENT_VERSION → $LATEST_TAG"
    CHANGES_MADE=1
}

# ========== 2. 更新变更日志 ==========
update_changelog() {
    # 删除旧的变更日志部分
    if grep -q "## 📝 最近更新" "$README"; then
        echo "✅ 变更日志部分已存在，更新中..."
        awk '
        /^## 📝 最近更新/ { in_changelog=1; next }
        (/^---$/ || /^## 📄 License/) && in_changelog { in_changelog=0 }
        !in_changelog { print }
        ' "$README" > "$README.tmp" && mv "$README.tmp" "$README"
    fi

    # 获取最近 5 次 commit（从缓存截取）
    local COMMITS=$(echo "$RECENT_COMMITS" | head -5 | sed 's/^/- /')

    if [ -z "$COMMITS" ]; then
        echo "⚠️  未找到 commit 记录"
        return 0
    fi

    local VERSION="${LATEST_TAG:-Unreleased}"

    # 创建变更日志内容
    local TMPFILE=$(mktemp)
    {
        echo ""
        echo "## 📝 最近更新"
        echo ""
        echo "### $VERSION"
        echo ""
        echo "$COMMITS"
        echo ""
        echo "---"
        echo ""
    } > "$TMPFILE"

    # 在 License 部分之前插入
    local LICENSE_LINE=$(grep -n "^## 📄 License" "$README" | head -1 | cut -d: -f1)

    if [ -n "$LICENSE_LINE" ]; then
        head -n $((LICENSE_LINE - 1)) "$README" > "$README.tmp"
        cat "$TMPFILE" >> "$README.tmp"
        tail -n +$LICENSE_LINE "$README" >> "$README.tmp"
        mv "$README.tmp" "$README"
        echo "✅ 变更日志已更新"
        CHANGES_MADE=1
    else
        echo "⚠️  未找到 License 部分"
    fi
    rm -f "$TMPFILE"
}

# ========== 3. 更新开发进度 ==========
update_progress() {
    update_progress_item "提示系统|hint" "提示系统（自动寻找可移动牌）" "提示系统"
    update_progress_item "主题|theme" "更多主题选择" "更多主题选择"
    update_progress_item "多语言|i18n|国际化" "多语言支持" "多语言支持"

    if [ $CHANGES_MADE -eq 0 ]; then
        echo "✅ 开发进度无需更新"
    fi
}

# ========== 主流程 ==========
echo "🔄 开始更新 README.md..."
echo ""

update_version
update_changelog
update_progress

echo ""
if [ $CHANGES_MADE -eq 1 ]; then
    echo "✨ README.md 更新完成，记得提交更改"
else
    echo "✨ README.md 无需更改"
fi
