#!/bin/bash

set -e

BIN_NAME="scurl"
SRC="dist/scurl-macos"
DEST="/usr/local/bin/$BIN_NAME"

# Ensure the binary exists
if [ ! -f "$SRC" ]; then
  echo "❌ Binary not found at $SRC"
  exit 1
fi

# Copy binary to /usr/local/bin
echo "[*] Installing $BIN_NAME to $DEST..."
sudo cp "$SRC" "$DEST"
sudo chmod +x "$DEST"

# Confirm installation
echo "[✓] $BIN_NAME installed successfully!"
echo "[*] Version check:"
"$BIN_NAME" --version
