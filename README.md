# 🪞 Zerkalo

**Zerkalo** (Зеркало - *Miroir*) est un démon macOS léger écrit en Rust qui permet une translittération phonétique du latin vers le cyrillique en temps réel à l'échelle du système.

## 🚀 Fonctionnalités

- **Système global** : Fonctionne dans toutes les applications macOS (Navigateurs, IDE, Messageries).
- **Zéro Latence** : Interception directe via Quartz Event Tap pour une réactivité maximale.
- **Mapping Phonétique Intelligent** : Supporte les caractères simples et les séquences complexes (ex: `sh` -> `ш`, `shch` -> `щ`).
- **Contrôle Total** : Activez ou désactivez la translittération instantanément avec un raccourci clavier.
- **Léger** : Utilisation CPU et mémoire minimale.

## 🛠 Installation

1. Assurez-vous d'avoir [Rust](https://rustup.rs/) installé sur votre Mac.
2. Clonez le dépôt et entrez dans le dossier.
3. Lancez le script d'installation :
   ```bash
   ./install.sh
   ```

## 🔐 Permissions macOS

Pour fonctionner, **Zerkalo** a besoin de l'accès à l'**Accessibilité** pour intercepter les touches du clavier :

1. Allez dans **Réglages Système** > **Confidentialité et sécurité** > **Accessibilité**.
2. Ajoutez et activez votre **Terminal** (ou l'application qui lancera Zerkalo).

## ⌨️ Utilisation

Lancez le programme via le script de lancement :
```bash
./run.sh
```

- **Activer / Désactiver** : `Cmd` + `Alt` + `C`
- **Quitter** : `Ctrl` + `C` dans le terminal.

## 📋 Table de Translittération

Le mapping suit une logique phonétique standard. Pour plus de détails, consultez la [TRANSLATION_TABLE.md](./TRANSLATION_TABLE.md).

Exemples :
- `p` -> `п`
- `r` -> `р`
- `i` -> `и`
- `v` -> `в`
- `e` -> `е`
- `t` -> `т`
- Résultat : `privet` -> `привет`
