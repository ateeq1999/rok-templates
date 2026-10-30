# Staging: the whole deployment, small. Its state is in the bucket
# `bootstrap/` made; fill in `backend.bucket` and the variables
# (terraform.tfvars), then `terraform init && terraform apply`.

terraform {
  required_version = ">= 1.10"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.71"
    }
  }
  backend "s3" {
    bucket       = "REPLACE-with-the-bootstrap-state-bucket"
    key          = "staging/terraform.tfstate"
    region       = "us-east-1"
    encrypt      = true
    use_lockfile = true
  }
}

provider "aws" {
  region = var.region
  default_tags {
    tags = { Project = var.name, Environment = "staging" }
  }
}

variable "region" {
  description = "The AWS region."
  type        = string
  default     = "us-east-1"
}

variable "name" {
  description = "Names every resource."
  type        = string
  default     = "api-staging"
}

variable "image_tag" {
  description = "The image tag to run."
  type        = string
}

variable "app_url" {
  description = "APP_URL."
  type        = string
}

variable "mail_domain" {
  description = "The domain mail is sent from."
  type        = string
}

variable "mail_from" {
  description = "MAIL_FROM."
  type        = string
}

variable "cors_allowed_origins" {
  description = "CORS_ALLOWED_ORIGINS."
  type        = list(string)
  default     = []
}

variable "certificate_arn" {
  description = "An ACM certificate for HTTPS."
  type        = string
  default     = null
}

module "app" {
  source               = "../../modules/app"
  name                 = var.name
  image_tag            = var.image_tag
  app_url              = var.app_url
  mail_domain          = var.mail_domain
  mail_from            = var.mail_from
  cors_allowed_origins = var.cors_allowed_origins
  certificate_arn      = var.certificate_arn
  database = {
    instance_class      = "db.t4g.micro"
    allocated_storage   = 20
    multi_az            = false
    deletion_protection = false
  }
  service = {
    cpu          = 256
    memory       = 512
    web_count    = 1
    worker_count = 1
  }
}

output "url" {
  description = "Where the API answers."
  value       = module.app.url
}

output "repository_url" {
  description = "Push the images here."
  value       = module.app.repository_url
}

output "migrate_command" {
  description = "Runs the migrations once before each deploy."
  value       = module.app.migrate_command
}

output "mail_dns_records" {
  description = "The DNS records the mail domain needs."
  value       = module.app.mail_dns_records
}
