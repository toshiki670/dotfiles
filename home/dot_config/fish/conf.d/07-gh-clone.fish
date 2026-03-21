function gh-clone --description 'Clone repository via gh and migrate with ghq'
  if not command -q gh
    echo "gh-clone: gh command not found." >&2
    return 127
  end

  if not command -q ghq
    echo "gh-clone: ghq command not found." >&2
    return 127
  end

  if test (count $argv) -lt 1
    echo "Usage: gh-clone <owner/repo>" >&2
    return 1
  end

  set -l repo_spec "$argv[1]"

  command gh repo clone "$repo_spec"
  if test $status -ne 0
    return $status
  end

  set -l repo_dir (path basename -- "$repo_spec")
  set -l migrated_path (command ghq migrate -y "$repo_dir")
  if test $status -ne 0
    return $status
  end

  cd "$migrated_path"
end
