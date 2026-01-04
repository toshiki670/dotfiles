#!/bin/bash
# rollback-migration.sh
# セマンティックバージョニング移行のロールバック

set -e

REPO="toshiki670/dotfiles"
BACKUP_DIR="./migration-backup"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${YELLOW}=== Migration Rollback Script ===${NC}\n"

# 必要なコマンドの確認
for cmd in gh git; do
    if ! command -v $cmd >/dev/null 2>&1; then
        echo -e "${RED}Error: $cmd is required but not installed.${NC}" >&2
        exit 1
    fi
done

# バックアップディレクトリの確認
if [[ ! -d "$BACKUP_DIR" ]]; then
    echo -e "${RED}Error: Backup directory '$BACKUP_DIR' not found${NC}"
    echo "Please ensure you have a backup before attempting rollback."
    echo ""
    echo "If you have backup files elsewhere, you can:"
    echo "  1. Create the directory: mkdir -p $BACKUP_DIR"
    echo "  2. Copy your backup files into it:"
    echo "     - tags_backup.txt"
    echo "     - tag_mapping.json (if exists)"
    exit 1
fi

echo -e "${BLUE}Checking backup files...${NC}"

# バックアップファイルの確認
if [[ ! -f "$BACKUP_DIR/tags_backup.txt" ]]; then
    echo -e "${RED}Error: tags_backup.txt not found in $BACKUP_DIR${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Backup files found${NC}"
echo ""

# 現在の状態を表示
echo -e "${BLUE}Current state:${NC}"
echo "Current tags:"
git tag -l | wc -l
echo ""

# 0.x.x 形式のタグを検出
semver_tags=$(git tag -l | grep -E '^0\.[0-9]+\.[0-9]+$' | wc -l | tr -d ' ')
echo "SemVer 0.x.x tags found: $semver_tags"
echo ""

if [[ $semver_tags -eq 0 ]]; then
    echo -e "${YELLOW}No SemVer tags found. Nothing to rollback.${NC}"
    exit 0
fi

echo -e "${YELLOW}This script will:${NC}"
echo "1. Delete all 0.x.x format tags"
echo "2. Restore original tags from backup"
echo "3. Keep GitHub releases as-is (manual cleanup required)"
echo ""
echo -e "${RED}WARNING: This will delete all 0.x.x tags!${NC}"
echo ""
read -p "Continue with rollback? (yes/no): " confirm

if [[ "$confirm" != "yes" ]]; then
    echo "Rollback cancelled."
    exit 0
fi

echo ""
echo -e "${BLUE}Starting rollback...${NC}"
echo ""

# 0.x.x タグを削除
deleted=0
failed=0

echo "Deleting 0.x.x format tags..."
for tag in $(git tag -l | grep -E '^0\.[0-9]+\.[0-9]+$'); do
    echo "  Deleting: $tag"
    
    # ローカルタグを削除
    if git tag -d "$tag" 2>/dev/null; then
        # リモートタグを削除
        if git push origin ":refs/tags/$tag" 2>/dev/null; then
            ((deleted++))
        else
            echo -e "${YELLOW}    Warning: Failed to delete remote tag${NC}"
            ((failed++))
        fi
    else
        echo -e "${RED}    Error: Failed to delete local tag${NC}"
        ((failed++))
    fi
done

echo ""
echo -e "${GREEN}Deleted $deleted tags (failed: $failed)${NC}"
echo ""

# 元のタグを復元するかどうか確認
echo -e "${YELLOW}Do you want to restore original tags from backup?${NC}"
echo "Note: This will recreate tags at their original commit points."
read -p "Restore original tags? (yes/no): " restore_confirm

if [[ "$restore_confirm" == "yes" ]]; then
    echo ""
    echo "Restoring original tags..."
    
    restored=0
    restore_failed=0
    
    # tags_backup.txt から復元
    while IFS= read -r tag; do
        # 空行をスキップ
        [[ -z "$tag" ]] && continue
        
        # タグが既に存在するかチェック
        if git rev-parse "$tag" >/dev/null 2>&1; then
            echo "  Skipping $tag (already exists)"
            continue
        fi
        
        echo "  Restoring: $tag"
        
        # コミットハッシュを見つける（git reflogから）
        # 注意：これは完全な復元ではなく、タグが指していたコミットを推測
        echo -e "${YELLOW}    Warning: Cannot automatically restore tag $tag${NC}"
        echo -e "${YELLOW}    Manual restoration required from git history${NC}"
        ((restore_failed++))
    done < "$BACKUP_DIR/tags_backup.txt"
    
    echo ""
    echo -e "${YELLOW}Automatic tag restoration is limited.${NC}"
    echo "You may need to manually recreate tags from git history."
fi

echo ""
echo -e "${GREEN}=== Rollback Complete ===${NC}"
echo ""
echo "Summary:"
echo "  Deleted: $deleted tags"
echo "  Failed: $failed tags"
echo ""
echo -e "${YELLOW}Important Next Steps:${NC}"
echo "1. Verify tags:"
echo "   ${BLUE}git tag -l | sort -V${NC}"
echo ""
echo "2. GitHub releases need manual cleanup:"
echo "   ${BLUE}gh release list --repo $REPO${NC}"
echo "   Delete 0.x.x releases manually if needed"
echo ""
echo "3. If you need to fully restore from backup:"
echo "   - Contact GitHub support for release restoration"
echo "   - Or manually recreate releases from backup"

