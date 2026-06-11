#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
RUN_SCRIPT="$PROJECT_DIR/run.sh"

if [ ! -f "$RUN_SCRIPT" ]; then
    echo "⚫️ Error: run.sh was not found."
    exit 1
fi

ALIAS_LINE="alias zerkalo='\"$RUN_SCRIPT\"'"
TARGET_RC=""

if [ -n "$ZSH_VERSION" ] || [ "$(basename "${SHELL:-}")" = "zsh" ]; then
    TARGET_RC="$HOME/.zshrc"
elif [ -n "$BASH_VERSION" ] || [ "$(basename "${SHELL:-}")" = "bash" ]; then
    TARGET_RC="$HOME/.bashrc"
else
    TARGET_RC="$HOME/.zshrc"
fi

touch "$TARGET_RC"

if grep -Fq "$ALIAS_LINE" "$TARGET_RC"; then
    echo "⚪️ Alias 'zerkalo' is already configured in $TARGET_RC"
else
    printf "\n%s\n" "$ALIAS_LINE" >> "$TARGET_RC"
    echo "⚪️ Alias 'zerkalo' was added to $TARGET_RC"
fi

echo "⚪️ Restart your terminal or run: source $TARGET_RC"
