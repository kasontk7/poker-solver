#!/bin/bash
# Check progress of batch solve

S3_BUCKET="poker-solver-solutions"

echo "=== Batch Solve Progress ==="
echo ""

# Count completed (in S3)
COMPLETED=$(aws s3 ls s3://${S3_BUCKET}/ --profile poker | grep "v1.1_.*\.bin" | wc -l)

# Count running instances
RUNNING=$(aws ec2 describe-instances \
    --filters "Name=tag:Name,Values=poker-solver-*" "Name=instance-state-name,Values=pending,running" \
    --profile poker \
    --query 'Reservations[*].Instances[*].InstanceId' \
    --output text | wc -w)

# Total expected (from boards file)
TOTAL=$(wc -l < boards.txt 2>/dev/null || echo "???")

echo "Completed: $COMPLETED / $TOTAL"
echo "Running:   $RUNNING"
echo "Remaining: $((TOTAL - COMPLETED - RUNNING))"
echo ""

# Estimate completion
if [ "$RUNNING" -gt 0 ] && [ "$COMPLETED" -gt 0 ]; then
    AVG_TIME=19  # minutes per solve
    REMAINING=$((TOTAL - COMPLETED))
    ETA_HOURS=$(echo "scale=1; $REMAINING * $AVG_TIME / 60 / $RUNNING" | bc)
    echo "Estimated completion: ${ETA_HOURS} hours (with $RUNNING parallel)"
fi

# Show recent completions
echo ""
echo "Recent completions:"
aws s3 ls s3://${S3_BUCKET}/ --profile poker | grep "v1.1_.*\.bin" | tail -5
