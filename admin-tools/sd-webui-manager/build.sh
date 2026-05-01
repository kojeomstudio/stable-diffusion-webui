#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "=== SD WebUI Manager Build ==="
echo ""

command -v rustc >/dev/null 2>&1 || { echo "Error: rustc not found. Install from https://rustup.rs"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "Error: node not found. Install from https://nodejs.org"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found. Install from https://rustup.rs"; exit 1; }

echo "[1/3] Installing npm dependencies..."
cd "$SCRIPT_DIR" && npm install

echo "[2/3] Building Tauri application..."
cd "$SCRIPT_DIR" && npm run build

echo "[3/3] Done!"
echo ""
echo "Build artifacts located in: $SCRIPT_DIR/src-tauri/target/release/bundle/"
