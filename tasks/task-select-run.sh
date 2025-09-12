#!/bin/bash

set -eu

# Check required commands
for cmd in task fzf; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    printf "[ERROR] Required command '%s' not found. Please install it.\n" "$cmd" >&2
    exit 1
  fi
done

# List tasks and let user select one
selected=$(task list | grep -E '^\* ' | fzf --reverse | sed 's/^\* //' | sed 's/:[[:space:]].*$//')

# Run the selected task
if [ -n "$selected" ]; then
  echo "Run task: [$selected]"
  task "$selected"
else
  echo "No task selected."
fi
