# EC2 Launch Instructions

Quick reference for launching a single EC2 instance to solve one board.

---

## Launch Instance

```bash
# Get latest AMI
AMI_ID=$(aws ec2 describe-images \
  --owners amazon \
  --filters "Name=name,Values=al2023-ami-2023.*-x86_64" \
  --query 'Images | sort_by(@, &CreationDate) | [-1].ImageId' \
  --output text \
  --profile poker)

# Launch spot instance
INSTANCE_ID=$(aws ec2 run-instances \
  --image-id $AMI_ID \
  --instance-type r6a.2xlarge \
  --key-name poker-solver-key \
  --security-groups poker-solver-sg \
  --iam-instance-profile Name=poker-solver-profile \
  --instance-market-options '{"MarketType":"spot","SpotOptions":{"MaxPrice":"0.25"}}' \
  --tag-specifications 'ResourceType=instance,Tags=[{Key=Name,Value=poker-solver}]' \
  --profile poker \
  --query 'Instances[0].InstanceId' \
  --output text)

echo "Instance ID: $INSTANCE_ID"

# Wait for instance
aws ec2 wait instance-running --instance-ids $INSTANCE_ID --profile poker

# Get IP
PUBLIC_IP=$(aws ec2 describe-instances \
  --instance-ids $INSTANCE_ID \
  --query 'Reservations[0].Instances[0].PublicIpAddress' \
  --output text \
  --profile poker)

echo "SSH: ssh -i ~/.ssh/poker-solver-key.pem ec2-user@$PUBLIC_IP"
```

---

## Run Solve

```bash
# SSH in
ssh -i ~/.ssh/poker-solver-key.pem ec2-user@$PUBLIC_IP

# On EC2
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
sudo dnf install -y git
git clone https://github.com/kasontk7/poker-solver.git
cd poker-solver/solver
cargo build --release --bin poker_solver_parameterized
cd ..
time ./solver/target/release/poker_solver_parameterized KhQs6h | tee solve_output.txt
```

---

## Download Results

```bash
# From Mac
scp -i ~/.ssh/poker-solver-key.pem \
  ec2-user@$PUBLIC_IP:~/poker-solver/solutions/v1.1_KhQs6h.bin \
  ~/personal/poker-solver/solutions/
```

---

## Terminate

```bash
aws ec2 terminate-instances --instance-ids $INSTANCE_ID --profile poker
```

---

**Cost**: ~$0.05 per solve (19 min @ $0.15/hour spot)  
**Time**: ~25 min total (5 min setup + 19 min solve + 1 min download)
