#!/bin/bash

set -e

PROJECT_DIR="$(cd "$(dirname "$0")" && pwd)"
INSTALLED_BINARY="$HOME/Applications/Zerkalo.app/Contents/MacOS/zerkalo"
RUN_SCRIPT="$PROJECT_DIR/run.sh"

if [ -f "$INSTALLED_BINARY" ]; then
    ALIAS_TARGET="$INSTALLED_BINARY"
elif [ -f "$RUN_SCRIPT" ]; then
    ALIAS_TARGET="$RUN_SCRIPT"
else
    echo "⚫️ Error: neither the installed binary nor run.sh was found."
    exit 1
fi

ALIAS_LINE="alias zerkalo='\"$ALIAS_TARGET\"'"
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
