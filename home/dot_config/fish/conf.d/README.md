# fish conf.d numbering rule

`conf.d` files are loaded in lexical order.
We use 2-digit prefixes where each 20-number range maps to one responsibility.

## Number ranges

- `00-19`: Core env (`PATH`, env vars, runtime activation)
- `20-39`: Interactive UX (bindings, abbreviations, input behavior)
- `40-59`: Command behavior (wrappers, command overrides)
- `60-79`: Background/status (startup jobs, status cache/update)
- `80-99`: Prompt/final (theme, prompt, final-stage setup)

## Naming rule

- Format: `NN-feature-name.fish`
- Keep names descriptive (`95-starship-prompt.fish`, `65-ci-status.fish`)

## Ordering rule

- Keep strict ordering only when there is a real dependency.
- Prefer gaps inside a range to keep insertion easy (e.g. `20`, `25`, `30`).
- Use filename order as the final tie-breaker when needed.
