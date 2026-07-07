terraform {
  required_version = ">= 1.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
  }
}

# ── CASE 1: Provider configuration ────────────────────────────────────────
provider "aws" {
  region     =   var.aws_region
  access_key = var.access_key
  secret_key=var.secret_key

  default_tags {
    tags = {
      Environment = var.environment
      Project     = var.project_name
      ManagedBy   = "Terraform"
    }
  }
}

# ── CASE 2: Variable declarations — mixed spacing ─────────────────────────
variable "aws_region" {
  type=string
  default  = "us-east-1"
  description="The AWS region to deploy to"
}

variable "environment" {
  type        = string
  description = "Environment name (dev/staging/prod)"
  validation {
    condition     = contains(["dev","staging","prod"],var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

# ── CASE 3: Resource with nested blocks ────────────────────────────────────
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "${var.project_name}-vpc"
  }
}

resource "aws_subnet" "public" {
  count             = length(var.availability_zones)
    vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr,8,count.index)
    availability_zone = var.availability_zones[count.index]

    map_public_ip_on_launch = true

  tags = {
    Name = "${var.project_name}-public-${count.index}"
  }
}

# ── CASE 4: Data source ───────────────────────────────────────────────────
data "aws_ami" "ubuntu" {
  most_recent=true
  owners=["099720109477"]

  filter {
    name   = "name"
    values = ["ubuntu/images/hvm-ssd/ubuntu-*-22.04-amd64-server-*"]
  }
}

# ── CASE 5: Output — long value ────────────────────────────────────────────
output "vpc_id" {
  description = "The ID of the VPC"
  value       = aws_vpc.main.id
}

output "very_long_output_name_that_exceeds_line_width" {
  description = "A very long description for this output that might exceed the normal line width limit of eighty characters"
  value       = aws_subnet.public[*].id
}
