#!/bin/bash
# PreToolUse(Bash): before any `git commit`, enforce the asset policy on what is
# staged. Exit 2 blocks the tool call and shows stderr to Claude.
input=$(cat)
cmd=$(printf '%s' "$input" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("tool_input",{}).get("command",""))' 2>/dev/null)
case "$cmd" in
  *"git commit"*) ;;
  *) exit 0 ;;
esac
[ -f "$HOME/.cargo/env" ] && . "$HOME/.cargo/env"
cd "${CLAUDE_PROJECT_DIR:-.}" || exit 0
if ! out=$(cargo xtask policy --staged 2>&1); then
  printf 'Asset policy check failed; commit blocked. Unstage or fix these files:\n%s\n' "$out" >&2
  exit 2
fi
exit 0
