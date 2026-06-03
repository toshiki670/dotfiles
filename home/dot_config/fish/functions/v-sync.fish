function v-sync --description 'Sync nvim plugins (Lazy sync) and re-add lazy-lock.json into chezmoi source'
    if not command -q nvim
        echo "v-sync: nvim command not found." >&2
        return 127
    end

    if not command -q chezmoi
        echo "v-sync: chezmoi command not found." >&2
        return 127
    end

    echo "v-sync: syncing nvim plugins..."
    command nvim --headless "+Lazy! sync" +qa
    if test $status -ne 0
        echo "v-sync: nvim plugin sync failed." >&2
        return $status
    end

    echo "v-sync: re-adding lazy-lock.json into chezmoi source..."
    command chezmoi re-add "$HOME/.config/nvim/lazy-lock.json"
end
