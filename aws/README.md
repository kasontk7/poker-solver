# AWS Setup Documentation

## Quick Start

**For launching EC2 and running solves:** See [`LAUNCH.md`](LAUNCH.md)

**For one-time AWS prerequisites:** Continue reading below.

---

## One-Time AWS Prerequisites

These steps only need to be done once per AWS account.

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

# Allow SSH from your IP (IPv4)
MY_IP=$(curl -4 -s ifconfig.me)
aws ec2 authorize-security-group-ingress \
  --group-id $SG_ID \
  --protocol tcp \
  --port 22 \
  --cidr ${MY_IP}/32 \
  --profile poker

# Also allow IPv6 if needed
MY_IP_V6=$(curl -6 -s ifconfig.me 2>/dev/null || echo "")
if [ -n "$MY_IP_V6" ]; then
  aws ec2 authorize-security-group-ingress \
    --group-id $SG_ID \
    --ip-permissions IpProtocol=tcp,FromPort=22,ToPort=22,Ipv6Ranges="[{CidrIpv6=${MY_IP_V6}/128,Description='My IP'}]" \
    --profile poker
fi

echo "Security group created: $SG_ID"
```

### 3. Create Key Pair

```bash
aws ec2 create-key-pair \
  --key-name poker-solver-key \
  --query 'KeyMaterial' \
  --output text \
  --profile poker > ~/.ssh/poker-solver-key.pem

chmod 400 ~/.ssh/poker-solver-key.pem

echo "Key pair saved to ~/.ssh/poker-solver-key.pem"
```

### 4. Create S3 Bucket (if not exists)

```bash
aws s3 mb s3://poker-solver-kason --profile poker

# Upload ranges
aws s3 sync ranges/ s3://poker-solver-kason/v1.1/ranges/ --profile poker
```

---

## Verify Setup

```bash
# Check IAM role
aws iam get-role --role-name poker-solver-ec2-role --profile poker

# Check security group
aws ec2 describe-security-groups --group-names poker-solver-sg --profile poker

# Check key pair
ls -l ~/.ssh/poker-solver-key.pem

# Check S3 bucket
aws s3 ls s3://poker-solver-kason/v1.1/ranges/ --profile poker
```

---

## Files in This Directory

- **`LAUNCH.md`** - Quick EC2 launch guide (use this for running solves)
- **`ec2-setup.sh`** - Automated setup script (runs on EC2)
- **`README.md`** - This file (one-time AWS prerequisites)
- **`v1.1-checklist.md`** - Detailed launch checklist (deprecated, use LAUNCH.md)

---

## Cost Estimates

**v1.1 (single solve):**
- Instance: r6a.2xlarge @ $0.504/hour
- Time: ~7-40 minutes
- Cost: **~$0.06-0.35**

**v1.2 (6,992 solves):**
- Parallel: 100 instances
- Time: ~3-5 days
- Cost: **~$1,000-1,500** (compute + egress)

---

## Troubleshooting

**Security group update (if IP changes):**
```bash
# Get new security group ID
SG_ID=$(aws ec2 describe-security-groups \
  --group-names poker-solver-sg \
  --profile poker \
  --query 'SecurityGroups[0].GroupId' \
  --output text)

# Add new IP
MY_IP=$(curl -4 -s ifconfig.me)
aws ec2 authorize-security-group-ingress \
  --group-id $SG_ID \
  --protocol tcp \
  --port 22 \
  --cidr ${MY_IP}/32 \
  --profile poker
```

**If IAM role already exists:**
- Skip role creation, just verify it's attached to instance profile

**If S3 bucket already exists:**
- Skip bucket creation, just sync ranges
