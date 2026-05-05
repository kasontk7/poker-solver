#!/bin/bash
# EC2 User Data Script - Runs automatically on instance launch
# Installs everything, runs solve, uploads to S3, then terminates

set -e

BOARD="${BOARD:-KhQs6h}"  # Default board
S3_BUCKET="poker-solver-solutions"

echo "=== Poker Solver EC2 Auto-Setup ==="
echo "Board: $BOARD"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install git
sudo dnf install -y git

# Clone repo
cd /home/ec2-user
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver

# Build solver
cd solver
cargo build --release --bin poker_solver

# Run solve (with timing)
cd /home/ec2-user/poker-solver
echo "Starting solve at $(date)"
time ./solver/target/release/poker_solver 2>&1 | tee solve_output.txt
echo "Solve completed at $(date)"

# Upload to S3
echo "Uploading to S3..."
aws s3 cp solutions/v1.1_${BOARD}.bin s3://${S3_BUCKET}/v1.1_${BOARD}.bin
aws s3 cp solve_output.txt s3://${S3_BUCKET}/logs/v1.1_${BOARD}_log.txt

echo "Upload complete!"

# Terminate instance (auto-cleanup)
INSTANCE_ID=$(ec2-metadata --instance-id | cut -d ' ' -f 2)
aws ec2 terminate-instances --instance-ids $INSTANCE_ID

echo "Instance terminating..."
