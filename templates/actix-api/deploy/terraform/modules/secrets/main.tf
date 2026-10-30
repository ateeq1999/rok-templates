# The API's secrets in Secrets Manager, one secret per variable: the two it
# generates (JWT_SECRET, TOTP_ENCRYPTION_KEY) and those other modules hand
# it (DATABASE_URL, SMTP_URL). ECS injects each as the variable of its name;
# nothing secret is in a task definition or a plain file.
#
# Rotating JWT_SECRET: set its current value as JWT_SECRET_PREVIOUS (a
# `values` entry), taint `random_password.jwt`, apply, and drop the entry
# one JWT_TTL later.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
    random = {
      source  = "hashicorp/random"
      version = "~> 3.7"
    }
  }
}

variable "prefix" {
  description = "The secrets' name prefix, such as `shop-prod/`."
  type        = string
}

variable "names" {
  description = "The variables `values` holds, such as [\"DATABASE_URL\", \"SMTP_URL\"]."
  type        = list(string)
  default     = []
}

variable "values" {
  description = "Each of `names`, its value."
  type        = map(string)
  default     = {}
  sensitive   = true
}

variable "recovery_window_in_days" {
  description = "How long a deleted secret can be restored; 0 deletes at once (throwaway environments)."
  type        = number
  default     = 7
}

resource "random_password" "jwt" {
  length  = 48
  special = false
}

resource "random_bytes" "totp" {
  length = 32
}

locals {
  generated = {
    JWT_SECRET          = random_password.jwt.result
    TOTP_ENCRYPTION_KEY = random_bytes.totp.base64
  }
  names = toset(concat(keys(local.generated), var.names))
}

resource "aws_secretsmanager_secret" "this" {
  for_each                = local.names
  name                    = "${var.prefix}${each.key}"
  recovery_window_in_days = var.recovery_window_in_days
}

resource "aws_secretsmanager_secret_version" "this" {
  for_each      = local.names
  secret_id     = aws_secretsmanager_secret.this[each.key].id
  secret_string = merge(local.generated, var.values)[each.key]
}

output "arns" {
  description = "Each variable's secret ARN, for the task definition's `secrets`."
  value       = { for name, secret in aws_secretsmanager_secret.this : name => secret.arn }
}
