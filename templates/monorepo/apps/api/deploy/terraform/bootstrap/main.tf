# Once per AWS account: the bucket staging and prod keep their state in
# (S3 locking, `use_lockfile`, needs no DynamoDB table). Apply it with
# local state, then put its output in envs/*/main.tf's `backend.bucket`.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

variable "region" {
  description = "The AWS region."
  type        = string
  default     = "us-east-1"
}

variable "bucket" {
  description = "The state bucket's name, unique across AWS, such as `shop-terraform-state`."
  type        = string
}

provider "aws" {
  region = var.region
}

resource "aws_s3_bucket" "state" {
  bucket = var.bucket
  lifecycle {
    prevent_destroy = true
  }
}

resource "aws_s3_bucket_versioning" "state" {
  bucket = aws_s3_bucket.state.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_server_side_encryption_configuration" "state" {
  bucket = aws_s3_bucket.state.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_public_access_block" "state" {
  bucket                  = aws_s3_bucket.state.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

output "bucket" {
  description = "Put this in envs/*/main.tf's `backend.bucket`."
  value       = aws_s3_bucket.state.bucket
}
