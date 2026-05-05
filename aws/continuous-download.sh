#!/bin/bash
# Continuously download completed solutions and DELETE from S3 to save costs
# Run this on your Mac while batch solve is running

S3_BUCKET="poker-solver-solutions"
LOCAL_DIR="${1:-../solutions}"
CHECK_INTERVAL="${2:-300}"  # Check every 5 minutes

echo "=== Continuous Download & Delete ==="
echo "S3: s3://${S3_BUCKET}/"
echo "Local: $LOCAL_DIR"
echo "Check interval: ${CHECK_INTERVAL}s"
echo ""
echo "This will:"
echo "  1. Download new solutions from S3"
echo "  2. DELETE them from S3 to save costs"
echo "  3. Keep checking until all 6,992 are done"
echo ""
read -p "Continue? (y/n) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    exit 1
fi

mkdir -p "$LOCAL_DIR"
mkdir -p "$LOCAL_DIR/logs"

TOTAL_EXPECTED=$(wc -l < boards.txt 2>/dev/null || echo 6992)
DOWNLOADED=0

while true; do
    echo "[$(date +%H:%M:%S)] Checking for new solutions..."

    # List all .bin files in S3
    aws s3 ls s3://${S3_BUCKET}/ --profile poker | grep "\.bin$" | awk '{print $4}' > /tmp/s3_files.txt

    NEW_FILES=0

    # Download each file and delete from S3
    while IFS= read -r file; do
        LOCAL_FILE="$LOCAL_DIR/$file"

        if [ ! -f "$LOCAL_FILE" ]; then
            echo "  ⬇️  Downloading: $file"

            # Download
            if aws s3 cp "s3://${S3_BUCKET}/$file" "$LOCAL_FILE" --profile poker; then
                ((NEW_FILES++))
                ((DOWNLOADED++))

                # Verify download
                if [ -f "$LOCAL_FILE" ] && [ -s "$LOCAL_FILE" ]; then
                    # Delete from S3 to save costs!
                    echo "  🗑️  Deleting from S3: $file"
                    aws s3 rm "s3://${S3_BUCKET}/$file" --profile poker

                    # Also download the log file
                    BOARD=$(echo "$file" | sed 's/v1.1_\(.*\)\.bin/\1/')
                    aws s3 cp "s3://${S3_BUCKET}/logs/v1.1_${BOARD}_log.txt" \
                        "$LOCAL_DIR/logs/" --profile poker 2>/dev/null || true
                    aws s3 rm "s3://${S3_BUCKET}/logs/v1.1_${BOARD}_log.txt" \
                        --profile poker 2>/dev/null || true
                else
                    echo "  ❌ Download verification failed!"
                fi
            else
                echo "  ❌ Download failed: $file"
            fi
        fi
    done < /tmp/s3_files.txt

    # Show progress
    PERCENT=$(echo "scale=1; $DOWNLOADED * 100 / $TOTAL_EXPECTED" | bc)
    SIZE=$(du -sh "$LOCAL_DIR" 2>/dev/null | cut -f1)

    echo "  Progress: $DOWNLOADED / $TOTAL_EXPECTED ($PERCENT%) - $SIZE downloaded"

    if [ "$NEW_FILES" -eq 0 ]; then
        echo "  No new files."
    fi

    # Check if done
    if [ "$DOWNLOADED" -ge "$TOTAL_EXPECTED" ]; then
        echo ""
        echo "✅ All $TOTAL_EXPECTED solutions downloaded!"
        echo "Total size: $(du -sh "$LOCAL_DIR" | cut -f1)"
        exit 0
    fi

    # Wait before next check
    echo "  Sleeping ${CHECK_INTERVAL}s..."
    sleep "$CHECK_INTERVAL"
done
