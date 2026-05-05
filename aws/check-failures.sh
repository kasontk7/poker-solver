#!/bin/bash
# Check for failed/interrupted solves and retry them

S3_BUCKET="poker-solver-solutions"

echo "=== Checking for Failed Solves ==="
echo ""

# Download all status files
mkdir -p status_cache
aws s3 sync s3://${S3_BUCKET}/status/ status_cache/ --profile poker --quiet

# Track failures
FAILED_COUNT=0
INTERRUPTED_COUNT=0
INCOMPLETE_COUNT=0

> failed_boards.txt
> interrupted_boards.txt
> incomplete_boards.txt

# Check each board from boards.txt
while IFS= read -r board; do
    STATUS_FILE="status_cache/${board}.json"

    if [ -f "$STATUS_FILE" ]; then
        STATUS=$(jq -r '.status' "$STATUS_FILE")
        MESSAGE=$(jq -r '.message' "$STATUS_FILE")

        case "$STATUS" in
            complete)
                # Success!
                ;;
            failed)
                echo "❌ FAILED: $board - $MESSAGE"
                echo "$board" >> failed_boards.txt
                ((FAILED_COUNT++))
                ;;
            interrupted)
                echo "⚠️  INTERRUPTED: $board - $MESSAGE"
                echo "$board" >> interrupted_boards.txt
                ((INTERRUPTED_COUNT++))
                ;;
            *)
                # Started but not complete (solving, building, etc)
                # Check if older than 2 hours (stuck?)
                TIMESTAMP=$(jq -r '.timestamp' "$STATUS_FILE")
                AGE_SECONDS=$(( $(date +%s) - $(date -d "$TIMESTAMP" +%s 2>/dev/null || echo 0) ))

                if [ "$AGE_SECONDS" -gt 7200 ]; then
                    echo "🕐 STUCK: $board - Status: $STATUS (${AGE_SECONDS}s ago)"
                    echo "$board" >> incomplete_boards.txt
                    ((INCOMPLETE_COUNT++))
                fi
                ;;
        esac
    else
        # No status file - never started or very old
        # Check if solution exists in S3
        if ! aws s3 ls s3://${S3_BUCKET}/v1.1_${board}.bin --profile poker >/dev/null 2>&1; then
            echo "❓ MISSING: $board - No status, no solution"
            echo "$board" >> incomplete_boards.txt
            ((INCOMPLETE_COUNT++))
        fi
    fi
done < boards.txt

echo ""
echo "=== Summary ==="
echo "Failed:       $FAILED_COUNT"
echo "Interrupted:  $INTERRUPTED_COUNT"
echo "Incomplete:   $INCOMPLETE_COUNT"
echo ""

TOTAL_RETRY=$((FAILED_COUNT + INTERRUPTED_COUNT + INCOMPLETE_COUNT))

if [ "$TOTAL_RETRY" -gt 0 ]; then
    echo "Retry with:"
    echo "  cat failed_boards.txt interrupted_boards.txt incomplete_boards.txt | while read board; do"
    echo "    ./spot-launch.sh \$board"
    echo "    sleep 5"
    echo "  done"
else
    echo "✅ No failures! All boards complete."
fi
