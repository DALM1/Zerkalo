#!/bin/bash

# Zerkalo Launch Script

BINARY_PATH="target/release/zerkalo"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Lancement de Zerkalo..."
    ./$BINARY_PATH
else
    echo "⚫️ Erreur: Le binaire n'existe pas. Veuillez lancer ./install.sh d'abord."
    exit 1
fi
