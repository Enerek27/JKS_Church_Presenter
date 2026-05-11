#!/usr/bin/env bash
set -e

########################################
# Konfigurácia (musí sedieť s install.sh)
########################################
APP_NAME="JKS Presenter"
EXEC_NAME="jks_premietac"

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXEC_PATH="$PROJECT_DIR/target/release/$EXEC_NAME"
DESKTOP_FILE="$HOME/.local/share/applications/jks-presenter.desktop"

echo "==> Odinštalovávam $APP_NAME"

########################################
# 1. Odstránenie build artefaktov
########################################
if [ -d "$PROJECT_DIR/target" ]; then
  echo "==> Mažem build artefakty (cargo clean)..."
  cd "$PROJECT_DIR"
  cargo clean || rm -rf "$PROJECT_DIR/target"
else
  echo "==> Žiadny target/ adresár, preskakujem cargo clean."
fi

########################################
# 2. jks.db pri binárke
########################################
if [ -f "$PROJECT_DIR/target/release/jks.db" ]; then
  echo "==> Mažem databázu pri binárke: $PROJECT_DIR/target/release/jks.db"
  rm -f "$PROJECT_DIR/target/release/jks.db"
else
  echo "==> Nenašiel som jks.db pri binárke, preskakujem."
fi

########################################
# 3. .desktop launcher v ~/.local/share/applications
########################################
if [ -f "$DESKTOP_FILE" ]; then
  echo "==> Mažem launcher: $DESKTOP_FILE"
  rm -f "$DESKTOP_FILE"
else
  echo "==> Launcher v ~/.local/share/applications som nenašiel."
fi

########################################
# 4. Skratka na ploche (ak existuje)
########################################
if [ -d "$HOME/Desktop" ]; then
  DESKTOP_LINK="$HOME/Desktop/jks-presenter.desktop"
  if [ -f "$DESKTOP_LINK" ]; then
    echo "==> Mažem skratku na ploche: $DESKTOP_LINK"
    rm -f "$DESKTOP_LINK"
  else
    echo "==> Skratku na ploche som nenašiel."
  fi
fi

echo "==> Odinštalovanie $APP_NAME dokončené."
echo "Projektové súbory (zdrojáky, jks.db v koreňovom priečinku) zostali zachované."