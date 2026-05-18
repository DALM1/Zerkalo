#!/bin/bash

# Zerkalo Installation Script
# This script compiles the project and provides instructions for permissions.

set -e

echo "⚪️ Initialisation de Zerkalo..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null
then
    echo "⚫️ Erreur: Rust/Cargo n'est pas installé. Veuillez l'installer via https://rustup.rs/"
    exit 1
fi

echo "⚪️ Compilation du projet en mode release..."
cargo build --release

BINARY_PATH="target/release/zerkalo"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Compilation réussie "
    echo ""
    echo "⚠️  IMPORTANT : macOS nécessite des permissions spéciales pour intercepter le clavier."
    echo "1. Allez dans Réglages Système > Confidentialité et sécurité > Accessibilité."
    echo "2. Ajoutez et activez votre Terminal (ou l'application qui lancera Zerkalo)."
    echo ""
    echo "⚪️ Pour lancer Zerkalo, utilisez la commande suivante :"
    echo "   ./$BINARY_PATH"
    echo ""
    echo "⚪️ Raccourci : Cmd + Alt + C pour activer/désactiver la translittération."
else
    echo "⚫️ Erreur: Échec de la compilation."
    exit 1
fi
