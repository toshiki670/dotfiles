#!/bin/bash
# migrate-add-v-prefix.sh
# 0.x.x タグに v プレフィックスを追加（0.x.x → v0.x.x）

set -e

REPO="toshiki670/dotfiles"
TEMP_DIR=$(mktemp -d)
BACKUP_DIR="./migration-backup"
DRY_RUN=false
BACKUP_ENABLED=true

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 使用方法
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --dry-run        Show what would be done without making changes"
    echo "  --no-backup      Skip automatic backup creation"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "This script adds 'v' prefix to all 0.x.x tags (0.1.0 → v0.1.0)"
    exit 0
}

# 引数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --no-backup)
            BACKUP_ENABLED=false
            shift
            ;;
        -h|--help)
            usage
            ;;
        *)
            echo "Unknown option: $1"
            usage
            ;;
    esac
done

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

if [[ "$DRY_RUN" == true ]]; then
    echo -e "${YELLOW}=== DRY RUN: Add v prefix to tags ===${NC}\n"
    echo -e "${YELLOW}No changes will be made${NC}\n"
else
    echo -e "${YELLOW}=== Add v prefix to tags ===${NC}\n"
fi

# 環境チェック
echo -e "${BLUE}Checking environment...${NC}"

if ! git rev-parse --git-dir > /dev/null 2>&1; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

REPO_ROOT=$(git rev-parse --show-toplevel)
CURRENT_DIR=$(pwd)

if [[ "$REPO_ROOT" != "$CURRENT_DIR" ]]; then
    echo -e "${RED}Error: Script must be run from repository root${NC}"
    echo "Please run: cd $REPO_ROOT"
    exit 1
fi

CURRENT_BRANCH=$(git branch --show-current)
echo "Current branch: ${YELLOW}$CURRENT_BRANCH${NC}"
echo ""

# コマンド確認
for cmd in git gh jq; do
    if ! command -v $cmd >/dev/null 2>&1; then
        echo -e "${RED}Error: $cmd is required but not installed${NC}"
        exit 1
    fi
done

# GitHub認証確認
if ! gh auth status >/dev/null 2>&1; then
    echo -e "${RED}Error: Not authenticated with GitHub CLI${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Environment check passed${NC}"
echo ""

# 0.x.x形式のタグを取得
echo -e "${BLUE}Finding 0.x.x tags...${NC}"
tags_to_migrate=$(git tag -l | grep -E '^0\.[0-9]+\.[0-9]+$' | sort -V)

if [[ -z "$tags_to_migrate" ]]; then
    echo -e "${YELLOW}No 0.x.x tags found. Nothing to do.${NC}"
    exit 0
fi

tag_count=$(echo "$tags_to_migrate" | wc -l | tr -d ' ')
echo "Found $tag_count tags to migrate:"
echo "$tags_to_migrate"
echo ""

if [[ "$DRY_RUN" == false ]]; then
    echo -e "${YELLOW}This script will:${NC}"
    echo "1. Add 'v' prefix to all 0.x.x tags"
    echo "2. Update GitHub releases"
    echo "3. Delete old tags without prefix"
    echo ""
    echo -e "${RED}WARNING: This will modify tags and releases!${NC}"
    echo ""
    
    # バックアップ作成
    if [[ "$BACKUP_ENABLED" == true ]]; then
        echo -e "${BLUE}Creating backup...${NC}"
        mkdir -p "$BACKUP_DIR"
        
        git tag -l > "$BACKUP_DIR/tags_before_v_prefix_$(date +%Y%m%d_%H%M%S).txt"
        gh release list --repo "$REPO" --limit 100 > "$BACKUP_DIR/releases_before_v_prefix_$(date +%Y%m%d_%H%M%S).txt" 2>/dev/null || true
        
        echo -e "${GREEN}✓ Backup created in $BACKUP_DIR${NC}"
        echo ""
    fi
    
    read -p "Continue? (yes/no): " confirm
    if [[ "$confirm" != "yes" ]]; then
        echo "Migration cancelled."
        exit 0
    fi
fi

echo ""
echo -e "${BLUE}Starting migration...${NC}"
echo ""

processed=0
skipped=0
failed=0

while IFS= read -r old_tag; do
    new_tag="v$old_tag"
    
    echo -e "${GREEN}→ $old_tag → $new_tag${NC}"
    
    if [[ "$DRY_RUN" == true ]]; then
        if gh release view "$old_tag" --repo "$REPO" >/dev/null 2>&1; then
            echo "  [DRY RUN] Would migrate release"
        fi
        echo "  [DRY RUN] Would create tag $new_tag"
        echo "  [DRY RUN] Would delete tag $old_tag"
        echo ""
        ((processed++))
        continue
    fi
    
    # コミットハッシュ取得
    commit_hash=$(git rev-parse "$old_tag")
    commit_date=$(git log -1 --format=%ai "$old_tag")
    echo "  Commit: ${commit_hash:0:8} ($commit_date)"
    
    # タグメッセージ取得
    tag_message=$(git tag -l --format='%(contents)' "$old_tag")
    
    # 新しいタグを作成
    if [[ -n "$tag_message" ]]; then
        git tag -a "$new_tag" "$commit_hash" -m "$tag_message

---
Added v prefix: $old_tag → $new_tag"
    else
        git tag -a "$new_tag" "$commit_hash" -m "Version $new_tag

Added v prefix to follow conventional tagging"
    fi
    
    # プッシュ
    if ! git push origin "$new_tag" 2>/dev/null; then
        echo -e "${RED}  ✗ Failed to push tag${NC}"
        git tag -d "$new_tag" 2>/dev/null
        ((failed++))
        continue
    fi
    
    # GitHubリリースを移行
    if gh release view "$old_tag" --repo "$REPO" >/dev/null 2>&1; then
        echo "  Migrating release..."
        
        release_json=$(gh release view "$old_tag" --repo "$REPO" --json \
            name,body,isDraft,isPrerelease,assets 2>/dev/null)
        
        if [[ $? -eq 0 ]]; then
            release_title=$(echo "$release_json" | jq -r '.name')
            release_body=$(echo "$release_json" | jq -r '.body')
            is_draft=$(echo "$release_json" | jq -r '.isDraft')
            is_prerelease=$(echo "$release_json" | jq -r '.isPrerelease')
            asset_count=$(echo "$release_json" | jq -r '.assets | length')
            
            # アセットダウンロード
            asset_paths=()
            if [[ $asset_count -gt 0 ]]; then
                echo "  Downloading $asset_count asset(s)..."
                if gh release download "$old_tag" --repo "$REPO" --dir "$TEMP_DIR" 2>/dev/null; then
                    for file in "$TEMP_DIR"/*; do
                        [[ -f "$file" ]] && asset_paths+=("$file")
                    done
                fi
            fi
            
            # タイトル調整
            if [[ "$release_title" == "$old_tag" ]]; then
                release_title="$new_tag"
            fi
            
            # 新しいリリース作成
            release_args=(
                "$new_tag"
                --repo "$REPO"
                --title "$release_title"
                --notes "$release_body"
            )
            
            [[ "$is_prerelease" == "true" ]] && release_args+=(--prerelease)
            [[ "$is_draft" == "true" ]] && release_args+=(--draft)
            
            for asset in "${asset_paths[@]}"; do
                release_args+=("$asset")
            done
            
            if gh release create "${release_args[@]}" 2>/dev/null; then
                gh release delete "$old_tag" --repo "$REPO" --yes 2>/dev/null
                echo -e "  ${GREEN}✓${NC} Release migrated"
            else
                echo -e "  ${YELLOW}⚠${NC} Release creation failed"
            fi
            
            rm -f "$TEMP_DIR"/*
        fi
    fi
    
    # 古いタグを削除
    git tag -d "$old_tag" 2>/dev/null
    git push origin ":refs/tags/$old_tag" 2>/dev/null
    
    echo ""
    ((processed++))
done <<< "$tags_to_migrate"

echo -e "${GREEN}=== Migration Complete ===${NC}"
echo ""
echo "Statistics:"
echo "  Processed: $processed"
echo "  Skipped:   $skipped"
echo "  Failed:    $failed"
echo ""
echo "All tags now have 'v' prefix (v0.1.0, v0.2.0, ..., v0.28.0)"

