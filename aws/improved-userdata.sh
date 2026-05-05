#!/bin/bash
# Improved EC2 User Data - Handles failures & tracking

set -e

# Board is passed via instance tag
INSTANCE_ID=$(ec2-metadata --instance-id | cut -d' ' -f2)
BOARD=$(aws ec2 describe-tags --filters "Name=resource-id,Values=${INSTANCE_ID}" "Name=key,Values=Board" --query 'Tags[0].Value' --output text 2>/dev/null || echo "KhQs6h")

S3_BUCKET="poker-solver-solutions"
STATUS_FILE="s3://${S3_BUCKET}/status/${BOARD}.json"

SPOT_ACTION=$(curl -s http://169.254.169.254/latest/meta-data/spot/instance-action || echo "none")

echo "=== Poker Solver Auto-Solve ==="
echo "Board: $BOARD"
echo "Instance: $INSTANCE_ID"

# Function: Update status in S3
update_status() {
    local status=$1
    local message=$2

    cat > /tmp/status.json <<EOF
{
  "board": "$BOARD",
  "instance_id": "$INSTANCE_ID",
  "status": "$status",
  "message": "$message",
  "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "spot_action": "$SPOT_ACTION"
}
EOF

    aws s3 cp /tmp/status.json "$STATUS_FILE"
}

# Mark as started
update_status "started" "Installing dependencies"

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# Install git
sudo dnf install -y git

# Clone repo
update_status "cloning" "Cloning repository"
cd /home/ec2-user
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver

# Build parameterized solver
update_status "building" "Building solver"
cd solver
cargo build --release --bin poker_solver_parameterized

# Check for spot termination warning every 60s in background
(
    while true; do
        ACTION=$(curl -s http://169.254.169.254/latest/meta-data/spot/instance-action || echo "none")
        if [ "$ACTION" != "none" ]; then
            update_status "interrupted" "Spot instance terminating in 2 minutes"
            exit 1
        fi
        sleep 60
    done
) &
MONITOR_PID=$!

# Run solve with board parameter
update_status "solving" "Running CFR solver"
cd /home/ec2-user/poker-solver

if time ./solver/target/release/poker_solver_parameterized "$BOARD" 2>&1 | tee solve_output.txt; then
    update_status "solved" "Solve complete, uploading"

    # Upload solution
    aws s3 cp solutions/v1.1_${BOARD}.bin s3://${S3_BUCKET}/v1.1_${BOARD}.bin
    aws s3 cp solve_output.txt s3://${S3_BUCKET}/logs/v1.1_${BOARD}_log.txt

    # Mark complete
    update_status "complete" "Upload successful"

    # Stop monitoring
    kill $MONITOR_PID 2>/dev/null || true

    # Terminate
    aws ec2 terminate-instances --instance-ids $INSTANCE_ID
else
    # Solve failed
    update_status "failed" "Solver crashed or OOM"

    # Upload logs for debugging
    aws s3 cp solve_output.txt s3://${S3_BUCKET}/logs/FAILED_${BOARD}_log.txt

    # Don't terminate - leave for investigation
    echo "Solve failed. Instance left running for debugging."
fi
