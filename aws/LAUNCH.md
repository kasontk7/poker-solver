# EC2 Launch Guide for v1.1

**Updated bet sizes:**
- Flop: 50%, 100% | Raises: 3x, 5x
- Turn: 50%, 100%, 150% | Raises: 3x, 5x  
- River: 75%, 150%, all-in | Raises: 3x, 5x

## Quick Start (3 steps)

### Step 1: Launch EC2 Instance

```bash
# Get latest AMI
AMI_ID=$(aws ec2 describe-images \
  --owners amazon \
  --filters "Name=name,Values=al2023-ami-2023.*-x86_64" \
  --query 'Images | sort_by(@, &CreationDate) | [-1].ImageId' \
  --output text \
  --profile poker)

# Launch instance
INSTANCE_ID=$(aws ec2 run-instances \
  --image-id $AMI_ID \
  --instance-type r6a.2xlarge \
  --key-name poker-solver-key \
  --security-groups poker-solver-sg \
  --iam-instance-profile Name=poker-solver-profile \
  --profile poker \
  --query 'Instances[0].InstanceId' \
  --output text)

echo "Instance ID: $INSTANCE_ID"

# Wait for instance to start
aws ec2 wait instance-running --instance-ids $INSTANCE_ID --profile poker

# Get public IP
PUBLIC_IP=$(aws ec2 describe-instances \
  --instance-ids $INSTANCE_ID \
  --query 'Reservations[0].Instances[0].PublicIpAddress' \
  --output text \
  --profile poker)

echo "Ready! SSH with:"
echo "  ssh -i ~/.ssh/poker-solver-key.pem ec2-user@$PUBLIC_IP"
echo ""
echo "SAVE THESE:"
echo "  INSTANCE_ID=$INSTANCE_ID"
echo "  PUBLIC_IP=$PUBLIC_IP"
```

---

### Step 2: Run Setup & Solve

SSH into the instance:
```bash
ssh -i ~/.ssh/poker-solver-key.pem ec2-user@<PUBLIC_IP>
```

Then copy/paste this entire block:
```bash
# Download and run setup script
curl -O https://raw.githubusercontent.com/kasontk7/poker-solver/main/aws/ec2-setup.sh && \
chmod +x ec2-setup.sh && \
./ec2-setup.sh && \
cd poker-solver && \
time ./solver/target/release/poker_solver | tee solve_output.txt
```

**Expected time:** ~2-6 hours for the solve (larger tree with more bet sizes)

---

### Step 3: Download Results & Terminate

**From your Mac (new terminal):**

```bash
# Create local directory
mkdir -p ~/personal/poker-solver/solutions

# Download solution file
scp -i ~/.ssh/poker-solver-key.pem \
  ec2-user@<PUBLIC_IP>:~/poker-solver/solutions/v1.1_KhQs6h.bin \
  ~/personal/poker-solver/solutions/

# Terminate instance
aws ec2 terminate-instances --instance-ids <INSTANCE_ID> --profile poker
```

---

## Cost Estimate

- **Instance:** r6a.2xlarge @ $0.504/hour (8 vCPU, 64 GB RAM)
- **Time:** ~2-6 hours
- **Cost:** ~$1.00-3.00 per solve
- **Data transfer:** ~3-8 GB @ $0/GB (under 100GB free tier)

---

## Troubleshooting

**If setup script fails:**
```bash
# Check what failed
cat ec2-setup.sh

# Run steps manually (see README.md)
```

**If solve crashes:**
```bash
# Check memory usage
free -h

# Check logs
tail solve_output.txt
```

**If download fails:**
```bash
# Verify file exists on EC2
ssh -i ~/.ssh/poker-solver-key.pem ec2-user@<PUBLIC_IP> "ls -lh ~/poker-solver/solutions/"

# Retry download with progress
scp -v -i ~/.ssh/poker-solver-key.pem ...
```

---

## One-Time AWS Prerequisites

If you haven't set up AWS yet, see `aws/README.md` for:
- IAM role creation
- Security group setup
- Key pair generation
- S3 bucket configuration
