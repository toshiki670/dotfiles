function gcm --description 'AI-powered git commit with Conventional Commits'
    # Verify staged changes exist
    set -l staged_files (git diff --cached --name-only 2>/dev/null)
    if test (count $staged_files) -eq 0
        echo "ステージされた変更がありません。" >&2
        echo "git add でファイルをステージしてから実行してください。" >&2
        return 1
    end

    if not command -q jq
        echo "jq が見つかりません: brew install jq" >&2
        return 1
    end

    set -l staged_str (string join ", " $staged_files)
    set -l diff_output (git diff --cached 2>/dev/null)

    set -l system_prompt "You are a git commit message generator using Conventional Commits.

Analyze the staged diff and split into the minimum number of semantically independent commits.
- Single concern → one entry
- Multiple independent concerns (e.g. feat + fix, or unrelated files) → multiple entries

Output ONLY a JSON array — no markdown fences, no extra text:
[{\"message\": \"<type>[scope]: <description>\", \"files\": [\"path/to/file\"]}, ...]

Rules:
- type: feat, fix, docs, style, refactor, test, chore, perf, ci, build
- description: English, imperative mood (add, fix, update, remove, ...)
- Every staged file must appear in exactly one entry's files array"

    set -l conversation "Propose commits for the following staged changes.

Staged files: $staged_str

git diff --staged:
$diff_output"

    # Generate initial proposal
    echo "コミットを生成中..." >&2
    set -l proposal_json (_gcm_call_claude "$system_prompt" "$conversation")
    if test $status -ne 0
        return 1
    end

    # Interactive approval loop (Ctrl-C terminates the function immediately)
    while true
        _gcm_display "$proposal_json"
        or return 1

        read -P "追加指示 (Enter でコミット実行 / Ctrl-C で中止): " -l instruction
        if test $status -ne 0
            echo ""
            echo "中止しました。"
            return 0
        end

        if test -z "$instruction"
            _gcm_execute "$proposal_json"
            return $status
        end

        set conversation "$conversation

Previous proposal (JSON): $proposal_json
Revision instruction: $instruction
Revise accordingly. Output ONLY the JSON array."

        echo "修正中..." >&2
        set proposal_json (_gcm_call_claude "$system_prompt" "$conversation")
        if test $status -ne 0
            return 1
        end
    end
end

function _gcm_call_claude
    set -l system_prompt $argv[1]
    set -l conversation $argv[2]

    set -l tmpfile (mktemp)
    printf '%s\n' "$conversation" >$tmpfile
    set -l out (claude -p --system-prompt "$system_prompt" < $tmpfile 2>/dev/null)
    rm -f $tmpfile

    if test -z "$out"
        echo "生成に失敗しました。claude コマンドが利用可能か確認してください。" >&2
        return 1
    end

    if not printf '%s' "$out" | jq '.' >/dev/null 2>&1
        echo "不正なJSON出力を受け取りました:" >&2
        printf '%s\n' "$out" >&2
        return 1
    end

    printf '%s' "$out"
end

function _gcm_display
    set -l proposal_json $argv[1]
    set -l count (printf '%s' "$proposal_json" | jq 'length')

    if not string match -qr '^[0-9]+$' "$count"; or test "$count" -eq 0
        echo "コミット提案が空です。" >&2
        return 1
    end

    echo ""
    if test "$count" -eq 1
        echo "提案されたコミット:"
    else
        echo "提案されたコミット ($count 件):"
    end

    for i in (seq 0 (math "$count - 1"))
        set -l msg (printf '%s' "$proposal_json" | jq -r ".[$i].message")
        set -l files (printf '%s' "$proposal_json" | jq -r ".[$i].files[]")
        echo ""
        set_color --bold cyan
        printf "  %d. %s\n" (math "$i + 1") "$msg"
        set_color normal
        for f in $files
            printf "     %s\n" "$f"
        end
    end
    echo ""
end

function _gcm_execute
    set -l proposal_json $argv[1]
    set -l count (printf '%s' "$proposal_json" | jq 'length')

    if test "$count" -eq 1
        set -l msg (printf '%s' "$proposal_json" | jq -r '.[0].message')
        git commit -m "$msg"
        return $status
    end

    # Multiple commits: unstage all, then stage and commit per entry
    git restore --staged .
    for i in (seq 0 (math "$count - 1"))
        set -l msg (printf '%s' "$proposal_json" | jq -r ".[$i].message")
        set -l files (printf '%s' "$proposal_json" | jq -r ".[$i].files[]")
        git add $files
        git commit -m "$msg"
        or begin
            echo "コミット $i 失敗。残りのファイルはステージされたままです。" >&2
            return 1
        end
    end
end
