#!/usr/bin/env bash
set -e

########################################
# Konfigurácia
########################################
APP_NAME="Song Presenter"
EXEC_NAME="jks_premietac"

# koreňový priečinok projektu = kde leží tento skript
PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXEC_PATH="$PROJECT_DIR/target/release/$EXEC_NAME"
ICON_PATH="$PROJECT_DIR/icon.png"   # zmeň, ak máš inú ikonu
DESKTOP_FILE="$HOME/.local/share/applications/jks-presenter.desktop"

########################################
# 1. Systémové závislosti podľa distra
########################################
echo "==> Detekujem distribúciu a inštalujem systémové závislosti..."

if command -v apt >/dev/null 2>&1; then
  # Debian / Ubuntu / Mint ...
  sudo apt update
  sudo apt install -y build-essential pkg-config libsqlite3-dev sqlite3 curl

elif command -v dnf >/dev/null 2>&1; then
  # Fedora / RHEL / CentOS Stream
  sudo dnf install -y gcc make pkgconfig sqlite sqlite-devel curl

elif command -v pacman >/dev/null 2>&1; then
  # Arch / Manjaro
  sudo pacman -Sy --needed base-devel sqlite sqlite3 curl

elif command -v zypper >/dev/null 2>&1; then
  # openSUSE
  sudo zypper install -y gcc make pkg-config sqlite3 sqlite3-devel curl

else
  echo "!! Nepodporovaný balíčkovací systém."
  echo "   Nainštaluj ručne: kompilátor (gcc + make), sqlite3, sqlite3-devel, curl."
fi

########################################
# 2. Rust + Cargo
########################################
echo "==> Kontrolujem Rust + Cargo..."

if ! command -v cargo >/dev/null 2>&1; then
  echo "==> Inštalujem Rust + Cargo cez rustup..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  # načítaj Cargo do PATH pre aktuálny shell
  # POZOR: pri ďalších termináloch sa načíta z ~/.profile / ~/.bashrc
  source "$HOME/.cargo/env"
else
  echo "==> Cargo je už nainštalované."
fi

########################################
# 3. Build projektu
########################################
echo "==> Kompilujem projekt v release móde..."
cd "$PROJECT_DIR"
cargo build --release

if [ ! -x "$EXEC_PATH" ]; then
  echo "!! Build zlyhal alebo neexistuje binárka: $EXEC_PATH"
  exit 1
fi

########################################
# 3.1 Skopírovanie jks.db k binárke
########################################
if [ -f "$PROJECT_DIR/jks.db" ]; then
  echo "==> Kopírujem jks.db k binárke..."
  cp "$PROJECT_DIR/jks.db" "$PROJECT_DIR/target/release/jks.db"
else
  echo "!! Súbor jks.db sa nenašiel v $PROJECT_DIR, preskakujem kopírovanie."
fi

########################################
# 4. Vytvorenie .desktop spúšťača
########################################
echo "==> Vytváram .desktop spúšťač..."

mkdir -p "$HOME/.local/share/applications"

# Ikona – buď vlastná, alebo fallback na systémovú
if [ -f "$ICON_PATH" ]; then
  ICON_LINE="Icon=$ICON_PATH"
else
  ICON_LINE="Icon=utilities-terminal"
fi

cat > "$DESKTOP_FILE" <<EOF
[Desktop Entry]
Type=Application
Name=$APP_NAME
Comment=Premietač piesní JKS
Exec=$EXEC_PATH
Terminal=true
$ICON_LINE
Categories=Utility;
EOF

chmod +x "$DESKTOP_FILE"

########################################
# 5. (Voliteľné) skratka na plochu
########################################
if [ -d "$HOME/Desktop" ]; then
  read -r -p "Chceš pridať skratku aj na plochu? [y/N]: " ADD_DESKTOP

  if [ "$ADD_DESKTOP" = "y" ] || [ "$ADD_DESKTOP" = "Y" ]; then
    DESKTOP_LINK="$HOME/Desktop/jks-presenter.desktop"
    cp "$DESKTOP_FILE" "$DESKTOP_LINK"
    chmod +x "$DESKTOP_LINK"
    echo "==> Pridaná skratka na plochu: $DESKTOP_LINK"
  else
    echo "==> Skratka na plochu nebude vytvorená."
  fi
else
  echo "==> Priečinok ~/Desktop neexistuje, preskakujem vytváranie skratky na plochu."
fi
