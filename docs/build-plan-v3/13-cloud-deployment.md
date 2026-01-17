# Section 13: Cloud Deployment

**Time Required**: ~3-4 hours  
**Difficulty**: High  
**Goal**: Deploy to AWS using Pulumi TypeScript for Infrastructure as Code

---

## Overview

This section covers:
1. AWS account setup
2. Pulumi TypeScript infrastructure definition
3. Production deployment
4. Ongoing maintenance

> [!NOTE]
> **Why Pulumi TypeScript instead of Rust?**
> While the architecture document mentions Pulumi Rust SDK for "full Rust stack consistency," we use TypeScript because:
> - Pulumi TypeScript is more mature with better documentation
> - More examples available for LLM assistance
> - Easier to debug with AI coding tools
> 
> This is documented as an acceptable deviation.

Reference: Architecture sections 8, 19

---

## Prerequisites

Before starting:
- [ ] MVP fully working locally
- [ ] All tests passing (CI green)
- [ ] AWS account will be created

---

## AWS Account Setup

### Step 13.1: Create AWS Account

---

#### Step 13.1.1: Manual Steps (Do This Yourself)

1. **Go to** [aws.amazon.com](https://aws.amazon.com)

2. **Click "Create an AWS Account"**

3. **Fill in details**:
   - Email address (use a dedicated email for AWS)
   - Account name: "Investment Research Platform"
   
4. **Verify email** and complete signup

5. **Add payment method** (credit card required)
   - AWS has a free tier for 12 months
   - Set up billing alerts to avoid surprises

6. **Enable MFA** (Multi-Factor Authentication):
   - Go to IAM → Your Security Credentials
   - Enable MFA with authenticator app
   - **This is critical for security**

7. **Create IAM user** (don't use root account):
   - Go to IAM → Users → Create User
   - Name: "iap-admin"
   - Permissions: AdministratorAccess (for setup)
   - Create access keys (for CLI)
   - Save credentials securely

---

### Step 13.2: Configure AWS CLI

---

#### 📋 PROMPT 13.2.1: Install and Configure AWS CLI

```
Set up AWS CLI for deployment.

1. Install AWS CLI:
   ```bash
   curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip"
   unzip awscliv2.zip
   sudo ./aws/install
   aws --version
   ```

2. Configure credentials:
   ```bash
   aws configure
   # Enter:
   # - Access Key ID (from IAM user)
   # - Secret Access Key
   # - Default region: ap-south-1 (Mumbai)
   # - Default output: json
   ```

3. Verify:
   ```bash
   aws sts get-caller-identity
   ```

4. Set up named profile for this project:
   ```bash
   aws configure --profile irp
   ```
```

**Verification**: `aws s3 ls` works without errors.

---

### Step 13.3: Install Pulumi

---

#### 📋 PROMPT 13.3.1: Set Up Pulumi with TypeScript

```
Install and configure Pulumi for Infrastructure as Code.

1. Install Pulumi:
   ```bash
   curl -fsSL https://get.pulumi.com | sh
   export PATH=$PATH:~/.pulumi/bin
   pulumi version
   ```

2. Login to Pulumi (free tier):
   ```bash
   pulumi login
   # Creates account at app.pulumi.com
   ```

3. Create infrastructure project with TypeScript:
   ```bash
   cd iap-alpha/infra
   pulumi new aws-typescript --name iap-infra --yes
   # Select ap-south-1 as region
   ```

   This creates:
   - package.json (Node.js project)
   - Pulumi.yaml (project config)
   - index.ts (infrastructure definition)
   - tsconfig.json

4. Install additional dependencies:
   ```bash
   npm install @pulumi/awsx @pulumi/aws
   ```

5. Configure AWS for Pulumi:
   ```bash
   pulumi config set aws:region ap-south-1
   pulumi config set aws:profile irp
   ```
```

**Verification**: `pulumi preview` runs.

---

### Step 13.4: Define Infrastructure

---

#### 📋 PROMPT 13.4.1: Create VPC and Networking

```
Create the VPC and networking infrastructure with Pulumi TypeScript.

Create/update `infra/index.ts`:

```typescript
import * as pulumi from "@pulumi/pulumi";
import * as aws from "@pulumi/aws";
import * as awsx from "@pulumi/awsx";

const config = new pulumi.Config();
const environment = pulumi.getStack(); // staging or prod

// VPC with public and private subnets
const vpc = new awsx.ec2.Vpc("iap-vpc", {
    cidrBlock: "10.0.0.0/16",
    numberOfAvailabilityZones: 2,
    subnetSpec: [
        { type: "Public", cidrMask: 24 },
        { type: "Private", cidrMask: 24 },
    ],
    natGateways: { strategy: "Single" }, // One NAT to save costs
    tags: { Environment: environment },
});

// Security Groups
const albSg = new aws.ec2.SecurityGroup("alb-sg", {
    vpcId: vpc.vpcId,
    ingress: [
        { protocol: "tcp", fromPort: 80, toPort: 80, cidrBlocks: ["0.0.0.0/0"] },
        { protocol: "tcp", fromPort: 443, toPort: 443, cidrBlocks: ["0.0.0.0/0"] },
    ],
    egress: [
        { protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"] },
    ],
});

const ecsSg = new aws.ec2.SecurityGroup("ecs-sg", {
    vpcId: vpc.vpcId,
    ingress: [
        { protocol: "tcp", fromPort: 8080, toPort: 8080, securityGroups: [albSg.id] },
    ],
    egress: [
        { protocol: "-1", fromPort: 0, toPort: 0, cidrBlocks: ["0.0.0.0/0"] },
    ],
});

const rdsSg = new aws.ec2.SecurityGroup("rds-sg", {
    vpcId: vpc.vpcId,
    ingress: [
        { protocol: "tcp", fromPort: 5432, toPort: 5432, securityGroups: [ecsSg.id] },
    ],
});

// Export values for other components
export const vpcId = vpc.vpcId;
export const publicSubnetIds = vpc.publicSubnetIds;
export const privateSubnetIds = vpc.privateSubnetIds;
```

This creates a production-ready VPC with:
- Public subnets for ALB
- Private subnets for ECS and RDS
- NAT Gateway for outbound internet access
- Security groups with minimal access
```

**Verification**: `pulumi preview` shows VPC resources.

---

#### 📋 PROMPT 13.4.2: Create Database (RDS)

```
Create the PostgreSQL RDS instance.

Add to `infra/index.ts`:

```typescript
// RDS Subnet Group
const dbSubnetGroup = new aws.rds.SubnetGroup("iap-db-subnet", {
    subnetIds: vpc.privateSubnetIds,
    tags: { Environment: environment },
});

// Generate random password
const dbPassword = new pulumi.random.RandomPassword("db-password", {
    length: 32,
    special: true,
});

// RDS PostgreSQL Instance
const db = new aws.rds.Instance("iap-db", {
    engine: "postgres",
    engineVersion: "15",
    instanceClass: environment === "prod" ? "db.t3.medium" : "db.t3.micro",
    allocatedStorage: 20,
    storageEncrypted: true,
    dbSubnetGroupName: dbSubnetGroup.name,
    vpcSecurityGroupIds: [rdsSg.id],
    dbName: "irp",
    username: "irpadmin",
    password: dbPassword.result,
    publiclyAccessible: false,
    skipFinalSnapshot: environment !== "prod",
    backupRetentionPeriod: environment === "prod" ? 7 : 1,
    tags: { Environment: environment },
});

// Store credentials in Secrets Manager
const dbSecret = new aws.secretsmanager.Secret("iap-db-credentials", {
    tags: { Environment: environment },
});

const dbSecretValue = new aws.secretsmanager.SecretVersion("iap-db-credentials-value", {
    secretId: dbSecret.id,
    secretString: pulumi.interpolate`{
        "username": "irpadmin",
        "password": "${dbPassword.result}",
        "host": "${db.address}",
        "port": 5432,
        "database": "irp"
    }`,
});

export const dbEndpoint = db.endpoint;
export const dbSecretArn = dbSecret.arn;
```
```

**Verification**: RDS instance shows in preview.

---

#### 📋 PROMPT 13.4.3: Create ECS Cluster and Services

```
Create ECS Fargate cluster and services.

Add to `infra/index.ts`:

```typescript
// ECR Repositories
const apiRepo = new aws.ecr.Repository("iap-api", {
    forceDelete: environment !== "prod",
});

const frontendRepo = new aws.ecr.Repository("iap-frontend", {
    forceDelete: environment !== "prod",
});

// ECS Cluster
const cluster = new aws.ecs.Cluster("iap-cluster", {
    settings: [{
        name: "containerInsights",
        value: "enabled",
    }],
});

// ALB
const alb = new awsx.lb.ApplicationLoadBalancer("iap-alb", {
    subnetIds: vpc.publicSubnetIds,
    securityGroups: [albSg.id],
});

// API Service
const apiService = new awsx.ecs.FargateService("iap-api", {
    cluster: cluster.arn,
    networkConfiguration: {
        subnets: vpc.privateSubnetIds,
        securityGroups: [ecsSg.id],
    },
    desiredCount: 1,
    taskDefinitionArgs: {
        container: {
            name: "api",
            image: apiRepo.repositoryUrl,
            cpu: 512,
            memory: 1024,
            portMappings: [{ containerPort: 8080, hostPort: 8080 }],
            environment: [
                { name: "ENVIRONMENT", value: environment },
                { name: "RUST_LOG", value: "info" },
            ],
            secrets: [
                { name: "DATABASE_URL", valueFrom: dbSecretArn },
            ],
        },
    },
});

// Frontend Service
const frontendService = new awsx.ecs.FargateService("iap-frontend", {
    cluster: cluster.arn,
    networkConfiguration: {
        subnets: vpc.privateSubnetIds,
        securityGroups: [ecsSg.id],
    },
    desiredCount: 1,
    taskDefinitionArgs: {
        container: {
            name: "frontend",
            image: frontendRepo.repositoryUrl,
            cpu: 256,
            memory: 512,
            portMappings: [{ containerPort: 80, hostPort: 80 }],
        },
    },
});

export const albDnsName = alb.loadBalancer.dnsName;
export const apiRepoUrl = apiRepo.repositoryUrl;
export const frontendRepoUrl = frontendRepo.repositoryUrl;
```
```

**Verification**: ECS cluster shows in preview.

---

#### 📋 PROMPT 13.4.4: Create S3 and CloudFront

```
Create S3 bucket and CloudFront distribution.

Add to `infra/index.ts`:

```typescript
// S3 Bucket for documents
const docsBucket = new aws.s3.BucketV2("iap-documents", {
    forceDestroy: environment !== "prod",
    tags: { Environment: environment },
});

const docsBucketVersioning = new aws.s3.BucketVersioningV2("iap-documents-versioning", {
    bucket: docsBucket.id,
    versioningConfiguration: {
        status: "Enabled",
    },
});

const docsBucketEncryption = new aws.s3.BucketServerSideEncryptionConfigurationV2("iap-documents-encryption", {
    bucket: docsBucket.id,
    rules: [{
        applyServerSideEncryptionByDefault: {
            sseAlgorithm: "AES256",
        },
    }],
});

const docsBucketPublicAccess = new aws.s3.BucketPublicAccessBlock("iap-documents-public-access", {
    bucket: docsBucket.id,
    blockPublicAcls: true,
    blockPublicPolicy: true,
    ignorePublicAcls: true,
    restrictPublicBuckets: true,
});

// CloudFront Distribution (optional, for static assets)
// Add later when you have a custom domain

export const documentsBucketName = docsBucket.bucket;
```
```

**Verification**: S3 bucket shows in preview.

---

### Step 13.5: Deploy Infrastructure

---

#### 📋 PROMPT 13.5.1: Create Deployment Script

```
Create deployment scripts for the application.

Create `scripts/deploy.sh`:

```bash
#!/bin/bash
set -e

ENVIRONMENT=${1:-staging}
echo "Deploying to $ENVIRONMENT..."

# Get AWS account ID
AWS_ACCOUNT_ID=$(aws sts get-caller-identity --query Account --output text)
AWS_REGION=${AWS_REGION:-ap-south-1}
ECR_REGISTRY="${AWS_ACCOUNT_ID}.dkr.ecr.${AWS_REGION}.amazonaws.com"

# 1. ECR Login
echo "Logging into ECR..."
aws ecr get-login-password --region $AWS_REGION | docker login --username AWS --password-stdin $ECR_REGISTRY

# 2. Build and push Docker images
echo "Building API..."
docker build -t iap-api:latest -f backend/Dockerfile .
docker tag iap-api:latest $ECR_REGISTRY/iap-api:latest
docker push $ECR_REGISTRY/iap-api:latest

echo "Building Frontend..."
docker build -t iap-frontend:latest -f frontend/Dockerfile .
docker tag iap-frontend:latest $ECR_REGISTRY/iap-frontend:latest
docker push $ECR_REGISTRY/iap-frontend:latest

# 3. Run database migrations
echo "Running migrations..."
# Get database connection string from Secrets Manager
DB_SECRET=$(aws secretsmanager get-secret-value --secret-id iap-db-credentials --query SecretString --output text)
DATABASE_URL=$(echo $DB_SECRET | jq -r '"postgres://\(.username):\(.password)@\(.host):\(.port)/\(.database)"')

# Run migrations using sqlx-cli (install if needed: cargo install sqlx-cli)
DATABASE_URL=$DATABASE_URL sqlx migrate run --source backend/db/migrations

# 4. Deploy Pulumi stack
echo "Deploying infrastructure..."
cd infra
npm ci
npx pulumi up --stack $ENVIRONMENT --yes

# 5. Force ECS service update
echo "Updating ECS services..."
CLUSTER_NAME="iap-cluster-${ENVIRONMENT}"
aws ecs update-service --cluster $CLUSTER_NAME --service iap-api --force-new-deployment
aws ecs update-service --cluster $CLUSTER_NAME --service iap-frontend --force-new-deployment

echo "Deployment complete!"
echo "ALB URL: $(pulumi stack output albDnsName --stack $ENVIRONMENT)"
```

Make it executable:
```bash
chmod +x scripts/deploy.sh
```

Create production Dockerfiles:
- `backend/Dockerfile` (production, multi-stage)
- `frontend/Dockerfile` (production, nginx)
```

**Verification**: Script runs without errors on dry run.

---

### Step 13.6: First Production Deployment

The first deployment is manual to verify everything:

```bash
# 1. Create Pulumi stack
cd infra
pulumi stack init staging

# 2. Preview changes
pulumi preview

# 3. Deploy infrastructure (this takes 10-15 minutes)
pulumi up

# 4. Run the deployment script
cd ..
./scripts/deploy.sh staging

# 5. Check ECS service status
aws ecs describe-services --cluster iap-cluster-staging --services iap-api

# 6. Get ALB URL and test
ALB_URL=$(cd infra && pulumi stack output albDnsName --stack staging)
curl https://$ALB_URL/health
```

---

### Step 13.7: Git Checkpoint

```bash
git add .

git commit -m "feat(infra): AWS infrastructure with Pulumi TypeScript

Infrastructure:
- VPC with public/private subnets (Pulumi TypeScript)
- RDS PostgreSQL in private subnet
- ECS Fargate cluster with API and frontend services
- Application Load Balancer with path routing
- S3 for document storage with encryption
- Secrets Manager for credentials
- CloudWatch for logging

Deployment:
- Production Dockerfiles
- Deployment script with sqlx-cli migrations
- Pulumi configuration for staging/prod stacks

Note: Using Pulumi TypeScript instead of Rust for better LLM support.

Estimated monthly cost: ~$80"

git push origin develop
```

---

## Cost Summary

| Service | Specification | Monthly Cost |
|---------|---------------|--------------|
| ECS Fargate (API) | 0.5 vCPU, 1GB | ~$15 |
| ECS Fargate (Frontend) | 0.25 vCPU, 0.5GB | ~$8 |
| RDS PostgreSQL | db.t3.micro | ~$15 |
| S3 | 50GB | ~$1 |
| CloudFront | 50GB transfer | ~$5 |
| NAT Gateway | 1 | ~$35 |
| Secrets Manager | 5 secrets | ~$2 |
| **Total** | | **~$80/month** |

> **Tip**: NAT Gateway is expensive. For dev, consider removing it and using VPC endpoints instead.

---

## Verification Checklist

- [ ] AWS account created with MFA
- [ ] AWS CLI configured and working
- [ ] Pulumi installed and logged in
- [ ] Infrastructure deployed to staging
- [ ] Database migrations ran successfully
- [ ] Application accessible via ALB URL
- [ ] Health check passes
- [ ] Commit pushed to GitHub

---

## Next Step

**Proceed to**: [14-antigravity-best-practices.md](./14-antigravity-best-practices.md)
