#!/bin/bash
# Batch Solve All Boards
# Orchestrates 6,992 EC2 spot instances

set -e

BOARDS_FILE="${1:-boards.txt}"
MAX_PARALLEL="${2:-50}"  # Max concurrent instances
S3_BUCKET="poker-solver-solutions"

if [ ! -f "$BOARDS_FILE" ]; then
    echo "Error: Boards file not found: $BOARDS_FILE"
    echo "Usage: ./batch-solve.sh <boards-file> [max-parallel]"
    exit 1
fi

TOTAL=$(wc -l < "$BOARDS_FILE")
echo "=== Batch Poker Solver ==="
echo "Total boards: $TOTAL"
echo "Max parallel: $MAX_PARALLEL"
echo "S3 bucket: $S3_BUCKET"
echo ""

# Track progress
COMPLETED=0
FAILED=0
RUNNING=0

# Function to launch one solve
launch_solve() {
    local board=$1
    echo "[$(date +%H:%M:%S)] Launching: $board"

    ./spot-launch.sh "$board" > "logs/${board}.launch.log" 2>&1

    if [ $? -eq 0 ]; then
        echo "  ✓ Launched"
    else
        echo "  ✗ Failed to launch"
        echo "$board" >> failed_launches.txt
    fi
}

# Create logs directory
mkdir -p logs

# Process boards
while IFS= read -r board; do
    # Wait if we're at max parallel
    while true; do
        RUNNING=$(aws ec2 describe-instances \
            --filters "Name=tag:Name,Values=poker-solver-*" "Name=instance-state-name,Values=pending,running" \
            --profile poker \
            --query 'Reservations[*].Instances[*].InstanceId' \
            --output text | wc -w)

        if [ "$RUNNING" -lt "$MAX_PARALLEL" ]; then
            break
        fi

        echo "[$RUNNING/$MAX_PARALLEL running] Waiting 30s..."
        sleep 30
    done

    # Launch solve
    launch_solve "$board" &

    # Brief delay to avoid API throttling
    sleep 2

done < "$BOARDS_FILE"

echo ""
echo "All boards queued!"
echo "Monitor progress: watch -n 30 './check-progress.sh'"
echo "Download all: ./download-all.sh"
