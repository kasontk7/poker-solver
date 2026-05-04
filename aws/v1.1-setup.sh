#!/bin/bash
# v1.1 EC2 Setup Script
# Run this inside EC2 instance to solve one flop with full tree

set -e

echo "========================================="
echo "  Poker Solver v1.1 - AWS Setup"
echo "========================================="
echo "Start time: $(date)"
echo

# Step 1: Install Rust
echo "[1/6] Installing Rust..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustc --version
echo

# Step 2: Clone repository
echo "[2/6] Cloning repository..."
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver
echo

# Step 3: Download ranges from S3
echo "[3/6] Downloading ranges from S3..."
aws s3 sync s3://poker-solver-kason/v1.1/ranges/ ranges/
ls -lh ranges/gto/BTN/
ls -lh ranges/gto/BB/
echo

# Step 4: Compile solver
echo "[4/6] Compiling solver (this takes ~5 minutes)..."
cd solver
time cargo build --release --bin poker_solver
ls -lh target/release/poker_solver
echo

# Step 5: Run solver
echo "[5/6] Running solver (this takes ~30 minutes)..."
echo "Board: KhQs6h (full tree)"
echo "Memory: Expected ~30 GB compressed"
echo
cd ..
time ./solver/target/release/poker_solver | tee solve_output.txt
echo

# Step 6: Upload results to S3
echo "[6/6] Uploading results to S3..."
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
aws s3 cp solve_output.txt s3://poker-solver-kason/v1.1/results/output_${TIMESTAMP}.txt
aws s3 cp solutions/ s3://poker-solver-kason/v1.1/results/solutions_${TIMESTAMP}/ --recursive

echo
echo "========================================="
echo "  ✓ Complete!"
echo "========================================="
echo "End time: $(date)"
echo
echo "Results uploaded to:"
echo "  s3://poker-solver-kason/v1.1/results/output_${TIMESTAMP}.txt"
echo "  s3://poker-solver-kason/v1.1/results/solutions_${TIMESTAMP}/"
echo
echo "To terminate this instance:"
echo "  aws ec2 terminate-instances --instance-ids \$(ec2-metadata --instance-id | cut -d ' ' -f 2)"
