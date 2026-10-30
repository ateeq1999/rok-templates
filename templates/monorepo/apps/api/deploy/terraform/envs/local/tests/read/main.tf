# Reads two of the secrets back, for local.tftest.hcl: what the API gets
# when ECS injects them.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

variable "jwt_secret_arn" {
  type = string
}

variable "totp_key_arn" {
  type = string
}

variable "floci_endpoint" {
  type = string
}

provider "aws" {
  region                      = "us-east-1"
  access_key                  = "test"
  secret_key                  = "test"
  skip_credentials_validation = true
  skip_requesting_account_id  = true
  skip_metadata_api_check     = true
  endpoints {
    secretsmanager = var.floci_endpoint
    sts            = var.floci_endpoint
  }
}

data "aws_secretsmanager_secret_version" "jwt" {
  secret_id = var.jwt_secret_arn
}

data "aws_secretsmanager_secret_version" "totp" {
  secret_id = var.totp_key_arn
}

output "jwt_secret" {
  value     = data.aws_secretsmanager_secret_version.jwt.secret_string
  sensitive = true
}

output "totp_key" {
  value     = data.aws_secretsmanager_secret_version.totp.secret_string
  sensitive = true
}
