# Local: the modules Floci emulates (storage, secrets, mail, the database)
# applied to Floci on http://localhost:4566, so the Terraform is tested
# without an AWS account. The network and the service (VPC, ECS, the load
# balancer) are checked by `terraform validate` in staging and prod; a
# real staging apply stays the last check (Floci is not AWS).
#
#   docker compose up -d floci        # from the project's root
#   terraform init && terraform test  # here: applies, checks, destroys
#
# State stays local (`terraform.tfstate`, ignored by git).

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
}

variable "floci_endpoint" {
  description = "Where Floci answers."
  type        = string
  default     = "http://localhost:4566"
}

provider "aws" {
  region                      = "us-east-1"
  access_key                  = "test"
  secret_key                  = "test"
  skip_credentials_validation = true
  skip_requesting_account_id  = true
  skip_metadata_api_check     = true
  s3_use_path_style           = true
  endpoints {
    iam            = var.floci_endpoint
    rds            = var.floci_endpoint
    s3             = var.floci_endpoint
    secretsmanager = var.floci_endpoint
    ses            = var.floci_endpoint
    sts            = var.floci_endpoint
  }
}

module "storage" {
  source               = "../../modules/storage"
  bucket               = "api-local-uploads"
  cors_allowed_origins = ["http://localhost:5173"]
  force_destroy        = true
}

module "mail" {
  source = "../../modules/mail"
  name   = "api-local"
  domain = "example.test"
}

module "database" {
  source              = "../../modules/database"
  name                = "api-local"
  deletion_protection = false
}

module "secrets" {
  source                  = "../../modules/secrets"
  prefix                  = "api-local/"
  names                   = ["DATABASE_URL", "SMTP_URL"]
  values                  = { DATABASE_URL = module.database.database_url, SMTP_URL = module.mail.smtp_url }
  recovery_window_in_days = 0
}

output "bucket" {
  description = "The uploads bucket."
  value       = module.storage.bucket
}

output "secret_arns" {
  description = "The API's secrets."
  value       = module.secrets.arns
}

output "database_address" {
  description = "The database's host."
  value       = module.database.address
}

output "mail_dns_records" {
  description = "The mail domain's DNS records."
  value       = module.mail.dns_records
}

output "floci_endpoint" {
  description = "Where Floci answers: what the test reads the secrets back from."
  value       = var.floci_endpoint
}
