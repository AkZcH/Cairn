#!/usr/bin/env bash
set -euo pipefail

REPO="AkZcH/Cairn"
INSTALL_DIR="$HOME/.local/bin"
MODEL_DIR="$HOME/.local/share/cairn/models/bge-small-en-v1.5"

echo "Installing cairn-ingest..."
mkdir -p "$INSTALL_DIR"

LATEST_URL="https://github.com/$REPO/releases/latest/download/cairn-ingest-linux-x86_64"
curl -fsSL -o "$INSTALL_DIR/cairn-ingest" "$LATEST_URL"
chmod +x "$INSTALL_DIR/cairn-ingest"

echo "Downloading embedding model (BAAI/bge-small-en-v1.5, ~130MB)..."
mkdir -p "$MODEL_DIR"
BASE_URL="https://huggingface.co/BAAI/bge-small-en-v1.5/resolve/main"
curl -fsSL -o "$MODEL_DIR/config.json" "$BASE_URL/config.json"
curl -fsSL -o "$MODEL_DIR/tokenizer.json" "$BASE_URL/tokenizer.json"
curl -fsSL -o "$MODEL_DIR/model.safetensors" "$BASE_URL/model.safetensors"

echo ""
echo "Done. cairn-ingest installed to $INSTALL_DIR"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
    echo ""
    echo "NOTE: $INSTALL_DIR is not on your PATH. Add this to your shell config:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
fi

echo ""
echo "Next: set up the backend (Postgres + API):"
echo "  curl -O https://raw.githubusercontent.com/$REPO/main/infra/docker-compose.prod.yml"
echo "  curl -O https://raw.githubusercontent.com/$REPO/main/infra/.env.example"
echo "  mv .env.example .env   # then edit in your GROQ_API_KEY"
echo "  docker compose -f docker-compose.prod.yml up -d"