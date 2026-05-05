#!/bin/bash
# Launch EC2 Spot Instance for Poker Solver
# 70% cheaper than on-demand!

set -e

BOARD=$1
if [ -z "$BOARD" ]; then
    echo "Usage: ./spot-launch.sh <board>"
    echo "Example: ./spot-launch.sh KhQs6h"
    exit 1
fi

echo "Launching spot instance for board: $BOARD"

# Get latest AMI
AMI_ID=$(aws ec2 describe-images \
  --owners amazon \
  --filters "Name=name,Values=al2023-ami-2023.*-x86_64" \
  --query 'Images | sort_by(@, &CreationDate) | [-1].ImageId' \
  --output text \
  --profile poker)

# Launch SPOT instance with user data (auto-setup script)
INSTANCE_ID=$(aws ec2 run-instances \
  --image-id $AMI_ID \
  --instance-type r6a.2xlarge \
  --key-name poker-solver-key \
  --security-groups poker-solver-sg \
  --iam-instance-profile Name=poker-solver-profile \
  --instance-market-options '{
    "MarketType": "spot",
    "SpotOptions": {
      "MaxPrice": "0.25",
      "SpotInstanceType": "one-time",
      "InstanceInterruptionBehavior": "terminate"
    }
  }' \
  --tag-specifications "ResourceType=instance,Tags=[{Key=Name,Value=poker-solver-${BOARD}},{Key=Board,Value=${BOARD}}]" \
  --user-data file://improved-userdata.sh \
  --profile poker \
  --query 'Instances[0].InstanceId' \
  --output text)

echo "Spot instance launched: $INSTANCE_ID"
echo "Board: $BOARD"
echo "Cost: ~$0.03-0.05 (vs $0.16 on-demand)"
echo ""
echo "Instance will auto-configure and solve, then upload to S3."
echo "Check status with: aws ec2 describe-instances --instance-ids $INSTANCE_ID --profile poker"
