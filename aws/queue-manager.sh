#!/bin/bash
# Smart Queue Manager - Auto-retries failed/interrupted boards
# Runs continuously until all 6,992 boards complete

set -e

BOARDS_FILE="${1:-boards.txt}"
MAX_PARALLEL="${2:-50}"
S3_BUCKET="poker-solver-solutions"
CHECK_INTERVAL=60  # Check for failures every 60 seconds

if [ ! -f "$BOARDS_FILE" ]; then
    echo "Error: Boards file not found: $BOARDS_FILE"
    exit 1
fi

TOTAL=$(wc -l < "$BOARDS_FILE")
QUEUE_DIR="queue_state"
mkdir -p "$QUEUE_DIR"

echo "╔════════════════════════════════════════╗"
echo "║   Smart Queue Manager with Auto-Retry  ║"
echo "╚════════════════════════════════════════╝"
echo ""
echo "Total boards: $TOTAL"
echo "Max parallel: $MAX_PARALLEL"
echo "Auto-retry:   ENABLED"
echo ""

# Initialize queue state
init_queue() {
    echo "Initializing queue..."

    > "$QUEUE_DIR/pending.txt"
    > "$QUEUE_DIR/running.txt"
    > "$QUEUE_DIR/complete.txt"
    > "$QUEUE_DIR/failed.txt"
    > "$QUEUE_DIR/retrying.txt"

    # Add all boards to pending
    cp "$BOARDS_FILE" "$QUEUE_DIR/pending.txt"

    echo "  Pending: $TOTAL boards"
}

# Check if board is complete
is_complete() {
    local board=$1

    # Check if solution exists in S3
    if aws s3 ls "s3://${S3_BUCKET}/v1.1_${board}.bin" --profile poker >/dev/null 2>&1; then
        return 0  # Complete
    fi

    return 1  # Not complete
}

# Check board status from S3
check_board_status() {
    local board=$1
    local status_file="/tmp/status_${board}.json"

    # Download status file
    if aws s3 cp "s3://${S3_BUCKET}/status/${board}.json" "$status_file" --profile poker >/dev/null 2>&1; then
        local status=$(jq -r '.status' "$status_file" 2>/dev/null || echo "unknown")
        local timestamp=$(jq -r '.timestamp' "$status_file" 2>/dev/null || echo "")

        # Check if stuck (running for >2 hours)
        if [ -n "$timestamp" ] && [ "$status" != "complete" ]; then
            local age_seconds=$(( $(date +%s) - $(date -d "$timestamp" +%s 2>/dev/null || echo 0) ))
            if [ "$age_seconds" -gt 7200 ]; then
                echo "stuck"
                rm -f "$status_file"
                return
            fi
        fi

        echo "$status"
        rm -f "$status_file"
    else
        echo "unknown"
    fi
}

# Launch a board solve
launch_board() {
    local board=$1

    echo "[$(date +%H:%M:%S)] Launching: $board"

    if ./spot-launch.sh "$board" > "$QUEUE_DIR/logs/${board}.launch.log" 2>&1; then
        echo "  ✓ Launched"
        # Move from pending to running
        grep -v "^${board}$" "$QUEUE_DIR/pending.txt" > "$QUEUE_DIR/pending.tmp" || true
        mv "$QUEUE_DIR/pending.tmp" "$QUEUE_DIR/pending.txt"
        echo "$board" >> "$QUEUE_DIR/running.txt"
        return 0
    else
        echo "  ✗ Launch failed"
        return 1
    fi
}

# Monitor running boards and detect failures
monitor_running() {
    local moved=0

    if [ ! -f "$QUEUE_DIR/running.txt" ] || [ ! -s "$QUEUE_DIR/running.txt" ]; then
        return 0
    fi

    while IFS= read -r board; do
        # Check if complete
        if is_complete "$board"; then
            echo "[$(date +%H:%M:%S)] ✅ Complete: $board"
            grep -v "^${board}$" "$QUEUE_DIR/running.txt" > "$QUEUE_DIR/running.tmp" || true
            mv "$QUEUE_DIR/running.tmp" "$QUEUE_DIR/running.txt"
            echo "$board" >> "$QUEUE_DIR/complete.txt"
            ((moved++))
            continue
        fi

        # Check status
        local status=$(check_board_status "$board")

        case "$status" in
            complete)
                echo "[$(date +%H:%M:%S)] ✅ Complete: $board"
                grep -v "^${board}$" "$QUEUE_DIR/running.txt" > "$QUEUE_DIR/running.tmp" || true
                mv "$QUEUE_DIR/running.tmp" "$QUEUE_DIR/running.txt"
                echo "$board" >> "$QUEUE_DIR/complete.txt"
                ((moved++))
                ;;

            failed|interrupted|stuck)
                echo "[$(date +%H:%M:%S)] 🔄 Auto-retry: $board (status: $status)"
                # Move back to pending for retry
                grep -v "^${board}$" "$QUEUE_DIR/running.txt" > "$QUEUE_DIR/running.tmp" || true
                mv "$QUEUE_DIR/running.tmp" "$QUEUE_DIR/running.txt"
                echo "$board" >> "$QUEUE_DIR/pending.txt"
                echo "$board:$status:$(date +%s)" >> "$QUEUE_DIR/retrying.txt"
                ((moved++))
                ;;

            started|building|solving)
                # Still running, leave it
                ;;

            unknown)
                # No status yet, might be launching or very early
                ;;
        esac

    done < "$QUEUE_DIR/running.txt"

    return $moved
}

# Main loop
main() {
    init_queue

    mkdir -p "$QUEUE_DIR/logs"

    local iteration=0
    local last_summary=$(date +%s)

    while true; do
        ((iteration++))

        # Monitor running boards
        monitor_running

        # Count current state
        local pending=$(wc -l < "$QUEUE_DIR/pending.txt" 2>/dev/null || echo 0)
        local running=$(wc -l < "$QUEUE_DIR/running.txt" 2>/dev/null || echo 0)
        local complete=$(wc -l < "$QUEUE_DIR/complete.txt" 2>/dev/null || echo 0)
        local retry_count=$(wc -l < "$QUEUE_DIR/retrying.txt" 2>/dev/null || echo 0)

        # Launch new boards if under limit
        while [ "$running" -lt "$MAX_PARALLEL" ] && [ "$pending" -gt 0 ]; do
            # Get next board from pending
            local board=$(head -1 "$QUEUE_DIR/pending.txt")

            if [ -z "$board" ]; then
                break
            fi

            # Launch it
            launch_board "$board" &

            # Brief delay to avoid API throttling
            sleep 2

            # Recount
            running=$(wc -l < "$QUEUE_DIR/running.txt" 2>/dev/null || echo 0)
            pending=$(wc -l < "$QUEUE_DIR/pending.txt" 2>/dev/null || echo 0)
        done

        # Print summary every 5 minutes
        local now=$(date +%s)
        if [ $((now - last_summary)) -gt 300 ]; then
            echo ""
            echo "╔════════════════════════════════════════╗"
            echo "║         Queue Status Summary           ║"
            echo "╚════════════════════════════════════════╝"
            echo "  Complete:  $complete / $TOTAL ($(echo "scale=1; $complete * 100 / $TOTAL" | bc)%)"
            echo "  Running:   $running"
            echo "  Pending:   $pending"
            echo "  Retried:   $retry_count boards"

            if [ "$complete" -gt 0 ]; then
                local elapsed=$((now - $(stat -f %B "$QUEUE_DIR/complete.txt" 2>/dev/null || echo $now)))
                if [ "$elapsed" -gt 0 ]; then
                    local rate=$(echo "scale=2; $complete * 3600 / $elapsed" | bc)
                    local remaining=$((TOTAL - complete))
                    local eta_hours=$(echo "scale=1; $remaining / $rate" | bc)
                    echo "  Rate:      ${rate} boards/hour"
                    echo "  ETA:       ${eta_hours} hours"
                fi
            fi

            echo ""
            last_summary=$now
        fi

        # Check if done
        if [ "$complete" -ge "$TOTAL" ]; then
            echo ""
            echo "╔════════════════════════════════════════╗"
            echo "║    🎉 ALL BOARDS COMPLETE! 🎉          ║"
            echo "╚════════════════════════════════════════╝"
            echo ""
            echo "Final stats:"
            echo "  Total solved: $complete"
            echo "  Total retries: $retry_count"
            echo ""

            # Show retry breakdown
            if [ "$retry_count" -gt 0 ]; then
                echo "Retry reasons:"
                awk -F: '{print "  " $2}' "$QUEUE_DIR/retrying.txt" | sort | uniq -c | sort -rn
            fi

            break
        fi

        # Wait before next check
        sleep "$CHECK_INTERVAL"
    done
}

# Handle Ctrl+C gracefully
trap 'echo ""; echo "Interrupted! Queue state saved in $QUEUE_DIR/"; echo "Resume with: ./queue-manager.sh $BOARDS_FILE $MAX_PARALLEL"; exit 0' INT

# Run main loop
main
