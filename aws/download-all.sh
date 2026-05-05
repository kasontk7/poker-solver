#!/bin/bash
# Download all completed solutions from S3

S3_BUCKET="poker-solver-solutions"
LOCAL_DIR="${1:-../solutions}"

echo "=== Downloading All Solutions ==="
echo "S3: s3://${S3_BUCKET}/"
echo "Local: $LOCAL_DIR"
echo ""

mkdir -p "$LOCAL_DIR"

# Sync all .bin files from S3
aws s3 sync \
    s3://${S3_BUCKET}/ \
    "$LOCAL_DIR" \
    --profile poker \
    --exclude "*" \
    --include "v1.1_*.bin" \
    --no-progress

echo ""
echo "Download complete!"
echo ""

# Show stats
TOTAL=$(ls -1 "$LOCAL_DIR"/v1.1_*.bin 2>/dev/null | wc -l)
SIZE=$(du -sh "$LOCAL_DIR" | cut -f1)

echo "Downloaded: $TOTAL solutions"
echo "Total size: $SIZE"
