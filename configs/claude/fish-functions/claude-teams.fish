function claude-teams --wraps claude --description 'claude with agent teams enabled'
    env CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 claude $argv
end
