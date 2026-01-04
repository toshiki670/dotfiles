#!/bin/bash
# migrate-to-semver-0x.sh
# 全既存タグとリリースを0.x.x形式に移行
# MINORは時系列順に連番、PATCHは元の番号を保持

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
    echo "Examples:"
    echo "  $0                    # Run migration with backup"
    echo "  $0 --dry-run          # Preview changes without applying"
    echo "  $0 --no-backup        # Run without creating backup"
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
    echo -e "${YELLOW}=== DRY RUN: Semantic Versioning Migration to 0.x.x ===${NC}\n"
    echo -e "${YELLOW}No changes will be made to tags or releases${NC}\n"
else
    echo -e "${YELLOW}=== Semantic Versioning Migration to 0.x.x ===${NC}\n"
fi

# 必要なコマンドの確認
for cmd in gh git jq; do
    if ! command -v $cmd >/dev/null 2>&1; then
        echo -e "${RED}Error: $cmd is required but not installed.${NC}" >&2
        exit 1
    fi
done

# 認証確認
if ! gh auth status >/dev/null 2>&1; then
    echo -e "${RED}Error: Not authenticated with GitHub CLI. Run 'gh auth login'${NC}"
    exit 1
fi

# 完全なバージョン対応表（時系列順、PATCH保持）
declare -A VERSION_MAP=(
    # 時系列順に0.1.0から開始、PATCHは保持
    ["1.0.0"]="0.1.0"
    ["1.1.0"]="0.2.0"
    ["1.1.1"]="0.2.1"
    ["1.2.0"]="0.3.0"
    ["1.2.1"]="0.3.1"
    ["1.2.2"]="0.3.2"
    ["1.2.3"]="0.3.3"
    ["1.3.0"]="0.4.0"
    ["1.4.0"]="0.5.0"
    ["1.5.0"]="0.6.0"
    ["1.5.1"]="0.6.1"
    ["1.5.2"]="0.6.2"
    ["1.6.0"]="0.7.0"
    ["1.6.1"]="0.7.1"
    ["1.6.2"]="0.7.2"
    ["1.6.3"]="0.7.3"
    ["1.6.4"]="0.7.4"
    ["1.6.5"]="0.7.5"
    ["1.6.6"]="0.7.6"
    ["1.6.7"]="0.7.7"
    ["1.7.0"]="0.8.0"
    ["1.7.1"]="0.8.1"
    ["2.0.0"]="0.9.0"
    ["2.1.0"]="0.10.0"
    ["2.1.1"]="0.10.1"
    ["2.2.0"]="0.11.0"
    ["2.2.1"]="0.11.1"
    ["2.2.2"]="0.11.2"
    ["2.3.0"]="0.12.0"
    ["3.0"]="0.13.0"
    ["4.0"]="0.14.0"
    ["5.0"]="0.15.0"
    ["6.0"]="0.16.0"
    ["7.0"]="0.17.0"
    ["8.0"]="0.18.0"
    ["8.1"]="0.18.1"
    ["8.2"]="0.18.2"
    ["8.3"]="0.18.3"
    ["8.4"]="0.18.4"
    ["9.0"]="0.19.0"
    ["v10.0"]="0.20.0"
    ["v11.0"]="0.21.0"
    ["v12.0"]="0.22.0"
    ["v13.0"]="0.23.0"
    ["v14.0"]="0.24.0"
    ["v14.1"]="0.24.1"
    ["v14.2"]="0.24.2"
    ["v15.0"]="0.25.0"
    ["v16.0"]="0.26.0"
    ["v17.0"]="0.27.0"
    ["v18.0"]="0.28.0"
)

echo -e "${BLUE}Current tags found:${NC}"
git tag -l | sort -V
echo ""

echo -e "${BLUE}Migration Strategy:${NC}"
echo "• Total tags to migrate: ${#VERSION_MAP[@]}"
echo "• MINOR: Sequential numbering (0.1.0 → 0.28.0)"
echo "• PATCH: Preserved from original version"
echo "• Latest: v18.0 → 0.28.0"
echo ""

if [[ "$DRY_RUN" == false ]]; then
    echo -e "${YELLOW}This script will:${NC}"
    echo "1. Create automatic backup"
    echo "2. Create new tags at the same commits"
    echo "3. Preserve all release notes and assets"
    echo "4. Delete old tags and releases"
    echo ""
    echo -e "${RED}WARNING: This is a one-way operation!${NC}"
    echo ""
    
    # バックアップ作成
    if [[ "$BACKUP_ENABLED" == true ]]; then
        echo -e "${BLUE}Creating backup...${NC}"
        mkdir -p "$BACKUP_DIR"
        
        # タグのバックアップ
        git tag -l > "$BACKUP_DIR/tags_backup_$(date +%Y%m%d_%H%M%S).txt"
        echo -e "${GREEN}✓ Tags backed up to $BACKUP_DIR${NC}"
        
        # リリースのバックアップ
        if gh release list --repo "$REPO" --limit 100 > "$BACKUP_DIR/releases_backup_$(date +%Y%m%d_%H%M%S).txt" 2>/dev/null; then
            echo -e "${GREEN}✓ Releases backed up to $BACKUP_DIR${NC}"
        fi
        
        # マッピング情報のバックアップ
        {
            echo "# Version Mapping"
            echo "# Old Tag -> New Tag"
            for old in "${!VERSION_MAP[@]}"; do
                echo "$old -> ${VERSION_MAP[$old]}"
            done
        } > "$BACKUP_DIR/mapping_$(date +%Y%m%d_%H%M%S).txt" | sort
        echo -e "${GREEN}✓ Mapping backed up to $BACKUP_DIR${NC}"
        echo ""
    fi
    
    read -p "Continue with migration? (yes/no): " confirm
    
    if [[ "$confirm" != "yes" ]]; then
        echo "Migration cancelled."
        exit 0
    fi
else
    echo -e "${BLUE}Dry run: showing planned operations...${NC}"
    echo ""
fi

echo ""
echo -e "${BLUE}Starting migration...${NC}"
echo ""

processed=0
skipped=0
failed=0

# タグを時系列順に処理（VERSION_MAPのキーをソート）
readarray -t sorted_tags < <(printf '%s\n' "${!VERSION_MAP[@]}" | sort -V)

for old_version in "${sorted_tags[@]}"; do
    new_version="${VERSION_MAP[$old_version]}"
    
    # タグが存在するか確認
    if ! git rev-parse "$old_version" >/dev/null 2>&1; then
        echo -e "${YELLOW}⊘ $old_version → $new_version (tag not found, skipping)${NC}"
        ((skipped++))
        continue
    fi
    
    commit_hash=$(git rev-parse "$old_version")
    commit_date=$(git log -1 --format=%ai "$old_version")
    
    echo -e "${GREEN}→ $old_version → $new_version${NC}"
    echo "  Commit: ${commit_hash:0:8} ($commit_date)"
    
    # ドライランモードの場合はここでスキップ
    if [[ "$DRY_RUN" == true ]]; then
        # リリースが存在するかチェック
        if gh release view "$old_version" --repo "$REPO" >/dev/null 2>&1; then
            echo "  [DRY RUN] Would migrate release"
        fi
        echo "  [DRY RUN] Would create tag $new_version"
        echo "  [DRY RUN] Would delete old tag $old_version"
        echo ""
        ((processed++))
        continue
    fi
    
    # タグメッセージを取得
    tag_message=$(git tag -l --format='%(contents)' "$old_version")
    
    # 新しいタグを作成
    if [[ -n "$tag_message" ]]; then
        git tag -a "$new_version" "$commit_hash" -m "$tag_message

---
Migrated from $old_version to $new_version (SemVer 0.x.x format)"
    else
        git tag -a "$new_version" "$commit_hash" -m "Version $new_version

Migrated from $old_version to semantic versioning 0.x.x format."
    fi
    
    # プッシュ
    if ! git push origin "$new_version" 2>/dev/null; then
        echo -e "${RED}  ✗ Failed to push tag${NC}"
        git tag -d "$new_version" 2>/dev/null
        ((failed++))
        continue
    fi
    
    # GitHubリリースを移行
    if gh release view "$old_version" --repo "$REPO" >/dev/null 2>&1; then
        echo "  Migrating release..."
        
        # リリース情報を取得
        release_json=$(gh release view "$old_version" --repo "$REPO" --json \
            name,body,isDraft,isPrerelease,createdAt,assets 2>/dev/null)
        
        if [[ $? -ne 0 ]]; then
            echo -e "${YELLOW}  Warning: Failed to get release info${NC}"
        else
            release_title=$(echo "$release_json" | jq -r '.name')
            release_body=$(echo "$release_json" | jq -r '.body')
            is_draft=$(echo "$release_json" | jq -r '.isDraft')
            asset_count=$(echo "$release_json" | jq -r '.assets | length')
            
            # アセットをダウンロード
            asset_paths=()
            if [[ $asset_count -gt 0 ]]; then
                echo "  Downloading $asset_count asset(s)..."
                if gh release download "$old_version" --repo "$REPO" --dir "$TEMP_DIR" 2>/dev/null; then
                    for file in "$TEMP_DIR"/*; do
                        if [[ -f "$file" ]]; then
                            asset_paths+=("$file")
                        fi
                    done
                fi
            fi
            
            # タイトルの調整
            if [[ "$release_title" == "$old_version" ]] || [[ "$release_title" == "v"* ]]; then
                release_title="$new_version"
            fi
            
            # 新しいリリースを作成
            release_args=(
                "$new_version"
                --repo "$REPO"
                --title "$release_title"
                --notes "$(echo "$release_body" | sed 's/\\n/\n/g')

---
**Migration:** \`$old_version\` → \`$new_version\` (SemVer 0.x.x - pre-release format)"
                --prerelease
            )
            
            if [[ "$is_draft" == "true" ]]; then
                release_args+=(--draft)
            fi
            
            for asset in "${asset_paths[@]}"; do
                release_args+=("$asset")
            done
            
            if gh release create "${release_args[@]}" 2>/dev/null; then
                # 古いリリースを削除
                gh release delete "$old_version" --repo "$REPO" --yes 2>/dev/null
                echo -e "  ${GREEN}✓${NC} Release migrated"
            else
                echo -e "  ${YELLOW}⚠${NC} Release creation failed (keeping old)"
            fi
            
            # クリーンアップ
            rm -f "$TEMP_DIR"/*
        fi
    fi
    
    # 古いタグを削除
    git tag -d "$old_version" 2>/dev/null
    git push origin ":refs/tags/$old_version" 2>/dev/null
    
    ((processed++))
done

echo ""
echo -e "${GREEN}=== Migration Complete ===${NC}"
echo ""
echo "Statistics:"
echo "  Processed: $processed"
echo "  Skipped:   $skipped"
echo "  Failed:    $failed"
echo ""
echo "Verify migration:"
echo "  ${BLUE}git tag -l | sort -V${NC}"
echo "  ${BLUE}gh release list --repo $REPO${NC}"
echo ""
echo "Update version file (current latest is 0.28.0):"
echo "  ${BLUE}echo '0.28.0' > version${NC}"
echo "  ${BLUE}git add version && git commit -m 'chore: update version to 0.28.0 (post-migration)'${NC}"
echo "  ${BLUE}git push${NC}"

