#!/bin/bash
# EC2 Setup Script for poker-solver v1.1
# Run this inside EC2 after SSH

set -e

echo "========================================="
echo "  Poker Solver v1.1 - EC2 Setup"
echo "========================================="
echo "Start time: $(date)"
echo

# Install dependencies
echo "[1/5] Installing dependencies..."
sudo dnf install -y git gcc > /dev/null 2>&1
echo "  ✓ git and gcc installed"
echo

# Install Rust
echo "[2/5] Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y > /dev/null 2>&1
source $HOME/.cargo/env
echo "  ✓ Rust installed: $(rustc --version)"
echo

# Clone repository
echo "[3/5] Cloning repository..."
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver
echo "  ✓ Repository cloned"
echo

# Download ranges from S3
echo "[4/5] Downloading ranges from S3..."
aws s3 sync s3://poker-solver-kason/v1.1/ranges/ ranges/
echo "  ✓ Ranges downloaded"
echo

# Build solver
echo "[5/5] Building solver..."
cd solver
cargo build --release --bin poker_solver
cd ..
echo "  ✓ Solver built"
echo

echo "========================================="
echo "  ✓ Setup Complete!"
echo "========================================="
echo
echo "To run the solve:"
echo "  time ./solver/target/release/poker_solver | tee solve_output.txt"
echo
echo "Expected: ~25-40 minutes"
echo
