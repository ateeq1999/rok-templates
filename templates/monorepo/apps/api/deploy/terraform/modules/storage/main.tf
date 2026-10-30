# The uploads bucket (STORAGE_URL=s3://<bucket>): private, encrypted, and,
# when the browser uploads directly (presigned PUTs), open to the front
# end's origins for PUT and GET only.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

variable "bucket" {
  description = "The bucket's name, unique across AWS."
  type        = string
}

variable "cors_allowed_origins" {
  description = "The front ends that upload and download directly with presigned URLs; none, no CORS rule."
  type        = list(string)
  default     = []
}

variable "force_destroy" {
  description = "Let `terraform destroy` delete a bucket that still holds files (for throwaway environments only)."
  type        = bool
  default     = false
}

resource "aws_s3_bucket" "uploads" {
  bucket        = var.bucket
  force_destroy = var.force_destroy
}

resource "aws_s3_bucket_public_access_block" "uploads" {
  bucket                  = aws_s3_bucket.uploads.id
  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

resource "aws_s3_bucket_server_side_encryption_configuration" "uploads" {
  bucket = aws_s3_bucket.uploads.id
  rule {
    apply_server_side_encryption_by_default {
      sse_algorithm = "AES256"
    }
  }
}

resource "aws_s3_bucket_cors_configuration" "uploads" {
  count  = length(var.cors_allowed_origins) > 0 ? 1 : 0
  bucket = aws_s3_bucket.uploads.id
  cors_rule {
    allowed_methods = ["PUT", "GET"]
    allowed_origins = var.cors_allowed_origins
    allowed_headers = ["*"]
    max_age_seconds = 600
  }
}

output "bucket" {
  description = "The bucket's name: STORAGE_URL is s3://<this>."
  value       = aws_s3_bucket.uploads.bucket
}

output "arn" {
  description = "The bucket's ARN, for the task role's policy."
  value       = aws_s3_bucket.uploads.arn
}
