# bat - cat command replacement
if type "bat" > /dev/null 2>&1; then
  alias cat='bat'
  alias scat='command cat'
fi
