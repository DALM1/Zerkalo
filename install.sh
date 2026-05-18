#!/bin/bash

# Zerkalo Installation Script
# Ce script compile le projet et fournit les instructions pour les permissions.

set -e

echo "⚪️ Initialisation de Zerkalo..."

# Vérification de l'installation de Rust
if ! command -v cargo &> /dev/null
then
    echo "⚫️ Erreur: Rust/Cargo n'est pas installé. Veuillez l'installer via https://rustup.rs/"
    exit 1
fi

echo "⚪️ Compilation du projet en mode release..."
cargo build --release

BINARY_PATH="target/release/zerkalo"

if [ -f "$BINARY_PATH" ]; then
    echo "⚪️ Compilation réussie"
    echo ""
    echo "⚪️ IMPORTANT : macOS nécessite des permissions spéciales pour intercepter le clavier."
    echo "1. Allez dans Réglages Système > Confidentialité et sécurité > Accessibilité."
    echo "2. Ajoutez et activez votre Terminal (ou l'application qui lancera Zerkalo)."
    echo ""
    echo "⚪️ Raccourci : Cmd + Alt + C pour activer/désactiver la translittération."
    echo ""

    read -p "⚪️ Voulez-vous lancer Zerkalo directement ? (y/n) " choice
    case "$choice" in
      y|Y )
        ./run.sh
        ;;
      * )
        echo "⚪️ Installation terminée. Vous pourrez lancer le programme plus tard avec ./run.sh"
        ;;
    esac
else
    echo "⚫️ Erreur: Échec de la compilation."
    exit 1
fi
