# AWS Setup for v1.1

## Prerequisites (One-Time Setup)

### 1. Create IAM Role for EC2

```bash
# Create trust policy
cat > ec2-trust-policy.json << 'EOF'
{
  "Version": "2012-10-17",
  "Statement": [{
    "Effect": "Allow",
    "Principal": {"Service": "ec2.amazonaws.com"},
    "Action": "sts:AssumeRole"
  }]
}
EOF

# Create role
aws iam create-role \
  --role-name poker-solver-ec2-role \
  --assume-role-policy-document file://ec2-trust-policy.json \
  --profile poker

# Attach S3 access
aws iam attach-role-policy \
  --role-name poker-solver-ec2-role \
  --policy-arn arn:aws:iam::aws:policy/AmazonS3FullAccess \
  --profile poker

# Create instance profile
aws iam create-instance-profile \
  --instance-profile-name poker-solver-profile \
  --profile poker

# Add role to profile
aws iam add-role-to-instance-profile \
  --instance-profile-name poker-solver-profile \
  --role-name poker-solver-ec2-role \
  --profile poker
```

### 2. Create Security Group

```bash
# Create security group
SG_ID=$(aws ec2 create-security-group \
  --group-name poker-solver-sg \
  --description "SSH access for poker solver" \
  --profile poker \
  --query 'GroupId' \
  --output text)

# Allow SSH from your IP
MY_IP=$(curl -s ifconfig.me)
aws ec2 authorize-security-group-ingress \
  --group-id $SG_ID \
  --protocol tcp \
  --port 22 \
  --cidr ${MY_IP}/32 \
  --profile poker
```

### 3. Create Key Pair

```bash
aws ec2 create-key-pair \
  --key-name poker-solver-key \
  --query 'KeyMaterial' \
  --output text \
  --profile poker > ~/.ssh/poker-solver-key.pem

chmod 400 ~/.ssh/poker-solver-key.pem
```

---

## Launch Instance for v1.1

### 1. Launch EC2 Instance

```bash
# Get latest Amazon Linux 2023 AMI
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
echo "Waiting for instance to start..."

# Wait for instance to be running
aws ec2 wait instance-running --instance-ids $INSTANCE_ID --profile poker

# Get public IP
PUBLIC_IP=$(aws ec2 describe-instances \
  --instance-ids $INSTANCE_ID \
  --query 'Reservations[0].Instances[0].PublicIpAddress' \
  --output text \
  --profile poker)

echo "Instance ready!"
echo "Public IP: $PUBLIC_IP"
echo
echo "SSH command:"
echo "  ssh -i ~/.ssh/poker-solver-key.pem ec2-user@$PUBLIC_IP"
```

### 2. SSH into Instance

```bash
ssh -i ~/.ssh/poker-solver-key.pem ec2-user@$PUBLIC_IP
```

### 3. Run Setup Script

```bash
# Inside EC2:
curl -O https://raw.githubusercontent.com/kasontk7/poker-solver/main/aws/v1.1-setup.sh
chmod +x v1.1-setup.sh
./v1.1-setup.sh
```

Or manually step-by-step:
```bash
# 1. Install Rust (~3 min)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env

# 2. Clone repo (~10 sec)
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver

# 3. Download ranges (~10 sec)
aws s3 sync s3://poker-solver-kason/v1.1/ranges/ ranges/

# 4. Compile (~5 min)
cd solver
cargo build --release --bin poker_solver

# 5. Run solver (~30 min)
cd ..
./solver/target/release/poker_solver | tee solve_output.txt

# 6. Upload results (~10 sec)
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
aws s3 cp solve_output.txt s3://poker-solver-kason/v1.1/results/output_${TIMESTAMP}.txt
aws s3 cp solutions/ s3://poker-solver-kason/v1.1/results/solutions_${TIMESTAMP}/ --recursive
```

### 4. Terminate Instance

```bash
# From local machine:
aws ec2 terminate-instances --instance-ids $INSTANCE_ID --profile poker
```

---

## Download Results

```bash
# List results
aws s3 ls s3://poker-solver-kason/v1.1/results/ --profile poker

# Download everything
aws s3 sync s3://poker-solver-kason/v1.1/results/ ~/personal/poker-solver/aws-results/ --profile poker
```

---

## Cost Estimate

**v1.1 (single flop, full tree):**
- Instance: r6a.2xlarge (64 GB RAM)
- Time: ~41 minutes (3 min Rust + 5 min compile + 30 min solve + 3 min overhead)
- Price: $0.504/hour
- **Cost: ~$0.35 per solve**

**v1.2 (full production - 6,992 solves):**
- 6,992 solves × 30 min each = 3,496 hours
- Using spot instances ($0.15/hour, 70% cheaper): **~$524**
- Parallel processing (10 instances): ~350 hours → **15 days**
- S3 storage: ~$20/month for all solutions

**Spot Instance Benefits:**
- Price: ~$0.15/hour (vs $0.504 regular)
- **Cost per solve: ~$0.10**

---

## Troubleshooting

### Out of Memory
If solver crashes with OOM:
- Try 16-bit compression (already enabled in code)
- Use larger instance: r6a.4xlarge (128 GB, $1/hour)

### Compile Errors
- Check Rust version: `rustc --version` (need 1.70+)
- Update Rust: `rustup update`

### S3 Access Denied
- Verify IAM role attached: `aws sts get-caller-identity`
- Check instance profile: `curl http://169.254.169.254/latest/meta-data/iam/info`
