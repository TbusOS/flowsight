#!/bin/bash
set -e

echo "=========================================="
echo "FlowSight Development Environment Setup"
echo "=========================================="
echo ""

# Check if running on Linux
if [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: This script is designed for Linux systems"
    exit 1
fi

# Install Rust
echo "[1/4] Installing Rust..."
if ! command -v rustc &> /dev/null; then
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed successfully"
else
    echo "✓ Rust already installed ($(rustc --version))"
fi

# Install Node.js (if not present)
echo ""
echo "[2/4] Checking Node.js..."
if ! command -v node &> /dev/null; then
    echo "Error: Node.js not found. Please install Node.js 20+ first"
    echo "Visit: https://nodejs.org/"
    exit 1
else
    echo "✓ Node.js found ($(node --version))"
fi

# Install pnpm
echo ""
echo "[3/4] Installing pnpm..."
if ! command -v pnpm &> /dev/null; then
    npm install -g pnpm
    echo "✓ pnpm installed successfully"
else
    echo "✓ pnpm already installed ($(pnpm --version))"
fi

# Install system dependencies for Tauri
echo ""
echo "[4/4] Installing system dependencies..."
echo "This step requires sudo privileges"
sudo apt-get update
sudo apt-get install -y \
    libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libxdo-dev \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev

echo ""
echo "=========================================="
echo "Installing project dependencies..."
echo "=========================================="

# Load Rust environment
source "$HOME/.cargo/env"

# Install frontend dependencies
echo ""
echo "Installing frontend dependencies..."
cd app
pnpm install
cd ..

# Build Rust workspace
echo ""
echo "Building Rust workspace..."
cargo build --workspace

echo ""
echo "=========================================="
echo "✓ Setup completed successfully!"
echo "=========================================="
echo ""
echo "To start development:"
echo "  cd app"
echo "  pnpm tauri dev"
echo ""
